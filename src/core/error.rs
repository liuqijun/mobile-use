use thiserror::Error;

#[derive(Error, Debug)]
pub enum MobileUseError {
    #[error("Not connected to any application")]
    NotConnected,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[allow(dead_code)]
    #[error("Ref not found: {0}. Run 'elements' to refresh")]
    RefNotFound(String),

    #[error("ADB error: {0}")]
    AdbError(String),

    #[error("VM Service error: {0}")]
    VmServiceError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MobileUseError>;
