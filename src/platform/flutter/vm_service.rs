use crate::core::{MobileUseError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info};
use url::Url;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<String>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

/// Dart VM Service client for Flutter app communication via WebSocket JSON-RPC 2.0
pub struct VmServiceClient {
    ws: Arc<Mutex<Option<WsStream>>>,
    request_id: AtomicU64,
    isolate_id: Arc<Mutex<Option<String>>>,
}

impl VmServiceClient {
    /// Create a new VmServiceClient instance
    pub fn new() -> Self {
        Self {
            ws: Arc::new(Mutex::new(None)),
            request_id: AtomicU64::new(1),
            isolate_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to VM Service at the given WebSocket URL
    pub async fn connect(&self, url: &str) -> Result<()> {
        let url = Url::parse(url)
            .map_err(|e| MobileUseError::ConnectionFailed(format!("Invalid URL: {}", e)))?;

        info!("Connecting to VM Service: {}", url);

        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            MobileUseError::ConnectionFailed(format!("WebSocket connect failed: {}", e))
        })?;

        *self.ws.lock().await = Some(ws_stream);

        // Get VM info and find main isolate
        let vm_info = self.call("getVM", None).await?;
        debug!("VM Info: {:?}", vm_info);

        // Find the first isolate
        if let Some(isolates) = vm_info.get("isolates").and_then(|v| v.as_array()) {
            if let Some(isolate) = isolates.first() {
                if let Some(id) = isolate.get("id").and_then(|v| v.as_str()) {
                    *self.isolate_id.lock().await = Some(id.to_string());
                    info!("Connected to isolate: {}", id);
                }
            }
        }

        // Validate isolate was found
        if self.isolate_id.lock().await.is_none() {
            return Err(MobileUseError::ConnectionFailed(
                "No Flutter isolate found. Is the app running in debug mode?".to_string(),
            ));
        }

        // Subscribe to Flutter extension stream
        self.call("streamListen", Some(json!({"streamId": "Extension"})))
            .await?;

        info!("Connected to VM Service");
        Ok(())
    }

    /// Disconnect from VM Service
    pub async fn disconnect(&self) -> Result<()> {
        let mut ws = self.ws.lock().await;
        if let Some(ref mut stream) = *ws {
            // Send close frame
            let _ = stream.send(Message::Close(None)).await;
        }
        *ws = None;
        *self.isolate_id.lock().await = None;
        info!("Disconnected from VM Service");
        Ok(())
    }

    /// Check if currently connected to VM Service
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        self.ws.lock().await.is_some()
    }

    /// Call a VM Service method via JSON-RPC
    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst).to_string();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: id.clone(),
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        debug!("Sending: {}", request_json);

        // Hold lock for entire send-receive cycle
        let mut ws_guard = self.ws.lock().await;
        let stream = ws_guard.as_mut().ok_or(MobileUseError::NotConnected)?;

        // Send request
        stream
            .send(Message::Text(request_json))
            .await
            .map_err(|e| MobileUseError::WebSocket(format!("Send failed: {}", e)))?;

        // Receive response with timeout (30 seconds)
        let response = loop {
            let msg = timeout(Duration::from_secs(30), stream.next())
                .await
                .map_err(|_| MobileUseError::Timeout("VM Service response timeout".to_string()))?
                .ok_or_else(|| MobileUseError::WebSocket("Connection closed".to_string()))?
                .map_err(|e| MobileUseError::WebSocket(format!("Receive failed: {}", e)))?;

            let text = msg
                .to_text()
                .map_err(|e| MobileUseError::WebSocket(format!("Invalid message: {}", e)))?;
            debug!("Received: {}", text);

            let parsed: JsonRpcResponse = serde_json::from_str(text)?;

            // Skip events (no id), only return matching response
            if parsed.id.as_ref() == Some(&id) {
                break parsed;
            }
            // Continue loop to skip events and wait for our response
        };

        if let Some(error) = response.error {
            return Err(MobileUseError::VmServiceError(format!(
                "{}: {}",
                error.code, error.message
            )));
        }

        response
            .result
            .ok_or_else(|| MobileUseError::VmServiceError("No result in response".to_string()))
    }

    /// Call a Flutter extension method
    pub async fn call_extension(&self, method: &str, args: Option<Value>) -> Result<Value> {
        let isolate_id = self.isolate_id.lock().await.clone().ok_or_else(|| {
            MobileUseError::VmServiceError("No isolate available. Call connect() first.".to_string())
        })?;

        // Extension methods are called directly with isolateId as param
        let mut params = match args {
            Some(Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };
        params.insert("isolateId".to_string(), Value::String(isolate_id));

        self.call(method, Some(Value::Object(params))).await
    }

    /// Get the semantics tree (accessibility tree)
    #[allow(dead_code)]
    pub async fn get_semantics_tree(&self) -> Result<Value> {
        self.call_extension("ext.flutter.debugDumpSemanticsTreeInTraversalOrder", None)
            .await
    }

    /// Get the render tree (widget render tree)
    #[allow(dead_code)]
    pub async fn get_render_tree(&self) -> Result<Value> {
        self.call_extension("ext.flutter.debugDumpRenderTree", None)
            .await
    }

    /// Trigger a hot reload
    #[allow(dead_code)]
    pub async fn hot_reload(&self) -> Result<Value> {
        self.call_extension("ext.flutter.reassemble", None).await
    }

    /// Trigger a hot restart
    #[allow(dead_code)]
    pub async fn hot_restart(&self) -> Result<Value> {
        self.call_extension("ext.flutter.exit", Some(json!({"pause": false})))
            .await
    }
}

impl Default for VmServiceClient {
    fn default() -> Self {
        Self::new()
    }
}
