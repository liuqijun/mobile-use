//! Flutter process manager for spawning and managing `flutter run --machine` subprocess.
//!
//! This module handles:
//! - Spawning the flutter run process with --machine flag
//! - Parsing JSON output to extract events (VM Service URL, app lifecycle)
//! - Streaming stdout/stderr to an event channel
//! - Providing stdin sender for interactive input (r/R/q)

use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};

/// Find the flutter executable path.
/// Tries `which flutter` first, then fvm, then common installation paths.
fn find_flutter_executable() -> Result<String> {
    // Try using 'which' command to find flutter in PATH
    if let Ok(output) = std::process::Command::new("which")
        .arg("flutter")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Make sure it's an actual path, not an alias description
            if !path.is_empty() && !path.contains("aliased to") && std::path::Path::new(&path).exists() {
                debug!("Found flutter via 'which': {}", path);
                return Ok(path);
            }
        }
    }

    // Try using 'which' to find fvm (Flutter Version Manager)
    if let Ok(output) = std::process::Command::new("which")
        .arg("fvm")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && !path.contains("aliased to") && std::path::Path::new(&path).exists() {
                debug!("Found fvm via 'which': {}", path);
                return Ok(path);
            }
        }
    }

    // Try common fvm locations
    if let Ok(home) = std::env::var("HOME") {
        let fvm_paths = [
            format!("{}/fvm/bin/fvm", home),
            format!("{}/.pub-cache/bin/fvm", home),
        ];

        for fvm_path in fvm_paths {
            if std::path::Path::new(&fvm_path).exists() {
                debug!("Found fvm at: {}", fvm_path);
                return Ok(fvm_path);
            }
        }

        // Try common flutter SDK locations
        let common_paths = [
            format!("{}/flutter/bin/flutter", home),
            format!("{}/development/flutter/bin/flutter", home),
            format!("{}/.flutter/bin/flutter", home),
            "/opt/flutter/bin/flutter".to_string(),
            "/usr/local/flutter/bin/flutter".to_string(),
        ];

        for path in common_paths {
            if std::path::Path::new(&path).exists() {
                debug!("Found flutter at: {}", path);
                return Ok(path);
            }
        }
    }

    // Fall back to just "flutter" and hope it's in PATH
    Ok("flutter".to_string())
}

/// Manager for a running Flutter process
pub struct FlutterProcessManager {
    #[allow(dead_code)]
    child: Child,
    stdin_tx: mpsc::Sender<String>,
}

/// Debug information extracted from app.debugPort event
#[derive(Debug, Clone)]
pub struct AppDebugInfo {
    #[allow(dead_code)]
    pub app_id: String,
    pub ws_uri: String,
    #[allow(dead_code)]
    pub device_id: Option<String>,
}

/// Events emitted by the Flutter process
#[derive(Debug)]
pub enum FlutterEvent {
    /// General log output (non-JSON lines or unrecognized JSON)
    Log(String),
    /// VM Service debug port information
    AppDebugPort(AppDebugInfo),
    /// App has started
    AppStarted(String),
    /// App has stopped
    AppStopped(String),
    /// Error message
    Error(String),
}

/// Internal structures for parsing flutter run --machine JSON output
#[derive(Debug, Deserialize)]
struct FlutterMachineEvent {
    event: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DebugPortParams {
    #[serde(rename = "appId")]
    app_id: String,
    #[serde(rename = "wsUri")]
    ws_uri: String,
    #[serde(rename = "deviceId")]
    device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AppIdParams {
    #[serde(rename = "appId")]
    app_id: String,
}

impl FlutterProcessManager {
    /// Spawn a flutter run process with the given arguments.
    ///
    /// The `--machine` flag is automatically added to enable JSON output parsing.
    ///
    /// Returns the manager and a receiver for Flutter events.
    pub async fn spawn(flutter_args: Vec<String>) -> Result<(Self, mpsc::Receiver<FlutterEvent>)> {
        let (event_tx, event_rx) = mpsc::channel::<FlutterEvent>(100);
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(10);

        // Find flutter executable
        let flutter_path = find_flutter_executable()?;
        debug!("Using flutter executable: {}", flutter_path);

        // Build command with --machine flag
        // Check if we're using fvm (Flutter Version Manager)
        let mut cmd = if flutter_path.ends_with("/fvm") {
            let mut c = Command::new(&flutter_path);
            c.arg("flutter").arg("run").arg("--machine");
            c
        } else {
            let mut c = Command::new(&flutter_path);
            c.arg("run").arg("--machine");
            c
        };

        // Add user-provided arguments
        for arg in &flutter_args {
            cmd.arg(arg);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        debug!("Spawning flutter run with args: {:?}", flutter_args);

        let mut child = cmd
            .spawn()
            .context("Failed to spawn flutter run process. Is Flutter installed and in PATH?")?;

        // Take ownership of stdin
        let mut stdin = child
            .stdin
            .take()
            .context("Failed to open stdin for flutter process")?;

        // Spawn stdin writer task
        tokio::spawn(async move {
            while let Some(input) = stdin_rx.recv().await {
                if let Err(e) = stdin.write_all(input.as_bytes()).await {
                    error!("Failed to write to flutter stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush flutter stdin: {}", e);
                    break;
                }
                trace!("Sent input to flutter: {:?}", input);
            }
            debug!("Flutter stdin writer task ended");
        });

        // Take ownership of stdout and spawn reader task
        if let Some(stdout) = child.stdout.take() {
            let stdout_tx = event_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let event = Self::parse_line(&line);
                    if stdout_tx.send(event).await.is_err() {
                        debug!("Event receiver dropped, stopping stdout reader");
                        break;
                    }
                }
                debug!("Flutter stdout reader task ended");
            });
        }

        // Take ownership of stderr and spawn reader task
        if let Some(stderr) = child.stderr.take() {
            let stderr_tx = event_tx;
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    // Stderr lines are typically errors or warnings
                    let event = if line.to_lowercase().contains("error") {
                        FlutterEvent::Error(line)
                    } else {
                        FlutterEvent::Log(format!("[stderr] {}", line))
                    };
                    if stderr_tx.send(event).await.is_err() {
                        debug!("Event receiver dropped, stopping stderr reader");
                        break;
                    }
                }
                debug!("Flutter stderr reader task ended");
            });
        }

        let manager = FlutterProcessManager { child, stdin_tx };

        Ok((manager, event_rx))
    }

    /// Parse a line of output from flutter run --machine.
    ///
    /// JSON lines are parsed to extract structured events.
    /// Non-JSON lines become Log events.
    fn parse_line(line: &str) -> FlutterEvent {
        let trimmed = line.trim();

        // Flutter --machine outputs JSON arrays like: [{"event":"...","params":{...}}]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Ok(events) = serde_json::from_str::<Vec<FlutterMachineEvent>>(trimmed) {
                if let Some(event) = events.into_iter().next() {
                    return Self::parse_machine_event(event);
                }
            }
        }

        // Also try parsing as a single JSON object (some flutter versions)
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(event) = serde_json::from_str::<FlutterMachineEvent>(trimmed) {
                return Self::parse_machine_event(event);
            }
        }

        // Non-JSON output becomes a log event
        FlutterEvent::Log(line.to_string())
    }

    /// Parse a structured flutter machine event into our FlutterEvent type.
    fn parse_machine_event(event: FlutterMachineEvent) -> FlutterEvent {
        match event.event.as_str() {
            "app.debugPort" => {
                if let Some(params) = event.params {
                    if let Ok(debug_params) = serde_json::from_value::<DebugPortParams>(params) {
                        return FlutterEvent::AppDebugPort(AppDebugInfo {
                            app_id: debug_params.app_id,
                            ws_uri: debug_params.ws_uri,
                            device_id: debug_params.device_id,
                        });
                    }
                }
                warn!("Failed to parse app.debugPort params");
                FlutterEvent::Log(format!("app.debugPort (unparseable)"))
            }
            "app.started" => {
                if let Some(params) = event.params {
                    if let Ok(app_params) = serde_json::from_value::<AppIdParams>(params) {
                        return FlutterEvent::AppStarted(app_params.app_id);
                    }
                }
                FlutterEvent::AppStarted(String::new())
            }
            "app.stop" => {
                if let Some(params) = event.params {
                    if let Ok(app_params) = serde_json::from_value::<AppIdParams>(params) {
                        return FlutterEvent::AppStopped(app_params.app_id);
                    }
                }
                FlutterEvent::AppStopped(String::new())
            }
            _ => {
                // Other events become logs with the event name
                let msg = if let Some(params) = event.params {
                    format!("[{}] {}", event.event, params)
                } else {
                    format!("[{}]", event.event)
                };
                FlutterEvent::Log(msg)
            }
        }
    }

    /// Get a clone of the stdin sender for sending input to the flutter process.
    ///
    /// Multiple clones can be held to allow sending input from different tasks.
    pub fn get_stdin_sender(&self) -> mpsc::Sender<String> {
        self.stdin_tx.clone()
    }

    /// Send input to the flutter process.
    ///
    /// Common inputs:
    /// - "r" - Hot reload
    /// - "R" - Hot restart
    /// - "q" - Quit
    #[allow(dead_code)]
    pub async fn send_input(&self, input: &str) -> Result<()> {
        self.stdin_tx
            .send(input.to_string())
            .await
            .context("Failed to send input to flutter process - channel closed")?;
        Ok(())
    }

    /// Kill the flutter process.
    #[allow(dead_code)]
    pub async fn kill(&mut self) -> Result<()> {
        debug!("Killing flutter process");
        self.child
            .kill()
            .await
            .context("Failed to kill flutter process")?;
        Ok(())
    }

    /// Wait for the process to exit and return the exit status.
    #[allow(dead_code)]
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child
            .wait()
            .await
            .context("Failed to wait for flutter process")
    }

    /// Check if the process is still running.
    #[allow(dead_code)]
    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>> {
        self.child
            .try_wait()
            .context("Failed to check flutter process status")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_debug_port_event() {
        let line = r#"[{"event":"app.debugPort","params":{"appId":"com.example.app","wsUri":"ws://127.0.0.1:12345/abc=/ws","deviceId":"emulator-5554"}}]"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::AppDebugPort(info) => {
                assert_eq!(info.app_id, "com.example.app");
                assert_eq!(info.ws_uri, "ws://127.0.0.1:12345/abc=/ws");
                assert_eq!(info.device_id, Some("emulator-5554".to_string()));
            }
            _ => panic!("Expected AppDebugPort event, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_debug_port_without_device_id() {
        let line = r#"[{"event":"app.debugPort","params":{"appId":"app123","wsUri":"ws://localhost:8080/ws"}}]"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::AppDebugPort(info) => {
                assert_eq!(info.app_id, "app123");
                assert_eq!(info.ws_uri, "ws://localhost:8080/ws");
                assert_eq!(info.device_id, None);
            }
            _ => panic!("Expected AppDebugPort event, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_app_started_event() {
        let line = r#"[{"event":"app.started","params":{"appId":"com.example.app"}}]"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::AppStarted(app_id) => {
                assert_eq!(app_id, "com.example.app");
            }
            _ => panic!("Expected AppStarted event, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_app_stop_event() {
        let line = r#"[{"event":"app.stop","params":{"appId":"com.example.app"}}]"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::AppStopped(app_id) => {
                assert_eq!(app_id, "com.example.app");
            }
            _ => panic!("Expected AppStopped event, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_non_json_line() {
        let line = "Launching lib/main.dart on Android SDK...";
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::Log(msg) => {
                assert_eq!(msg, line);
            }
            _ => panic!("Expected Log event, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_other_event() {
        let line = r#"[{"event":"app.progress","params":{"message":"Building..."}}]"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::Log(msg) => {
                assert!(msg.contains("app.progress"));
            }
            _ => panic!("Expected Log event for unknown event type, got {:?}", event),
        }
    }

    #[test]
    fn test_parse_single_json_object() {
        // Some flutter versions might output single objects instead of arrays
        let line = r#"{"event":"app.started","params":{"appId":"test-app"}}"#;
        let event = FlutterProcessManager::parse_line(line);

        match event {
            FlutterEvent::AppStarted(app_id) => {
                assert_eq!(app_id, "test-app");
            }
            _ => panic!("Expected AppStarted event, got {:?}", event),
        }
    }
}
