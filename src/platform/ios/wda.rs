use crate::core::types::{DeviceOperator, Platform};
use base64::Engine as _;
use serde_json::{json, Value};
use tracing::{debug, info};

/// WebDriverAgent HTTP client for iOS device automation
pub struct WdaClient {
    /// Base URL for WDA HTTP API (e.g., "http://localhost:8100")
    base_url: String,
    /// WDA session ID
    session_id: Option<String>,
    /// Device scale factor (points -> physical pixels)
    scale: f64,
    /// Screen size in physical pixels
    screen_size: Option<(i32, i32)>,
    /// HTTP client
    client: reqwest::blocking::Client,
}

impl WdaClient {
    /// Create a new WdaClient connecting to WDA at the given base URL
    pub fn new(base_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .no_proxy()
            .build()?;

        let mut wda = Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            session_id: None,
            scale: 1.0,
            screen_size: None,
            client,
        };

        // Check WDA status
        wda.check_status()?;
        // Create session
        wda.create_session()?;
        // Detect scale factor
        wda.detect_scale()?;

        Ok(wda)
    }

    /// Get the WDA session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get the base URL
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the scale factor
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Check WDA /status endpoint
    fn check_status(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/status", self.base_url);
        let resp = self.client.get(&url).send()?;
        let body: Value = resp.json()?;
        debug!("WDA status: {:?}", body);
        Ok(body)
    }

    /// Create a WDA session
    fn create_session(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/session", self.base_url);
        let body = json!({
            "capabilities": {
                "alwaysMatch": {}
            }
        });
        let resp = self.client.post(&url).json(&body).send()?;
        let data: Value = resp.json()?;

        let session_id = data["value"]["sessionId"]
            .as_str()
            .or_else(|| data["sessionId"].as_str())
            .ok_or("No sessionId in WDA response")?
            .to_string();

        info!("WDA session created: {}", session_id);
        self.session_id = Some(session_id);
        Ok(())
    }

    /// Detect device scale factor from window size
    fn detect_scale(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session_id = self.session_id.as_ref().ok_or("No session")?;
        let url = format!("{}/session/{}/window/size", self.base_url, session_id);
        let resp = self.client.get(&url).send()?;
        let data: Value = resp.json()?;

        let logical_width = data["value"]["width"].as_f64().unwrap_or(390.0);
        let logical_height = data["value"]["height"].as_f64().unwrap_or(844.0);

        // Get screen info for scale detection
        let screen_url = format!("{}/session/{}/wda/screen", self.base_url, session_id);
        if let Ok(resp) = self.client.get(&screen_url).send() {
            if let Ok(screen_data) = resp.json::<Value>() {
                if let Some(scale) = screen_data["value"]["scale"].as_f64() {
                    self.scale = scale;
                    let phys_w = (logical_width * scale) as i32;
                    let phys_h = (logical_height * scale) as i32;
                    self.screen_size = Some((phys_w, phys_h));
                    info!(
                        "iOS scale={}, screen={}x{} physical pixels",
                        scale, phys_w, phys_h
                    );
                    return Ok(());
                }
            }
        }

        // Fallback: common scale factors by logical width
        self.scale = if logical_width > 400.0 { 3.0 } else { 2.0 };
        let phys_w = (logical_width * self.scale) as i32;
        let phys_h = (logical_height * self.scale) as i32;
        self.screen_size = Some((phys_w, phys_h));
        info!(
            "iOS scale={} (fallback), screen={}x{}",
            self.scale, phys_w, phys_h
        );
        Ok(())
    }

    /// Get session URL prefix
    fn session_url(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let session_id = self.session_id.as_ref().ok_or("No WDA session")?;
        Ok(format!("{}/session/{}", self.base_url, session_id))
    }

    /// Convert physical pixel coordinates to logical points for WDA
    fn to_points(&self, x: i32, y: i32) -> (f64, f64) {
        (x as f64 / self.scale, y as f64 / self.scale)
    }

    /// Perform W3C Actions (pointer actions)
    fn perform_actions(
        &self,
        actions: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/actions", self.session_url()?);
        let body = json!({ "actions": actions });
        debug!(
            "WDA actions: {}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );
        let resp = self.client.post(&url).json(&body).send()?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(format!("WDA action failed ({}): {}", status, text).into());
        }
        Ok(())
    }

    /// Release actions (cleanup after perform)
    fn release_actions(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/actions", self.session_url()?);
        let _ = self.client.delete(&url).send();
        Ok(())
    }
}

impl DeviceOperator for WdaClient {
    fn tap(&self, x: i32, y: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (px, py) = self.to_points(x, y);
        let actions = json!([{
            "type": "pointer",
            "id": "finger1",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "duration": 0, "x": px, "y": py},
                {"type": "pointerDown", "button": 0},
                {"type": "pause", "duration": 50},
                {"type": "pointerUp", "button": 0}
            ]
        }]);
        self.perform_actions(actions)?;
        self.release_actions()?;
        Ok(())
    }

    fn double_tap(&self, x: i32, y: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tap(x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.tap(x, y)?;
        Ok(())
    }

    fn long_press(
        &self,
        x: i32,
        y: i32,
        duration_ms: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (px, py) = self.to_points(x, y);
        let actions = json!([{
            "type": "pointer",
            "id": "finger1",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "duration": 0, "x": px, "y": py},
                {"type": "pointerDown", "button": 0},
                {"type": "pause", "duration": duration_ms},
                {"type": "pointerUp", "button": 0}
            ]
        }]);
        self.perform_actions(actions)?;
        self.release_actions()?;
        Ok(())
    }

    fn swipe(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        duration_ms: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (px1, py1) = self.to_points(x1, y1);
        let (px2, py2) = self.to_points(x2, y2);
        let actions = json!([{
            "type": "pointer",
            "id": "finger1",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "duration": 0, "x": px1, "y": py1},
                {"type": "pointerDown", "button": 0},
                {"type": "pause", "duration": 100},
                {"type": "pointerMove", "duration": duration_ms, "x": px2, "y": py2},
                {"type": "pointerUp", "button": 0}
            ]
        }]);
        self.perform_actions(actions)?;
        self.release_actions()?;
        Ok(())
    }

    fn input_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/wda/keys", self.session_url()?);
        let body = json!({ "value": text.chars().map(|c| c.to_string()).collect::<Vec<_>>() });
        let resp = self.client.post(&url).json(&body).send()?;
        if !resp.status().is_success() {
            let err = resp.text().unwrap_or_default();
            return Err(format!("WDA input_text failed: {}", err).into());
        }
        Ok(())
    }

    fn keyevent(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Map common key names to iOS key values
        let keys: Vec<String> = match key.to_uppercase().as_str() {
            "ENTER" | "KEYCODE_ENTER" => vec!["\n".to_string()],
            "TAB" | "KEYCODE_TAB" => vec!["\t".to_string()],
            "DEL" | "KEYCODE_DEL" | "DELETE" | "67" => vec!["\u{8}".to_string()], // backspace
            "ESCAPE" | "KEYCODE_ESCAPE" => vec!["\u{1b}".to_string()],
            "MOVE_END" | "KEYCODE_MOVE_END" | "123" => {
                vec!["\u{F72B}".to_string()] // NSEndFunctionKey
            }
            _ => vec![key.to_string()],
        };
        let url = format!("{}/wda/keys", self.session_url()?);
        let body = json!({ "value": keys });
        let resp = self.client.post(&url).json(&body).send()?;
        if !resp.status().is_success() {
            let err = resp.text().unwrap_or_default();
            return Err(format!("WDA keyevent failed: {}", err).into());
        }
        Ok(())
    }

    fn screenshot(
        &self,
        local_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/screenshot", self.session_url()?);
        let resp = self.client.get(&url).send()?;
        let data: Value = resp.json()?;
        let b64 = data["value"]
            .as_str()
            .ok_or("No screenshot data in response")?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
        std::fs::write(local_path, &bytes)?;
        info!("iOS screenshot saved to {}", local_path);
        Ok(())
    }

    fn get_screen_size(&self) -> Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>> {
        self.screen_size
            .ok_or_else(|| "Screen size not detected".into())
    }

    fn platform(&self) -> Platform {
        Platform::IOS
    }
}
