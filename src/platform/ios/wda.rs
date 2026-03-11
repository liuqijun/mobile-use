use crate::core::types::{DeviceOperator, Platform};

/// WebDriverAgent HTTP client for iOS device automation
#[allow(dead_code)]
pub struct WdaClient {
    /// Base URL for WDA HTTP API (e.g., "http://localhost:8100")
    base_url: String,
    /// WDA session ID
    session_id: Option<String>,
    /// Device scale factor (logical points to physical pixels)
    scale: f64,
    /// Screen size in physical pixels
    screen_size: Option<(i32, i32)>,
    /// HTTP client
    client: reqwest::blocking::Client,
}
