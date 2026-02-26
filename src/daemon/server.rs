use crate::commands;
use crate::core::{RefMap, Result};
use crate::daemon::{DaemonRequest, DaemonResponse, SessionManager};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info};

/// WebSocket URL template for connecting to Flutter VM Service
const WS_URL_TEMPLATE: &str = "ws://127.0.0.1:{}/ws";

/// Helper macro for session lookup with error handling
macro_rules! with_session {
    ($sessions:expr, $session_name:expr, |$sess:ident| $body:expr) => {{
        let sessions_guard = $sessions.lock().await;
        match sessions_guard.get($session_name) {
            Some($sess) => $body,
            None => DaemonResponse::error(format!("Session not found: {}", $session_name)),
        }
    }};
    (mut $sessions:expr, $session_name:expr, |$sess:ident| $body:expr) => {{
        let mut sessions_guard = $sessions.lock().await;
        match sessions_guard.get_mut($session_name) {
            Some($sess) => $body,
            None => DaemonResponse::error(format!("Session not found: {}", $session_name)),
        }
    }};
}

/// Get the Unix socket path for the daemon
pub fn get_socket_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("daemon.sock")
}

/// Get the PID file path for the daemon
pub fn get_pid_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("daemon.pid")
}

/// Daemon server that listens on Unix socket and handles requests
pub struct DaemonServer {
    /// Shared session manager
    sessions: Arc<Mutex<SessionManager>>,
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,
}

impl DaemonServer {
    /// Create a new daemon server
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            sessions: Arc::new(Mutex::new(SessionManager::new())),
            shutdown_tx,
        }
    }

    /// Run the daemon server
    pub async fn run(&self) -> Result<()> {
        let socket_path = get_socket_path();
        let pid_path = get_pid_path();

        // Create parent directory
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Remove old socket file if it exists
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        // Write PID file
        let pid = std::process::id();
        tokio::fs::write(&pid_path, pid.to_string()).await?;
        info!("Daemon started with PID {} at {:?}", pid, socket_path);

        // Bind Unix listener
        let listener = UnixListener::bind(&socket_path)?;
        info!("Listening on {:?}", socket_path);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                // Accept new connections
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let sessions = Arc::clone(&self.sessions);
                            let shutdown_tx = self.shutdown_tx.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, sessions, shutdown_tx).await {
                                    error!("Client handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        // Cleanup
        if socket_path.exists() {
            let _ = tokio::fs::remove_file(&socket_path).await;
        }
        if pid_path.exists() {
            let _ = tokio::fs::remove_file(&pid_path).await;
        }

        info!("Daemon server stopped");
        Ok(())
    }

    /// Send shutdown signal to stop the server
    #[allow(dead_code)]
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl Default for DaemonServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a single client connection
async fn handle_client(
    stream: UnixStream,
    sessions: Arc<Mutex<SessionManager>>,
    shutdown_tx: broadcast::Sender<()>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            // Connection closed
            debug!("Client disconnected");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        debug!("Received request: {}", trimmed);

        // Parse request
        let response = match serde_json::from_str::<DaemonRequest>(trimmed) {
            Ok(request) => handle_request(request, &sessions, &shutdown_tx).await,
            Err(e) => DaemonResponse::error(format!("Invalid request: {}", e)),
        };

        // Write response
        let response_json = serde_json::to_string(&response)?;
        debug!("Sending response: {}", response_json);
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

/// Handle a daemon request and return a response
async fn handle_request(
    request: DaemonRequest,
    sessions: &Arc<Mutex<SessionManager>>,
    shutdown_tx: &broadcast::Sender<()>,
) -> DaemonResponse {
    match request {
        DaemonRequest::Ping => DaemonResponse::ok(Some(json!({"status": "alive"}))),

        DaemonRequest::Shutdown => {
            let _ = shutdown_tx.send(());
            DaemonResponse::ok(Some(json!({"status": "shutting_down"})))
        }

        DaemonRequest::Connect {
            session,
            device,
            url,
            port,
        } => {
            // Determine VM Service URL first (may require async auto-discover)
            let vm_url = if let Some(url) = url {
                url
            } else if let Some(port) = port {
                WS_URL_TEMPLATE.replace("{}", &port.to_string())
            } else {
                // Auto-discover - need ADB client from session
                let adb = {
                    let mut sessions_guard = sessions.lock().await;
                    let daemon_session = sessions_guard.get_or_create(&session, device.clone());
                    daemon_session.adb.clone()
                };
                // Lock released, now do async work
                match commands::find_flutter_vm_service(&adb).await {
                    Ok(url) => url,
                    Err(e) => return DaemonResponse::error(format!("Auto-discover failed: {}", e)),
                }
            };

            // Now lock and connect to VM Service
            let mut sessions_guard = sessions.lock().await;
            let daemon_session = sessions_guard.get_or_create(&session, device.clone());

            match daemon_session.vm_service.connect(&vm_url).await {
                Ok(()) => {
                    daemon_session.vm_url = Some(vm_url.clone());
                    DaemonResponse::ok(Some(json!({
                        "session": session,
                        "url": vm_url,
                        "connected": true
                    })))
                }
                Err(e) => DaemonResponse::error(format!("Connection failed: {}", e)),
            }
        }

        DaemonRequest::Disconnect { session } => {
            let mut sessions_guard = sessions.lock().await;
            if let Some(daemon_session) = sessions_guard.remove(&session) {
                if let Err(e) = daemon_session.vm_service.disconnect().await {
                    return DaemonResponse::error(format!("Disconnect failed: {}", e));
                }
                DaemonResponse::ok(Some(json!({
                    "session": session,
                    "disconnected": true
                })))
            } else {
                DaemonResponse::error(format!("Session not found: {}", session))
            }
        }

        DaemonRequest::Call {
            session,
            method,
            params,
        } => {
            with_session!(sessions, &session, |sess| {
                // HashMap<String, Value> -> Value conversion cannot fail
                let params_value = params.map(|p| Value::Object(p.into_iter().collect()));
                match sess.vm_service.call(&method, params_value).await {
                    Ok(result) => DaemonResponse::ok(Some(result)),
                    Err(e) => DaemonResponse::error(format!("Call failed: {}", e)),
                }
            })
        }

        DaemonRequest::CallExtension {
            session,
            method,
            args,
        } => {
            with_session!(sessions, &session, |sess| {
                // HashMap<String, Value> -> Value conversion cannot fail
                let args_value = args.map(|a| Value::Object(a.into_iter().collect()));
                match sess.vm_service.call_extension(&method, args_value).await {
                    Ok(result) => DaemonResponse::ok(Some(result)),
                    Err(e) => DaemonResponse::error(format!("Extension call failed: {}", e)),
                }
            })
        }

        DaemonRequest::Info { session } => {
            with_session!(sessions, &session, |sess| {
                let mode = if sess.is_android_mode() {
                    "android"
                } else if sess.vm_url.is_some() {
                    "flutter"
                } else {
                    "disconnected"
                };
                DaemonResponse::ok(Some(json!({
                    "session": sess.name,
                    "device": sess.device,
                    "vm_url": sess.vm_url,
                    "package": sess.package,
                    "mode": mode,
                    "connected": sess.is_connected()
                })))
            })
        }

        DaemonRequest::StoreRefs { session, refs } => {
            with_session!(mut sessions, &session, |sess| {
                // Calculate new counter based on refs
                let max_counter = refs
                    .keys()
                    .filter_map(|k| k.strip_prefix('e').and_then(|n| n.parse::<u32>().ok()))
                    .max()
                    .unwrap_or(0);
                sess.ref_map = RefMap::with_refs(refs, max_counter);
                DaemonResponse::ok(Some(json!({
                    "stored": true,
                    "count": sess.ref_map.refs.len()
                })))
            })
        }

        DaemonRequest::GetRefs { session } => {
            with_session!(sessions, &session, |sess| {
                let refs_json = serde_json::to_value(&sess.ref_map.refs)
                    .unwrap_or(Value::Object(serde_json::Map::new()));
                DaemonResponse::ok(Some(refs_json))
            })
        }

        DaemonRequest::ResolveRef { session, reference } => {
            with_session!(sessions, &session, |sess| {
                if let Some(element_ref) = sess.ref_map.get(&reference) {
                    match serde_json::to_value(element_ref) {
                        Ok(value) => DaemonResponse::ok(Some(value)),
                        Err(e) => DaemonResponse::error(format!("Serialize failed: {}", e)),
                    }
                } else {
                    DaemonResponse::error(format!("Reference not found: {}", reference))
                }
            })
        }

        DaemonRequest::RegisterFlutterProcess { session } => {
            with_session!(mut sessions, &session, |sess| {
                sess.has_flutter_process = true;
                DaemonResponse::ok(Some(json!({
                    "session": session,
                    "registered": true
                })))
            })
        }

        DaemonRequest::SendFlutterInput { session, input: _input } => {
            // Note: The actual stdin sender is managed client-side in the run command.
            // This request is a placeholder for future IPC if needed.
            // For now, return error since stdin is not daemon-managed.
            with_session!(sessions, &session, |sess| {
                if sess.has_flutter_process {
                    // Stdin is managed client-side, not by daemon
                    DaemonResponse::error(
                        "Flutter stdin is managed client-side, not by daemon".to_string(),
                    )
                } else {
                    DaemonResponse::error(format!(
                        "No flutter process registered for session: {}",
                        session
                    ))
                }
            })
        }

        DaemonRequest::ConnectAndroid {
            session,
            device,
            package,
        } => {
            let mut sessions_guard = sessions.lock().await;
            let daemon_session = sessions_guard.get_or_create(&session, device.clone());
            daemon_session.package = Some(package.clone());
            // No VM Service connection needed for native Android
            DaemonResponse::ok(Some(json!({
                "session": session,
                "package": package,
                "mode": "android",
                "connected": true
            })))
        }

        DaemonRequest::HasFlutterProcess { session } => {
            with_session!(sessions, &session, |sess| {
                DaemonResponse::HasFlutterProcess {
                    has_process: sess.has_flutter_process,
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ElementRef;
    use std::collections::HashMap;

    #[test]
    fn test_get_socket_path() {
        let path = get_socket_path();
        assert!(path.to_string_lossy().contains("mobile-use"));
        assert!(path.to_string_lossy().contains("daemon.sock"));
    }

    #[test]
    fn test_get_pid_path() {
        let path = get_pid_path();
        assert!(path.to_string_lossy().contains("mobile-use"));
        assert!(path.to_string_lossy().contains("daemon.pid"));
    }

    #[test]
    fn test_daemon_server_new() {
        let server = DaemonServer::new();
        // Just verify it can be created
        assert!(Arc::strong_count(&server.sessions) == 1);
    }

    #[test]
    fn test_daemon_server_default() {
        let server = DaemonServer::default();
        assert!(Arc::strong_count(&server.sessions) == 1);
    }

    #[tokio::test]
    async fn test_handle_request_ping() {
        let sessions = Arc::new(Mutex::new(SessionManager::new()));
        let (shutdown_tx, _) = broadcast::channel(1);

        let response = handle_request(DaemonRequest::Ping, &sessions, &shutdown_tx).await;

        match response {
            DaemonResponse::Ok { data } => {
                let data = data.unwrap();
                assert_eq!(data["status"], "alive");
            }
            DaemonResponse::Error { message } => panic!("Expected Ok, got Error: {}", message),
            DaemonResponse::HasFlutterProcess { .. } => {
                panic!("Expected Ok, got HasFlutterProcess")
            }
        }
    }

    #[tokio::test]
    async fn test_handle_request_info_not_found() {
        let sessions = Arc::new(Mutex::new(SessionManager::new()));
        let (shutdown_tx, _) = broadcast::channel(1);

        let response = handle_request(
            DaemonRequest::Info {
                session: "nonexistent".to_string(),
            },
            &sessions,
            &shutdown_tx,
        )
        .await;

        match response {
            DaemonResponse::Error { message } => {
                assert!(message.contains("Session not found"));
            }
            DaemonResponse::Ok { .. } => panic!("Expected Error, got Ok"),
            DaemonResponse::HasFlutterProcess { .. } => {
                panic!("Expected Error, got HasFlutterProcess")
            }
        }
    }

    #[tokio::test]
    async fn test_handle_request_store_and_get_refs() {
        let sessions = Arc::new(Mutex::new(SessionManager::new()));
        let (shutdown_tx, _) = broadcast::channel(1);

        // First create a session by connecting (we'll skip the actual connection)
        {
            let mut sessions_guard = sessions.lock().await;
            sessions_guard.get_or_create("test", None);
        }

        // Store refs
        let mut refs = HashMap::new();
        refs.insert(
            "e1".to_string(),
            ElementRef {
                ref_id: "e1".to_string(),
                element_type: "button".to_string(),
                label: Some("Submit".to_string()),
                bounds: crate::core::Bounds {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                properties: HashMap::new(),
                style: None,
            },
        );

        let response = handle_request(
            DaemonRequest::StoreRefs {
                session: "test".to_string(),
                refs,
            },
            &sessions,
            &shutdown_tx,
        )
        .await;

        match response {
            DaemonResponse::Ok { data } => {
                let data = data.unwrap();
                assert_eq!(data["stored"], true);
                assert_eq!(data["count"], 1);
            }
            DaemonResponse::Error { message } => panic!("Expected Ok, got Error: {}", message),
            DaemonResponse::HasFlutterProcess { .. } => {
                panic!("Expected Ok, got HasFlutterProcess")
            }
        }

        // Get refs
        let response = handle_request(
            DaemonRequest::GetRefs {
                session: "test".to_string(),
            },
            &sessions,
            &shutdown_tx,
        )
        .await;

        match response {
            DaemonResponse::Ok { data } => {
                let data = data.unwrap();
                assert!(data["e1"].is_object());
            }
            DaemonResponse::Error { message } => panic!("Expected Ok, got Error: {}", message),
            DaemonResponse::HasFlutterProcess { .. } => {
                panic!("Expected Ok, got HasFlutterProcess")
            }
        }
    }

    #[tokio::test]
    async fn test_handle_request_resolve_ref() {
        let sessions = Arc::new(Mutex::new(SessionManager::new()));
        let (shutdown_tx, _) = broadcast::channel(1);

        // Create session and store refs
        {
            let mut sessions_guard = sessions.lock().await;
            let session = sessions_guard.get_or_create("test", None);
            session.ref_map.add(ElementRef {
                ref_id: String::new(),
                element_type: "button".to_string(),
                label: Some("Submit".to_string()),
                bounds: crate::core::Bounds {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                properties: HashMap::new(),
                style: None,
            });
        }

        // Resolve ref
        let response = handle_request(
            DaemonRequest::ResolveRef {
                session: "test".to_string(),
                reference: "@e1".to_string(),
            },
            &sessions,
            &shutdown_tx,
        )
        .await;

        match response {
            DaemonResponse::Ok { data } => {
                let data = data.unwrap();
                assert_eq!(data["element_type"], "button");
                assert_eq!(data["label"], "Submit");
            }
            DaemonResponse::Error { message } => panic!("Expected Ok, got Error: {}", message),
            DaemonResponse::HasFlutterProcess { .. } => {
                panic!("Expected Ok, got HasFlutterProcess")
            }
        }
    }
}
