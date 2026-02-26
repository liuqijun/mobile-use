use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::ElementRef;

/// Request from CLI to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequest {
    /// Connect to Flutter VM Service
    Connect {
        session: String,
        device: Option<String>,
        url: Option<String>,
        port: Option<u16>,
    },
    /// Disconnect and clear session
    Disconnect { session: String },
    /// Call VM Service method
    Call {
        session: String,
        method: String,
        params: Option<HashMap<String, Value>>,
    },
    /// Call Flutter extension method
    CallExtension {
        session: String,
        method: String,
        args: Option<HashMap<String, Value>>,
    },
    /// Get session info
    Info { session: String },
    /// Store element refs
    StoreRefs {
        session: String,
        refs: HashMap<String, ElementRef>,
    },
    /// Get element refs
    GetRefs { session: String },
    /// Resolve element reference
    ResolveRef { session: String, reference: String },
    /// Register a flutter process stdin sender with a session
    RegisterFlutterProcess { session: String },
    /// Send input to flutter process stdin
    SendFlutterInput { session: String, input: String },
    /// Check if session has a flutter process registered
    HasFlutterProcess { session: String },
    /// Connect to native Android app (ADB-only, no VM Service)
    ConnectAndroid {
        session: String,
        device: Option<String>,
        package: String,
    },
    /// Health check
    Ping,
    /// Stop daemon
    Shutdown,
}

/// Response from daemon to CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonResponse {
    /// Success response
    Ok { data: Option<Value> },
    /// Error response
    Error { message: String },
    /// Response for HasFlutterProcess
    HasFlutterProcess { has_process: bool },
}

impl DaemonResponse {
    /// Create a success response with optional data
    pub fn ok(data: Option<Value>) -> Self {
        DaemonResponse::Ok { data }
    }

    /// Create an error response with a message
    pub fn error(message: impl Into<String>) -> Self {
        DaemonResponse::Error {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = DaemonRequest::Connect {
            session: "test".to_string(),
            device: None,
            url: Some("ws://127.0.0.1:12345/ws".to_string()),
            port: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"connect\""));
        assert!(json.contains("\"session\":\"test\""));
    }

    #[test]
    fn test_response_ok() {
        let response = DaemonResponse::ok(Some(serde_json::json!({"status": "connected"})));
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"type\":\"ok\""));
        assert!(json.contains("\"status\":\"connected\""));
    }

    #[test]
    fn test_response_error() {
        let response = DaemonResponse::error("Connection failed");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"Connection failed\""));
    }

    #[test]
    fn test_ping_request() {
        let request = DaemonRequest::Ping;
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, "{\"type\":\"ping\"}");
    }

    #[test]
    fn test_shutdown_request() {
        let request = DaemonRequest::Shutdown;
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, "{\"type\":\"shutdown\"}");
    }
}
