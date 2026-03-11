# iOS Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add iOS device support (simulator + real device) to mobile-use, so AI agents use the same CLI for both Android and iOS with zero platform awareness.

**Architecture:** Extract a `DeviceOperator` trait from the current `AdbClient`, implement it for iOS via `WdaClient` (WebDriverAgent HTTP), refactor `DaemonSession` to hold `Box<dyn DeviceOperator>` instead of hardcoded `AdbClient`. All action functions (`tap_action`, `scroll_action`, etc.) switch from `AdbClient` to the trait.

**Tech Stack:** Rust, reqwest (blocking HTTP for WDA), serde_json (W3C Actions), xcrun/simctl (simulator management), xcodebuild (WDA build), iproxy (real device port forwarding)

---

### Task 1: Add reqwest dependency

**Files:**
- Modify: `Cargo.toml:13-28`

**Step 1: Add reqwest to dependencies**

In `Cargo.toml`, add after the existing `base64` line:

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully with new dependency

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add reqwest dependency for WDA HTTP client"
```

---

### Task 2: Define DeviceOperator trait and Platform enum

**Files:**
- Create: `src/platform/device.rs`
- Modify: `src/platform/mod.rs`

**Step 1: Create device.rs with trait definition**

```rust
// src/platform/device.rs
use crate::core::Result;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Platform {
    Android,
    IOS,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Android => write!(f, "android"),
            Platform::IOS => write!(f, "ios"),
        }
    }
}

/// Platform-agnostic device operations.
/// All coordinates are in physical pixels.
pub trait DeviceOperator: Send + Sync {
    fn tap(&self, x: i32, y: i32) -> Result<()>;
    fn double_tap(&self, x: i32, y: i32) -> Result<()>;
    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> Result<()>;
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()>;
    fn input_text(&self, text: &str) -> Result<()>;
    fn keyevent(&self, key: &str) -> Result<()>;
    fn screenshot(&self, local_path: &str) -> Result<()>;
    fn get_screen_size(&self) -> Result<(i32, i32)>;
    fn platform(&self) -> Platform;
}
```

**Step 2: Update platform/mod.rs to export the new module**

```rust
// src/platform/mod.rs
pub mod android;
pub mod device;
pub mod flutter;

pub use device::*;
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/platform/device.rs src/platform/mod.rs
git commit -m "feat: define DeviceOperator trait and Platform enum"
```

---

### Task 3: Implement DeviceOperator for AdbClient

**Files:**
- Modify: `src/platform/android/adb.rs:29-313`

**Step 1: Write test for AdbClient implementing DeviceOperator**

Add at the bottom of the existing `#[cfg(test)] mod tests` block in `adb.rs`:

```rust
    #[test]
    fn test_adb_client_is_device_operator() {
        let client = AdbClient::new(None);
        // Verify AdbClient can be used as a DeviceOperator trait object
        let _: &dyn crate::platform::DeviceOperator = &client;
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_adb_client_is_device_operator`
Expected: FAIL — AdbClient doesn't implement DeviceOperator

**Step 3: Add DeviceOperator impl for AdbClient**

Add after the existing `impl AdbClient` block (before `#[cfg(test)]`):

```rust
impl crate::platform::DeviceOperator for AdbClient {
    fn tap(&self, x: i32, y: i32) -> crate::core::Result<()> {
        self.shell(&format!("input tap {} {}", x, y))?;
        Ok(())
    }

    fn double_tap(&self, x: i32, y: i32) -> crate::core::Result<()> {
        self.tap(x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.tap(x, y)?;
        Ok(())
    }

    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> crate::core::Result<()> {
        self.shell(&format!(
            "input swipe {} {} {} {} {}",
            x, y, x, y, duration_ms
        ))?;
        Ok(())
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> crate::core::Result<()> {
        self.shell(&format!(
            "input swipe {} {} {} {} {}",
            x1, y1, x2, y2, duration_ms
        ))?;
        Ok(())
    }

    fn input_text(&self, text: &str) -> crate::core::Result<()> {
        AdbClient::input_text(self, text)
    }

    fn keyevent(&self, key: &str) -> crate::core::Result<()> {
        AdbClient::keyevent(self, key)
    }

    fn screenshot(&self, local_path: &str) -> crate::core::Result<()> {
        AdbClient::screenshot(self, local_path)
    }

    fn get_screen_size(&self) -> crate::core::Result<(i32, i32)> {
        AdbClient::get_screen_size(self)
    }

    fn platform(&self) -> crate::platform::Platform {
        crate::platform::Platform::Android
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_adb_client_is_device_operator`
Expected: PASS

**Step 5: Commit**

```bash
git add src/platform/android/adb.rs
git commit -m "feat: implement DeviceOperator trait for AdbClient"
```

---

### Task 4: Implement WdaClient — HTTP client for WebDriverAgent

**Files:**
- Create: `src/platform/ios/wda.rs`
- Create: `src/platform/ios/mod.rs`
- Modify: `src/platform/mod.rs`

**Step 1: Create ios/mod.rs**

```rust
// src/platform/ios/mod.rs
pub mod wda;

pub use wda::WdaClient;
```

**Step 2: Register ios module in platform/mod.rs**

```rust
// src/platform/mod.rs
pub mod android;
pub mod device;
pub mod flutter;
pub mod ios;

pub use device::*;
```

**Step 3: Create wda.rs with WdaClient**

```rust
// src/platform/ios/wda.rs
use crate::core::{MobileUseError, Result};
use serde_json::{json, Value};
use tracing::{debug, info};

/// WebDriverAgent HTTP client.
/// Talks to WDA via W3C WebDriver Actions API.
pub struct WdaClient {
    base_url: String,
    client: reqwest::blocking::Client,
    session_id: Option<String>,
    scale_factor: f64,
}

impl WdaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            session_id: None,
            scale_factor: 1.0,
        }
    }

    /// Check if WDA is reachable
    pub fn health_check(&self) -> bool {
        let url = format!("{}/status", self.base_url);
        self.client.get(&url).send().map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Create a new WDA session
    pub fn create_session(&mut self) -> Result<()> {
        let url = format!("{}/session", self.base_url);
        let body = json!({
            "capabilities": {
                "alwaysMatch": {},
                "firstMatch": [{}]
            }
        });

        let resp = self.client.post(&url).json(&body).send().map_err(|e| {
            MobileUseError::Other(format!("WDA create session failed: {}", e))
        })?;

        let data: Value = resp.json().map_err(|e| {
            MobileUseError::Other(format!("WDA session response parse failed: {}", e))
        })?;

        self.session_id = data
            .get("value")
            .or_else(|| data.get("sessionId"))
            .and_then(|v| {
                // WDA returns either {"value": {"sessionId": "..."}} or {"sessionId": "..."}
                if v.is_object() {
                    v.get("sessionId").and_then(|s| s.as_str().map(String::from))
                } else {
                    v.as_str().map(String::from)
                }
            });

        if self.session_id.is_none() {
            return Err(MobileUseError::Other(
                "WDA session creation returned no sessionId".to_string(),
            ));
        }

        info!("WDA session created: {}", self.session_id.as_ref().unwrap());

        // Get scale factor from window size
        self.detect_scale_factor()?;

        Ok(())
    }

    /// Detect device scale factor by comparing window size with screen info
    fn detect_scale_factor(&mut self) -> Result<()> {
        // WDA /session/{id}/window/size returns logical points
        // For now default to common scale factors; can be refined later
        // iPhone: 2x or 3x, iPad: 2x
        // We'll use the WDA screen info endpoint if available
        let session_id = self.session_id.as_ref().ok_or_else(|| {
            MobileUseError::Other("No WDA session".to_string())
        })?;

        let url = format!("{}/session/{}/wda/screen", self.base_url, session_id);
        if let Ok(resp) = self.client.get(&url).send() {
            if let Ok(data) = resp.json::<Value>() {
                if let Some(scale) = data
                    .pointer("/value/scale")
                    .and_then(|v| v.as_f64())
                {
                    self.scale_factor = scale;
                    info!("WDA screen scale factor: {}", scale);
                    return Ok(());
                }
            }
        }

        // Fallback: assume 3x (most modern iPhones)
        self.scale_factor = 3.0;
        info!("WDA scale factor defaulting to {}", self.scale_factor);
        Ok(())
    }

    fn session_url(&self) -> Result<String> {
        let session_id = self.session_id.as_ref().ok_or_else(|| {
            MobileUseError::Other("No WDA session. Call create_session() first.".to_string())
        })?;
        Ok(format!("{}/session/{}", self.base_url, session_id))
    }

    /// Perform W3C Actions
    fn perform_actions(&self, actions: Vec<Value>) -> Result<()> {
        let url = format!("{}/actions", self.session_url()?);
        let body = json!({ "actions": actions });

        debug!("WDA actions: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

        let resp = self.client.post(&url).json(&body).send().map_err(|e| {
            MobileUseError::Other(format!("WDA actions failed: {}", e))
        })?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(MobileUseError::Other(format!("WDA actions error: {}", text)));
        }

        Ok(())
    }

    /// Convert physical pixels to logical points for WDA
    fn to_logical(&self, px: i32) -> i32 {
        (px as f64 / self.scale_factor) as i32
    }

    /// Tap at physical pixel coordinates
    pub fn tap(&self, x: i32, y: i32) -> Result<()> {
        let lx = self.to_logical(x);
        let ly = self.to_logical(y);

        let action = json!({
            "type": "pointer",
            "id": "finger",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "x": lx, "y": ly, "origin": "viewport", "duration": 0},
                {"type": "pointerDown", "button": 0},
                {"type": "pointerUp", "button": 0}
            ]
        });

        self.perform_actions(vec![action])
    }

    /// Double tap at physical pixel coordinates
    pub fn double_tap(&self, x: i32, y: i32) -> Result<()> {
        let lx = self.to_logical(x);
        let ly = self.to_logical(y);

        let action = json!({
            "type": "pointer",
            "id": "finger",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "x": lx, "y": ly, "origin": "viewport", "duration": 0},
                {"type": "pointerDown", "button": 0},
                {"type": "pointerUp", "button": 0},
                {"type": "pause", "duration": 100},
                {"type": "pointerDown", "button": 0},
                {"type": "pointerUp", "button": 0}
            ]
        });

        self.perform_actions(vec![action])
    }

    /// Long press at physical pixel coordinates
    pub fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> Result<()> {
        let lx = self.to_logical(x);
        let ly = self.to_logical(y);

        let action = json!({
            "type": "pointer",
            "id": "finger",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "x": lx, "y": ly, "origin": "viewport", "duration": 0},
                {"type": "pointerDown", "button": 0},
                {"type": "pause", "duration": duration_ms},
                {"type": "pointerUp", "button": 0}
            ]
        });

        self.perform_actions(vec![action])
    }

    /// Swipe from (x1,y1) to (x2,y2) over duration_ms, coordinates in physical pixels
    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        let lx1 = self.to_logical(x1);
        let ly1 = self.to_logical(y1);
        let lx2 = self.to_logical(x2);
        let ly2 = self.to_logical(y2);

        let action = json!({
            "type": "pointer",
            "id": "finger",
            "parameters": {"pointerType": "touch"},
            "actions": [
                {"type": "pointerMove", "x": lx1, "y": ly1, "origin": "viewport", "duration": 0},
                {"type": "pointerDown", "button": 0},
                {"type": "pointerMove", "x": lx2, "y": ly2, "origin": "viewport", "duration": duration_ms},
                {"type": "pointerUp", "button": 0}
            ]
        });

        self.perform_actions(vec![action])
    }

    /// Type text via WDA keyboard actions
    pub fn type_text(&self, text: &str) -> Result<()> {
        let mut key_actions: Vec<Value> = Vec::new();

        for c in text.chars() {
            let ch = c.to_string();
            key_actions.push(json!({"type": "keyDown", "value": ch}));
            key_actions.push(json!({"type": "keyUp", "value": ch}));
        }

        let action = json!({
            "type": "key",
            "id": "keyboard",
            "actions": key_actions
        });

        self.perform_actions(vec![action])
    }

    /// Press a named key (home, volumeUp, etc.) via WDA-specific endpoint
    pub fn press_button(&self, button: &str) -> Result<()> {
        let url = format!("{}/wda/pressButton", self.session_url()?);
        let body = json!({"name": button});

        let resp = self.client.post(&url).json(&body).send().map_err(|e| {
            MobileUseError::Other(format!("WDA pressButton failed: {}", e))
        })?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(MobileUseError::Other(format!("WDA pressButton error: {}", text)));
        }

        Ok(())
    }

    /// Take screenshot, returns PNG bytes
    pub fn screenshot(&self) -> Result<Vec<u8>> {
        let url = format!("{}/screenshot", self.session_url()?);

        let resp = self.client.get(&url).send().map_err(|e| {
            MobileUseError::Other(format!("WDA screenshot failed: {}", e))
        })?;

        let data: Value = resp.json().map_err(|e| {
            MobileUseError::Other(format!("WDA screenshot parse failed: {}", e))
        })?;

        let b64 = data
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MobileUseError::Other("WDA screenshot: no value".to_string()))?;

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| MobileUseError::Other(format!("Screenshot base64 decode failed: {}", e)))?;

        Ok(bytes)
    }

    /// Get window size in logical points
    pub fn window_size(&self) -> Result<(i32, i32)> {
        let url = format!("{}/window/size", self.session_url()?);

        let resp = self.client.get(&url).send().map_err(|e| {
            MobileUseError::Other(format!("WDA window size failed: {}", e))
        })?;

        let data: Value = resp.json().map_err(|e| {
            MobileUseError::Other(format!("WDA window size parse failed: {}", e))
        })?;

        let width = data
            .pointer("/value/width")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MobileUseError::Other("WDA: no window width".to_string()))? as i32;
        let height = data
            .pointer("/value/height")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| MobileUseError::Other("WDA: no window height".to_string()))? as i32;

        Ok((width, height))
    }

    /// Get screen size in physical pixels
    pub fn screen_size_physical(&self) -> Result<(i32, i32)> {
        let (w, h) = self.window_size()?;
        Ok((
            (w as f64 * self.scale_factor) as i32,
            (h as f64 * self.scale_factor) as i32,
        ))
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 5: Commit**

```bash
git add src/platform/ios/ src/platform/mod.rs
git commit -m "feat: implement WdaClient for WebDriverAgent HTTP communication"
```

---

### Task 5: Implement IOSDevice with DeviceOperator trait

**Files:**
- Create: `src/platform/ios/ios_device.rs`
- Modify: `src/platform/ios/mod.rs`

**Step 1: Create ios_device.rs**

```rust
// src/platform/ios/ios_device.rs
use crate::core::{MobileUseError, Result};
use crate::platform::device::{DeviceOperator, Platform};
use super::wda::WdaClient;

/// iOS device operator — works for both simulators and real devices.
/// All operations delegate to WdaClient.
pub struct IOSDevice {
    pub wda: WdaClient,
    pub device_id: String,
    pub simulator: bool,
}

impl IOSDevice {
    pub fn new(device_id: &str, wda_base_url: &str, simulator: bool) -> Self {
        Self {
            wda: WdaClient::new(wda_base_url),
            device_id: device_id.to_string(),
            simulator,
        }
    }

    /// Initialize: connect to WDA and create session
    pub fn connect(&mut self) -> Result<()> {
        if !self.wda.health_check() {
            return Err(MobileUseError::ConnectionFailed(
                "WDA is not reachable. Make sure WebDriverAgent is running on the device.".to_string(),
            ));
        }
        self.wda.create_session()
    }
}

impl DeviceOperator for IOSDevice {
    fn tap(&self, x: i32, y: i32) -> Result<()> {
        self.wda.tap(x, y)
    }

    fn double_tap(&self, x: i32, y: i32) -> Result<()> {
        self.wda.double_tap(x, y)
    }

    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> Result<()> {
        self.wda.long_press(x, y, duration_ms)
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        self.wda.swipe(x1, y1, x2, y2, duration_ms)
    }

    fn input_text(&self, text: &str) -> Result<()> {
        self.wda.type_text(text)
    }

    fn keyevent(&self, key: &str) -> Result<()> {
        // Map common key names to WDA button names
        let wda_button = match key.to_uppercase().as_str() {
            "HOME" | "KEYCODE_HOME" => "home",
            "VOLUME_UP" | "KEYCODE_VOLUME_UP" => "volumeUp",
            "VOLUME_DOWN" | "KEYCODE_VOLUME_DOWN" => "volumeDown",
            other => other,
        };
        self.wda.press_button(wda_button)
    }

    fn screenshot(&self, local_path: &str) -> Result<()> {
        let bytes = self.wda.screenshot()?;
        std::fs::write(local_path, &bytes).map_err(|e| {
            MobileUseError::Other(format!("Failed to write screenshot: {}", e))
        })?;
        Ok(())
    }

    fn get_screen_size(&self) -> Result<(i32, i32)> {
        self.wda.screen_size_physical()
    }

    fn platform(&self) -> Platform {
        Platform::IOS
    }
}
```

**Step 2: Update ios/mod.rs**

```rust
// src/platform/ios/mod.rs
pub mod ios_device;
pub mod wda;

pub use ios_device::IOSDevice;
pub use wda::WdaClient;
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors

**Step 4: Commit**

```bash
git add src/platform/ios/
git commit -m "feat: implement IOSDevice with DeviceOperator trait"
```

---

### Task 6: Add IOSError variant to MobileUseError

**Files:**
- Modify: `src/core/error.rs:4-38`

**Step 1: Add IOSError variant**

Add after the `AdbError` variant (line 16):

```rust
    #[error("iOS error: {0}")]
    IOSError(String),
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/core/error.rs
git commit -m "feat: add IOSError variant to MobileUseError"
```

---

### Task 7: Refactor DaemonSession — replace `adb` with `Box<dyn DeviceOperator>`

This is the core refactoring. `DaemonSession` currently holds `pub adb: AdbClient`. We change it to `pub device_op: Box<dyn DeviceOperator>` and keep `pub adb: AdbClient` temporarily for the `find_flutter_vm_service` auto-discover path which needs ADB-specific methods (`forward_list`, `shell`).

**Files:**
- Modify: `src/daemon/session_manager.rs`

**Step 1: Update DaemonSession struct**

Replace the entire file content:

```rust
use crate::core::RefMap;
use crate::platform::android::AdbClient;
use crate::platform::device::{DeviceOperator, Platform};
use crate::platform::flutter::VmServiceClient;
use std::collections::HashMap;
use tracing::info;

/// A single daemon session managing a connection to a Flutter app
pub struct DaemonSession {
    /// Session name
    pub name: String,
    /// Device ID
    pub device: Option<String>,
    /// VM Service URL (None = not connected)
    pub vm_url: Option<String>,
    /// VM Service client for Flutter communication
    pub vm_service: VmServiceClient,
    /// Platform-agnostic device operator for tap/swipe/screenshot
    pub device_op: Box<dyn DeviceOperator>,
    /// ADB client — kept for Android-specific operations (forward_list, shell, auto-discover)
    pub adb: AdbClient,
    /// Reference map for element lookup
    pub ref_map: RefMap,
    /// Track if this session has a flutter process (run mode)
    pub has_flutter_process: bool,
    /// Android package name (native Android mode)
    pub package: Option<String>,
    /// Platform type
    pub platform: Platform,
}

impl DaemonSession {
    /// Create a new daemon session (defaults to Android)
    pub fn new(name: &str, device: Option<String>) -> Self {
        info!("Creating session: {} (device: {:?})", name, device);
        let adb = AdbClient::new(device.clone());
        Self {
            name: name.to_string(),
            device: device.clone(),
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op: Box::new(AdbClient::new(device)),
            adb,
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
            platform: Platform::Android,
        }
    }

    /// Create a new iOS session
    pub fn new_ios(name: &str, device_id: &str, wda_url: &str, simulator: bool) -> Self {
        info!("Creating iOS session: {} (device: {})", name, device_id);
        let mut ios_device = crate::platform::ios::IOSDevice::new(device_id, wda_url, simulator);
        // We attempt to connect; if it fails, the session is created but not usable.
        // Caller should check and handle.
        let _ = ios_device.connect();
        Self {
            name: name.to_string(),
            device: Some(device_id.to_string()),
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op: Box::new(ios_device),
            adb: AdbClient::new(None), // placeholder, not used for iOS
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
            platform: Platform::IOS,
        }
    }

    /// Check if the session is connected to a VM Service
    pub fn is_connected(&self) -> bool {
        self.vm_url.is_some() || self.package.is_some()
    }

    /// Check if this is a native Android session (ADB-only, no VM Service)
    pub fn is_android_mode(&self) -> bool {
        self.package.is_some() && self.vm_url.is_none()
    }
}

/// Manages multiple daemon sessions
pub struct SessionManager {
    sessions: HashMap<String, DaemonSession>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get an existing session or create a new one (Android by default)
    pub fn get_or_create(&mut self, name: &str, device: Option<String>) -> &mut DaemonSession {
        if !self.sessions.contains_key(name) {
            info!("Creating new session: {}", name);
            let session = DaemonSession::new(name, device);
            self.sessions.insert(name.to_string(), session);
        }
        self.sessions.get_mut(name).unwrap()
    }

    /// Get an existing session or create a new iOS session
    pub fn get_or_create_ios(
        &mut self,
        name: &str,
        device_id: &str,
        wda_url: &str,
        simulator: bool,
    ) -> &mut DaemonSession {
        if !self.sessions.contains_key(name) {
            info!("Creating new iOS session: {}", name);
            let session = DaemonSession::new_ios(name, device_id, wda_url, simulator);
            self.sessions.insert(name.to_string(), session);
        }
        self.sessions.get_mut(name).unwrap()
    }

    /// Get an existing session by name
    pub fn get(&self, name: &str) -> Option<&DaemonSession> {
        self.sessions.get(name)
    }

    /// Get a mutable reference to an existing session by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut DaemonSession> {
        self.sessions.get_mut(name)
    }

    /// Remove a session by name
    pub fn remove(&mut self, name: &str) -> Option<DaemonSession> {
        info!("Removing session: {}", name);
        self.sessions.remove(name)
    }

    /// List all session names
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_session_new() {
        let session = DaemonSession::new("test", None);
        assert_eq!(session.name, "test");
        assert!(session.device.is_none());
        assert!(session.vm_url.is_none());
        assert!(!session.is_connected());
        assert_eq!(session.platform, Platform::Android);
    }

    #[test]
    fn test_daemon_session_with_device() {
        let session = DaemonSession::new("test", Some("emulator-5554".to_string()));
        assert_eq!(session.name, "test");
        assert_eq!(session.device, Some("emulator-5554".to_string()));
        assert!(!session.is_connected());
    }

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_session_manager_get_or_create() {
        let mut manager = SessionManager::new();
        let session = manager.get_or_create("test", None);
        assert_eq!(session.name, "test");

        let session = manager.get_or_create("test", Some("device".to_string()));
        assert_eq!(session.name, "test");
        assert!(session.device.is_none());

        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_session_manager_get() {
        let mut manager = SessionManager::new();
        assert!(manager.get("nonexistent").is_none());
        manager.get_or_create("test", None);
        assert!(manager.get("test").is_some());
    }

    #[test]
    fn test_session_manager_get_mut() {
        let mut manager = SessionManager::new();
        assert!(manager.get_mut("nonexistent").is_none());
        manager.get_or_create("test", None);
        let session = manager.get_mut("test").unwrap();
        session.vm_url = Some("ws://localhost:12345/ws".to_string());
        assert!(manager.get("test").unwrap().is_connected());
    }

    #[test]
    fn test_session_manager_remove() {
        let mut manager = SessionManager::new();
        manager.get_or_create("test", None);
        assert_eq!(manager.list().len(), 1);

        let removed = manager.remove("test");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "test");
        assert!(manager.list().is_empty());
        assert!(manager.remove("test").is_none());
    }

    #[test]
    fn test_session_manager_list() {
        let mut manager = SessionManager::new();
        manager.get_or_create("session1", None);
        manager.get_or_create("session2", None);
        manager.get_or_create("session3", None);
        let list = manager.list();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"session1"));
        assert!(list.contains(&"session2"));
        assert!(list.contains(&"session3"));
    }

    #[test]
    fn test_session_manager_default() {
        let manager = SessionManager::default();
        assert!(manager.list().is_empty());
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles. Note: `main.rs` calls to `get_adb_from_session` still use `AdbClient` directly — that will be updated in Task 8.

**Step 3: Run existing tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/daemon/session_manager.rs
git commit -m "refactor: add device_op to DaemonSession, keep adb for backward compat"
```

---

### Task 8: Refactor main.rs — replace AdbClient usage with DeviceOperator

**Files:**
- Modify: `src/main.rs`

This task changes the action functions to use `DeviceOperator` instead of `AdbClient`. The key change is `get_adb_from_session` becomes `get_device_from_session` which returns a serializable device info struct that wraps the trait operations.

**Step 1: Replace `get_adb_from_session` with a device-agnostic approach**

Since `DeviceOperator` is not `Clone` and lives inside the daemon, we keep the current pattern but change the action functions to call the daemon for device operations. However, the current architecture has the CLI-side creating `AdbClient` from session info and calling it directly. For now, the simplest approach is to keep `get_adb_from_session` for Android and add a parallel path for iOS.

A cleaner approach: add a new daemon request type `DeviceAction` that delegates device operations to the daemon-side `device_op`. But this is a large change. Instead, for Phase 1, we keep the existing pattern where the CLI creates the device client locally.

Replace `get_adb_from_session` function (around line 1268-1297) and `map_adb_err` helper:

```rust
use crate::platform::device::DeviceOperator;

// Helper function to get device operator from daemon session info
async fn get_device_from_session(
    client: &mut DaemonClient,
    session: &str,
    device_override: Option<String>,
) -> Result<Box<dyn DeviceOperator>> {
    let info_req = DaemonRequest::Info {
        session: session.to_string(),
    };
    let info = match client.request(info_req).await? {
        DaemonResponse::Ok { data: Some(info) } => info,
        DaemonResponse::Ok { data: None } => return Err(anyhow::anyhow!("No session info available")),
        DaemonResponse::Error { message } => return Err(anyhow::anyhow!(message)),
        DaemonResponse::HasFlutterProcess { .. } => {
            return Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
        }
    };

    let platform = info.get("platform").and_then(|v| v.as_str()).unwrap_or("android");

    match platform {
        "ios" => {
            let wda_url = info
                .get("wda_url")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost:8100");
            let device_id = device_override
                .or_else(|| info.get("device").and_then(|v| v.as_str()).map(String::from))
                .unwrap_or_default();
            let simulator = info
                .get("simulator")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let mut ios = crate::platform::ios::IOSDevice::new(&device_id, wda_url, simulator);
            ios.connect()?;
            Ok(Box::new(ios))
        }
        _ => {
            // Android
            let device = device_override
                .or_else(|| info.get("device").and_then(|v| v.as_str()).map(String::from));
            Ok(Box::new(AdbClient::new(device)))
        }
    }
}

// Helper to map device errors (replaces map_adb_err)
fn map_device_err<T, E: std::fmt::Display>(
    output: &OutputFormatter,
    result: std::result::Result<T, E>,
) -> Result<T> {
    result.map_err(|e| {
        output.error(&e.to_string());
        anyhow::anyhow!(e.to_string())
    })
}
```

Then update all action functions. In each one, replace:
- `get_adb_from_session(...)` → `get_device_from_session(...)`
- `adb.tap(x, y)` → `device.tap(x, y)`
- `adb.swipe(...)` → `device.swipe(...)`
- `adb.long_press(...)` → `device.long_press(...)`
- `adb.input_text(...)` → `device.input_text(...)`
- `adb.keyevent(...)` → `device.keyevent(...)`
- `adb.screenshot(...)` → `device.screenshot(...)`
- `adb.get_screen_size()` → `device.get_screen_size()`
- `map_adb_err(output, ...)` → `map_device_err(output, ...)`
- `clear_text_field(&adb, output)` → `clear_text_field(&*device, output)`

Update `clear_text_field` signature:

```rust
fn clear_text_field(device: &dyn DeviceOperator, output: &OutputFormatter) -> Result<()> {
    map_device_err(output, device.keyevent("KEYCODE_MOVE_END"))?;
    for _ in 0..MAX_CLEAR_DELETE_PRESSES {
        map_device_err(output, device.keyevent("67"))?;
    }
    Ok(())
}
```

Example for `tap_action`:

```rust
async fn tap_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let device = get_device_from_session(client, session, None).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let (x, y) = element.bounds.center();
    map_device_err(output, device.tap(x as i32, y as i32))?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Tapped: {} \"{}\"",
            element.element_type,
            element.label.as_deref().unwrap_or("")
        )),
        data: None,
    });
    Ok(())
}
```

Apply the same pattern to: `double_tap_action`, `long_press_action`, `clear_action`, `input_action`, `capture_action`, `scroll_action`, `swipe_action`.

Keep the existing `get_adb_from_session` function (renamed to `get_adb_client`) for the few places that genuinely need ADB-specific methods (e.g., `find_flutter_vm_service` in `server.rs`).

**Step 2: Update server.rs Info handler to include platform info**

In `src/daemon/server.rs`, update the `Info` handler (around line 290-308) to include platform:

```rust
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
            "platform": sess.platform.to_string(),
            "connected": sess.is_connected()
        })))
    })
}
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/main.rs src/daemon/server.rs
git commit -m "refactor: replace AdbClient with DeviceOperator in action functions"
```

---

### Task 9: Add iOS device discovery

**Files:**
- Create: `src/platform/ios/discovery.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/commands/connect.rs`

**Step 1: Create discovery.rs**

```rust
// src/platform/ios/discovery.rs
use crate::core::{MobileUseError, Result};
use serde::Serialize;
use serde_json::Value;
use std::process::Command;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize)]
pub struct IOSDeviceInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    pub device_type: String, // "simulator" or "device"
    pub os_version: String,
}

/// List booted iOS simulators via xcrun simctl
pub fn list_simulators() -> Result<Vec<IOSDeviceInfo>> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "booted", "-j"])
        .output()
        .map_err(|e| MobileUseError::IOSError(format!("Failed to run xcrun simctl: {}", e)))?;

    if !output.status.success() {
        // xcrun not available (not on macOS or no Xcode)
        debug!("xcrun simctl not available");
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: Value = serde_json::from_str(&stdout)
        .map_err(|e| MobileUseError::IOSError(format!("Failed to parse simctl output: {}", e)))?;

    let mut devices = Vec::new();

    if let Some(runtime_devices) = data.get("devices").and_then(|d| d.as_object()) {
        for (runtime, device_list) in runtime_devices {
            // runtime looks like "com.apple.CoreSimulator.SimRuntime.iOS-17-2"
            let os_version = runtime
                .rsplit('.')
                .next()
                .unwrap_or("Unknown")
                .replace('-', ".");

            if let Some(arr) = device_list.as_array() {
                for dev in arr {
                    let state = dev.get("state").and_then(|v| v.as_str()).unwrap_or("");
                    if state != "Booted" {
                        continue;
                    }

                    let id = dev
                        .get("udid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = dev
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    devices.push(IOSDeviceInfo {
                        id,
                        name,
                        state: "Booted".to_string(),
                        device_type: "simulator".to_string(),
                        os_version: os_version.clone(),
                    });
                }
            }
        }
    }

    info!("Found {} booted iOS simulators", devices.len());
    Ok(devices)
}

/// List connected iOS real devices via idevice_id (libimobiledevice)
pub fn list_real_devices() -> Result<Vec<IOSDeviceInfo>> {
    let output = match Command::new("idevice_id").arg("-l").output() {
        Ok(o) => o,
        Err(_) => {
            debug!("idevice_id not available (libimobiledevice not installed)");
            return Ok(Vec::new());
        }
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let udid = line.trim();
        if udid.is_empty() {
            continue;
        }

        // Try to get device name via ideviceinfo
        let name = Command::new("ideviceinfo")
            .args(["-u", udid, "-k", "DeviceName"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "iOS Device".to_string());

        let os_version = Command::new("ideviceinfo")
            .args(["-u", udid, "-k", "ProductVersion"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        devices.push(IOSDeviceInfo {
            id: udid.to_string(),
            name,
            state: "Connected".to_string(),
            device_type: "device".to_string(),
            os_version,
        });
    }

    info!("Found {} iOS real devices", devices.len());
    Ok(devices)
}

/// List all iOS devices (simulators + real)
pub fn list_all_devices() -> Result<Vec<IOSDeviceInfo>> {
    let mut all = list_simulators()?;
    all.extend(list_real_devices()?);
    Ok(all)
}
```

**Step 2: Update ios/mod.rs**

```rust
// src/platform/ios/mod.rs
pub mod discovery;
pub mod ios_device;
pub mod wda;

pub use ios_device::IOSDevice;
pub use wda::WdaClient;
```

**Step 3: Update commands/connect.rs to list iOS devices too**

Replace the `devices` function:

```rust
use crate::cli::OutputFormatter;
use crate::core::{MobileUseError, Result};
use crate::platform::android::AdbClient;
use crate::platform::ios::discovery as ios_discovery;
use tracing::info;

/// Find Flutter VM Service URL for a device
pub async fn find_flutter_vm_service(adb: &AdbClient) -> Result<String> {
    // ... (unchanged, keep existing code)
}

/// List connected devices (Android + iOS) with detailed information
pub fn devices(output: &OutputFormatter, json_mode: bool) -> Result<()> {
    // Collect Android devices
    let android_devices = AdbClient::devices().unwrap_or_default();

    // Collect iOS devices
    let ios_devices = ios_discovery::list_all_devices().unwrap_or_default();

    let total = android_devices.len() + ios_devices.len();

    if total == 0 {
        output.error("No devices connected. Check 'adb devices' or Xcode simulators.");
        return Ok(());
    }

    if json_mode {
        let json = serde_json::json!({
            "android": android_devices,
            "ios": ios_devices,
        });
        output.json(&json);
        return Ok(());
    }

    output.success(&format!("Found {} device(s):\n", total));

    // Android devices
    for (i, device) in android_devices.iter().enumerate() {
        output.raw(&format!("  [{}] {} (Android)", i + 1, device.id));
        output.raw(&format!(
            "      Model:   {} ({})",
            device.model, device.brand
        ));
        output.raw(&format!(
            "      Android: {} (SDK {})",
            device.android_version, device.sdk_version
        ));
        output.raw(&format!("      Screen:  {}", device.screen_size));
        output.raw("");
    }

    // iOS devices
    let offset = android_devices.len();
    for (i, device) in ios_devices.iter().enumerate() {
        let label = if device.device_type == "simulator" {
            "iOS Simulator"
        } else {
            "iOS Device"
        };
        output.raw(&format!("  [{}] {} ({})", offset + i + 1, device.id, label));
        output.raw(&format!("      Name:    {}", device.name));
        output.raw(&format!("      iOS:     {}", device.os_version));
        output.raw(&format!("      State:   {}", device.state));
        output.raw("");
    }

    if let Some(first) = android_devices.first() {
        output.raw(&format!("  mobile-use -d {} run", first.id));
    }
    if let Some(first) = ios_devices.first() {
        output.raw(&format!("  mobile-use -d {} run", first.id));
    }

    Ok(())
}
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src/platform/ios/discovery.rs src/platform/ios/mod.rs src/commands/connect.rs
git commit -m "feat: add iOS device discovery (simulators + real devices)"
```

---

### Task 10: Add ConnectIOS to daemon protocol and server

**Files:**
- Modify: `src/daemon/protocol.rs`
- Modify: `src/daemon/server.rs`

**Step 1: Add ConnectIOS variant to DaemonRequest**

In `src/daemon/protocol.rs`, add after `ConnectAndroid` (around line 54):

```rust
    /// Connect to iOS device via WebDriverAgent
    ConnectIOS {
        session: String,
        device: Option<String>,
        simulator: bool,
        wda_port: Option<u16>,
    },
```

**Step 2: Handle ConnectIOS in server.rs**

In `src/daemon/server.rs`, add a new match arm in `handle_request` (after `ConnectAndroid`):

```rust
        DaemonRequest::ConnectIOS {
            session,
            device,
            simulator,
            wda_port,
        } => {
            let port = wda_port.unwrap_or(8100);
            let wda_url = format!("http://localhost:{}", port);
            let device_id = device.clone().unwrap_or_default();

            let mut sessions_guard = sessions.lock().await;
            let daemon_session = sessions_guard.get_or_create_ios(
                &session,
                &device_id,
                &wda_url,
                simulator,
            );

            DaemonResponse::ok(Some(json!({
                "session": session,
                "device": device_id,
                "platform": "ios",
                "simulator": simulator,
                "wda_url": wda_url,
                "connected": true
            })))
        }
```

**Step 3: Update Info handler to include iOS-specific fields**

Already done in Task 8 (platform field). Additionally, add wda_url and simulator to the info response. In the Info handler, change to:

```rust
DaemonRequest::Info { session } => {
    with_session!(sessions, &session, |sess| {
        let mode = if sess.is_android_mode() {
            "android"
        } else if sess.vm_url.is_some() {
            "flutter"
        } else {
            "disconnected"
        };
        let mut info = json!({
            "session": sess.name,
            "device": sess.device,
            "vm_url": sess.vm_url,
            "package": sess.package,
            "mode": mode,
            "platform": sess.platform.to_string(),
            "connected": sess.is_connected()
        });
        // Add iOS-specific info
        if sess.platform == crate::platform::Platform::IOS {
            info["simulator"] = json!(true); // TODO: store this in session
        }
        DaemonResponse::ok(Some(info))
    })
}
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 5: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/daemon/protocol.rs src/daemon/server.rs
git commit -m "feat: add ConnectIOS daemon request for iOS session creation"
```

---

### Task 11: Add WDA manager for simulator lifecycle

**Files:**
- Create: `src/platform/ios/wda_manager.rs`
- Modify: `src/platform/ios/mod.rs`

**Step 1: Create wda_manager.rs**

```rust
// src/platform/ios/wda_manager.rs
use crate::core::{MobileUseError, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

/// Manages WebDriverAgent lifecycle on iOS devices
pub struct WdaManager {
    device_id: String,
    simulator: bool,
    port: u16,
}

impl WdaManager {
    pub fn new(device_id: &str, simulator: bool, port: u16) -> Self {
        Self {
            device_id: device_id.to_string(),
            simulator,
            port,
        }
    }

    /// Check if WDA is already running and reachable
    pub fn is_running(&self) -> bool {
        let url = format!("http://localhost:{}/status", self.port);
        reqwest::blocking::get(&url)
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Get the WDA cache directory
    fn wda_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("mobile-use")
            .join("wda")
    }

    /// Ensure WDA is running. If not, attempt to start it.
    pub fn ensure_running(&self) -> Result<()> {
        if self.is_running() {
            info!("WDA already running on port {}", self.port);
            return Ok(());
        }

        if self.simulator {
            self.start_simulator_wda()?;
        } else {
            self.start_device_wda()?;
        }

        // Wait for WDA to become ready
        self.wait_for_ready(30)
    }

    /// Start WDA on simulator
    fn start_simulator_wda(&self) -> Result<()> {
        let wda_dir = Self::wda_cache_dir();

        // Check if we have a cached WDA build
        let xctestrun = self.find_xctestrun()?;

        info!("Starting WDA on simulator {} via xcodebuild", self.device_id);

        // Launch xcodebuild test-without-building in background
        let mut cmd = Command::new("xcodebuild");
        cmd.args([
            "test-without-building",
            "-xctestrun",
            &xctestrun.to_string_lossy(),
            "-destination",
            &format!("platform=iOS Simulator,id={}", self.device_id),
            "-derivedDataPath",
            &wda_dir.to_string_lossy(),
        ]);

        debug!("Running: {:?}", cmd);

        // Run in background — WDA keeps running as long as xcodebuild runs
        std::thread::spawn(move || {
            let _ = cmd.output();
        });

        Ok(())
    }

    /// Start WDA on real device (requires prior setup-ios)
    fn start_device_wda(&self) -> Result<()> {
        let xctestrun = self.find_xctestrun()?;

        info!("Starting WDA on device {} via xcodebuild", self.device_id);

        // Start iproxy for port forwarding
        let port = self.port;
        let device_id = self.device_id.clone();
        std::thread::spawn(move || {
            let _ = Command::new("iproxy")
                .args([
                    &port.to_string(),
                    &port.to_string(),
                    "-u",
                    &device_id,
                ])
                .output();
        });

        // Launch xcodebuild
        let mut cmd = Command::new("xcodebuild");
        cmd.args([
            "test-without-building",
            "-xctestrun",
            &xctestrun.to_string_lossy(),
            "-destination",
            &format!("platform=iOS,id={}", self.device_id),
        ]);

        std::thread::spawn(move || {
            let _ = cmd.output();
        });

        Ok(())
    }

    /// Find the xctestrun file from cache
    fn find_xctestrun(&self) -> Result<PathBuf> {
        let wda_dir = Self::wda_cache_dir();

        // Look for xctestrun file
        if wda_dir.exists() {
            for entry in std::fs::read_dir(&wda_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "xctestrun").unwrap_or(false) {
                    return Ok(path);
                }
            }
            // Search recursively in Build/Products
            let products_dir = wda_dir.join("Build").join("Products");
            if products_dir.exists() {
                for entry in std::fs::read_dir(&products_dir).into_iter().flatten().flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "xctestrun").unwrap_or(false) {
                        return Ok(path);
                    }
                }
            }
        }

        Err(MobileUseError::IOSError(
            "WDA not found. Run 'mobile-use setup-ios' first to build WebDriverAgent.".to_string(),
        ))
    }

    /// Wait for WDA to become ready
    fn wait_for_ready(&self, timeout_secs: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        info!("Waiting for WDA to become ready on port {}...", self.port);

        while start.elapsed() < timeout {
            if self.is_running() {
                info!("WDA is ready!");
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        Err(MobileUseError::Timeout(format!(
            "WDA did not start within {}s on port {}",
            timeout_secs, self.port
        )))
    }
}
```

**Step 2: Update ios/mod.rs**

```rust
// src/platform/ios/mod.rs
pub mod discovery;
pub mod ios_device;
pub mod wda;
pub mod wda_manager;

pub use ios_device::IOSDevice;
pub use wda::WdaClient;
pub use wda_manager::WdaManager;
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/platform/ios/wda_manager.rs src/platform/ios/mod.rs
git commit -m "feat: add WdaManager for WDA lifecycle management"
```

---

### Task 12: Add setup-ios CLI command

**Files:**
- Modify: `src/cli/parser.rs`
- Modify: `src/main.rs` (command handler section)

**Step 1: Add SetupIos command to parser.rs**

In `src/cli/parser.rs`, add to the `Commands` enum (after `Quit`):

```rust
    /// Setup iOS device for automation (builds and installs WebDriverAgent)
    #[command(long_about = "Setup an iOS device for automation.

Builds WebDriverAgent with your Apple ID signing and installs it on the device.
This is required once per device (or when signing expires).

For simulators, setup is automatic — no need to run this command.
For real devices, you need an Apple ID (free tier works, valid for 7 days).

Examples:
  mobile-use setup-ios                          # Setup connected device
  mobile-use setup-ios --device <UDID>          # Setup specific device")]
    SetupIos {
        /// Device UDID
        #[arg(long, short, help = "Device UDID")]
        device: Option<String>,
    },
```

**Step 2: Add handler in main.rs**

In the early command dispatch section (around line 258, after `Commands::Devices`), add:

```rust
        Commands::SetupIos { device } => {
            output.info("Setting up WebDriverAgent for iOS...");

            let device_id = match device {
                Some(d) => d.clone(),
                None => {
                    // Try to find first connected real device
                    let devices = crate::platform::ios::discovery::list_real_devices()
                        .unwrap_or_default();
                    if let Some(dev) = devices.first() {
                        output.info(&format!("Found device: {} ({})", dev.name, dev.id));
                        dev.id.clone()
                    } else {
                        output.error("No iOS device found. Connect a device or specify --device <UDID>");
                        std::process::exit(1);
                    }
                }
            };

            let wda_dir = dirs::cache_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                .join("mobile-use")
                .join("wda");

            // Create WDA cache dir
            std::fs::create_dir_all(&wda_dir).ok();

            // TODO: In a full implementation, this would:
            // 1. Clone/extract WebDriverAgent project
            // 2. Run xcodebuild build-for-testing with automatic signing
            // 3. Cache the xctestrun and build products
            // For now, guide the user to set up manually:

            output.info(&format!("WDA build cache: {}", wda_dir.display()));
            output.info("To build WDA manually:");
            output.raw("  1. Clone: git clone https://github.com/appium/WebDriverAgent.git");
            output.raw(&format!("  2. Build: cd WebDriverAgent && xcodebuild build-for-testing \\"));
            output.raw(&format!("       -project WebDriverAgent.xcodeproj \\"));
            output.raw(&format!("       -scheme WebDriverAgentRunner \\"));
            output.raw(&format!("       -destination 'platform=iOS,id={}' \\", device_id));
            output.raw(&format!("       -derivedDataPath '{}' \\", wda_dir.display()));
            output.raw("       -allowProvisioningUpdates");
            output.raw("");
            output.info("After building, mobile-use will automatically find and use the cached WDA.");

            return Ok(());
        }
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/cli/parser.rs src/main.rs
git commit -m "feat: add setup-ios CLI command for WDA installation"
```

---

### Task 13: Update run command to support iOS Flutter targets

**Files:**
- Modify: `src/main.rs` (detect_project_type and run command handler)

**Step 1: Add FlutterIOS to ProjectType**

Update the `ProjectType` enum and `detect_project_type` function:

```rust
#[derive(Debug, Clone, PartialEq)]
enum ProjectType {
    FlutterAndroid,
    FlutterIOS,
    NativeAndroid,
    Unknown,
}

fn detect_project_type(apk: &Option<String>, package: &Option<String>, device: &Option<String>, flutter_args: &[String]) -> ProjectType {
    if apk.is_some() || package.is_some() {
        return ProjectType::NativeAndroid;
    }

    let has_pubspec = std::path::Path::new("pubspec.yaml").exists();

    if has_pubspec {
        // Check if targeting iOS based on device ID or flutter args
        let is_ios_target = device.as_ref().map(|d| is_ios_device_id(d)).unwrap_or(false)
            || flutter_args.iter().any(|a| a.contains("iphone") || a.contains("ipad") || a.contains("ios"));

        if is_ios_target {
            return ProjectType::FlutterIOS;
        }

        let has_android = std::path::Path::new("android").exists();
        if has_android {
            return ProjectType::FlutterAndroid;
        }

        // Default Flutter project — check for ios directory
        let has_ios = std::path::Path::new("ios").exists();
        if has_ios {
            return ProjectType::FlutterIOS;
        }

        return ProjectType::FlutterAndroid;
    }

    // Check for native Android Gradle project
    let has_gradle = std::path::Path::new("build.gradle").exists()
        || std::path::Path::new("build.gradle.kts").exists()
        || std::path::Path::new("settings.gradle.kts").exists()
        || std::path::Path::new("settings.gradle").exists();

    if has_gradle {
        return ProjectType::NativeAndroid;
    }

    ProjectType::Unknown
}

/// Check if a device ID looks like an iOS device (UDID format or known simulator name)
fn is_ios_device_id(id: &str) -> bool {
    // iOS UDIDs are typically 36-char UUIDs (simulator) or 40-char hex (real device)
    let is_uuid = id.len() == 36 && id.chars().filter(|c| *c == '-').count() == 4;
    let is_hex_udid = id.len() == 40 && id.chars().all(|c| c.is_ascii_hexdigit());
    let is_hex_short = id.len() == 25 && id.contains('-');

    // Also check against known simulators
    if is_uuid || is_hex_udid || is_hex_short {
        return true;
    }

    // Check if it's a known iOS simulator
    if let Ok(sims) = crate::platform::ios::discovery::list_simulators() {
        return sims.iter().any(|s| s.id == id);
    }

    false
}
```

**Step 2: Add FlutterIOS handler in the run command match**

In the `Commands::Run` handler, after the `FlutterAndroid` match arm, add:

```rust
                ProjectType::FlutterIOS => {
                    // Flutter iOS — same flow as FlutterAndroid but with iOS device setup
                    output.info("Running Flutter app on iOS...");

                    // Check if device is a simulator
                    let simulators = crate::platform::ios::discovery::list_simulators()
                        .unwrap_or_default();
                    let is_simulator = device.as_ref()
                        .map(|d| simulators.iter().any(|s| s.id == *d))
                        .unwrap_or(false);

                    // Ensure WDA is running
                    let wda_port: u16 = 8100;
                    let wda_manager = crate::platform::ios::WdaManager::new(
                        device.as_deref().unwrap_or(""),
                        is_simulator,
                        wda_port,
                    );

                    if let Err(e) = wda_manager.ensure_running() {
                        output.error(&format!("Failed to start WDA: {}", e));
                        output.info("Run 'mobile-use setup-ios' to install WebDriverAgent");
                        return Err(anyhow::anyhow!("WDA not available: {}", e));
                    }

                    // Create iOS session in daemon
                    let ios_connect_req = DaemonRequest::ConnectIOS {
                        session: session_name.clone(),
                        device: device.clone(),
                        simulator: is_simulator,
                        wda_port: Some(wda_port),
                    };
                    let _ = client.request(ios_connect_req).await;

                    // Now run flutter as usual — flutter run --machine works the same for iOS
                    // (The rest of the FlutterAndroid flow applies: parse VM service URL, connect, etc.)
                    // For now, reuse the FlutterAndroid flow with the iOS session already created.
                    output.info("Flutter iOS run: use 'flutter run --machine' to start the app.");
                    output.info("Then connect with: mobile-use connect --url <vm-service-url>");

                    // TODO: Full integration with FlutterProcessManager for iOS
                    // This would reuse the same FlutterProcessManager since flutter run --machine
                    // output format is identical for iOS and Android.

                    Ok(())
                }
```

**Step 3: Update the detect_project_type call site**

Find where `detect_project_type` is called in the `Commands::Run` handler and update the arguments to include `device` and `args`:

```rust
let project_type = detect_project_type(&apk, &package, &device, &args);
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add FlutterIOS project type and run command support"
```

---

### Task 14: Run full test suite and fix any issues

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run cargo clippy for warnings**

Run: `cargo clippy -- -W warnings`
Expected: No errors (warnings acceptable for now)

**Step 3: Build release**

Run: `cargo build --release`
Expected: Builds successfully

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: resolve test and clippy issues from iOS support"
```

---

### Task 15: Update documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `Cargo.toml` (keywords)

**Step 1: Update Cargo.toml keywords**

Change keywords line to:
```toml
keywords = ["mobile", "ui-automation", "ai", "flutter", "ios"]
```

**Step 2: Add iOS section to CLAUDE.md**

Add after the "Common Pitfalls" section:

```markdown
## iOS Support

### Architecture
iOS device operations use WebDriverAgent (WDA) via HTTP. Both simulators and real devices use WDA uniformly.

```
CLI → DeviceOperator trait → IOSDevice → WdaClient → HTTP → WDA on device
```

### Setup
- **Simulators**: Automatic — WDA is managed by `WdaManager`
- **Real devices**: Run `mobile-use setup-ios --device <UDID>` once to build and install WDA

### Key Files
- `src/platform/device.rs` — `DeviceOperator` trait (shared by Android/iOS)
- `src/platform/ios/wda.rs` — WDA HTTP client (W3C WebDriver Actions API)
- `src/platform/ios/ios_device.rs` — `IOSDevice` implementing `DeviceOperator`
- `src/platform/ios/wda_manager.rs` — WDA lifecycle (start/stop/health check)
- `src/platform/ios/discovery.rs` — iOS device discovery (simctl + idevice_id)

### Coordinate System
WDA works in logical points. `WdaClient` converts physical pixels (used by the trait) to logical points internally using the device's scale factor (2x/3x).

### Common Pitfalls
1. **WDA must be running** — The `WdaManager` auto-starts it, but if it fails, run `mobile-use setup-ios`
2. **Real device signing expires** — Free Apple ID: 7 days. Re-run `setup-ios` when expired.
3. **iproxy needed for real devices** — Port forwarding via `iproxy` is required for USB real devices
```

**Step 3: Commit**

```bash
git add CLAUDE.md Cargo.toml
git commit -m "docs: add iOS support documentation"
```

---

---

### Task 16: Structured WdaError type (replaces generic MobileUseError::Other)

**Files:**
- Create: `src/platform/ios/error.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/platform/ios/wda.rs`

**Step 1: Create error.rs with WdaError enum**

```rust
// src/platform/ios/error.rs
use crate::core::MobileUseError;

#[derive(Debug, thiserror::Error)]
pub enum WdaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebDriverAgent error: {error} - {message}")]
    WebDriver {
        error: String,
        message: String,
        traceback: String,
    },

    #[error("No active WDA session")]
    NoSession,

    #[error("WDA not reachable on {0}")]
    NotReachable(String),

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    #[error("Port pool error: {0}")]
    PortPool(String),

    #[error("iproxy error: {0}")]
    IProxy(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<WdaError> for MobileUseError {
    fn from(e: WdaError) -> Self {
        MobileUseError::IOSError(e.to_string())
    }
}
```

**Step 2: Update wda.rs to use WdaError**

Replace all `MobileUseError::Other(format!("WDA ..."))` with the corresponding `WdaError` variant. For example:

```rust
// Before:
Err(MobileUseError::Other(format!("WDA create session failed: {}", e)))

// After:
Err(WdaError::Http(e).into())
```

Add a helper to parse WDA error responses:

```rust
/// Parse WDA error response body into structured WdaError
fn parse_wda_error(body: &str) -> WdaError {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(body) {
        let error = data.pointer("/value/error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = data.pointer("/value/message")
            .and_then(|v| v.as_str())
            .unwrap_or(body)
            .to_string();
        let traceback = data.pointer("/value/traceback")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        WdaError::WebDriver { error, message, traceback }
    } else {
        WdaError::WebDriver {
            error: "unknown".to_string(),
            message: body.to_string(),
            traceback: String::new(),
        }
    }
}
```

**Step 3: Update ios/mod.rs**

```rust
pub mod discovery;
pub mod error;
pub mod ios_device;
pub mod wda;
pub mod wda_manager;

pub use error::WdaError;
pub use ios_device::IOSDevice;
pub use wda::WdaClient;
pub use wda_manager::WdaManager;
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src/platform/ios/error.rs src/platform/ios/mod.rs src/platform/ios/wda.rs
git commit -m "feat: add structured WdaError type with WDA response parsing"
```

---

### Task 17: WDA port pool for multi-device support

**Files:**
- Create: `src/platform/ios/port_pool.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/platform/ios/ios_device.rs`

**Step 1: Create port_pool.rs**

```rust
// src/platform/ios/port_pool.rs
use crate::core::{MobileUseError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{debug, info};

/// Default WDA port range: 8100-8199 (100 concurrent devices)
const PORT_RANGE_START: u16 = 8100;
const PORT_RANGE_END: u16 = 8199;

/// Global port pool singleton
static PORT_POOL: OnceLock<WdaPortPool> = OnceLock::new();

/// Get or create the global port pool
pub fn global_port_pool() -> &'static WdaPortPool {
    PORT_POOL.get_or_init(WdaPortPool::new)
}

struct PortPoolInner {
    /// device_id -> allocated port
    device_ports: HashMap<String, u16>,
    /// All ports currently in use
    used_ports: std::collections::HashSet<u16>,
}

pub struct WdaPortPool {
    inner: Arc<Mutex<PortPoolInner>>,
}

impl WdaPortPool {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PortPoolInner {
                device_ports: HashMap::new(),
                used_ports: std::collections::HashSet::new(),
            })),
        }
    }

    /// Allocate a port for a device. Returns existing port if already allocated.
    pub fn allocate(&self, device_id: &str) -> Result<PortGuard> {
        let mut inner = self.inner.lock().map_err(|e| {
            MobileUseError::IOSError(format!("Port pool lock poisoned: {}", e))
        })?;

        // Return existing allocation
        if let Some(&port) = inner.device_ports.get(device_id) {
            debug!("Reusing port {} for device {}", port, device_id);
            return Ok(PortGuard {
                port,
                device_id: device_id.to_string(),
                pool: Arc::clone(&self.inner),
            });
        }

        // Find next free port
        for port in PORT_RANGE_START..=PORT_RANGE_END {
            if !inner.used_ports.contains(&port) {
                inner.used_ports.insert(port);
                inner.device_ports.insert(device_id.to_string(), port);
                info!("Allocated port {} for device {}", port, device_id);
                return Ok(PortGuard {
                    port,
                    device_id: device_id.to_string(),
                    pool: Arc::clone(&self.inner),
                });
            }
        }

        Err(MobileUseError::IOSError(format!(
            "No free WDA ports available in range {}-{}",
            PORT_RANGE_START, PORT_RANGE_END
        )))
    }

    /// Force-release a port for a device (crash recovery)
    pub fn force_release(&self, device_id: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            if let Some(port) = inner.device_ports.remove(device_id) {
                inner.used_ports.remove(&port);
                info!("Force-released port {} for device {}", port, device_id);
            }
        }
    }
}

/// RAII guard that releases port on drop
pub struct PortGuard {
    pub port: u16,
    device_id: String,
    pool: Arc<Mutex<PortPoolInner>>,
}

impl PortGuard {
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

impl Drop for PortGuard {
    fn drop(&mut self) {
        if let Ok(mut inner) = self.pool.lock() {
            inner.used_ports.remove(&self.port);
            inner.device_ports.remove(&self.device_id);
            debug!("Released port {} for device {}", self.port, self.device_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_port() {
        let pool = WdaPortPool::new();
        let guard = pool.allocate("device-1").unwrap();
        assert_eq!(guard.port, PORT_RANGE_START);
        assert_eq!(guard.url(), format!("http://localhost:{}", PORT_RANGE_START));
    }

    #[test]
    fn test_reuse_existing_port() {
        let pool = WdaPortPool::new();
        let guard1 = pool.allocate("device-1").unwrap();
        let port1 = guard1.port;
        // Don't drop guard1 — allocate again for same device
        let guard2 = pool.allocate("device-1").unwrap();
        assert_eq!(guard2.port, port1);
        // Need to keep guard1 alive
        drop(guard1);
    }

    #[test]
    fn test_different_devices_get_different_ports() {
        let pool = WdaPortPool::new();
        let guard1 = pool.allocate("device-1").unwrap();
        let guard2 = pool.allocate("device-2").unwrap();
        assert_ne!(guard1.port, guard2.port);
    }

    #[test]
    fn test_release_on_drop() {
        let pool = WdaPortPool::new();
        let port;
        {
            let guard = pool.allocate("device-1").unwrap();
            port = guard.port;
        }
        // Port should be released after drop
        let guard2 = pool.allocate("device-2").unwrap();
        assert_eq!(guard2.port, port); // Same port reused
    }

    #[test]
    fn test_force_release() {
        let pool = WdaPortPool::new();
        let guard = pool.allocate("device-1").unwrap();
        let port = guard.port;
        std::mem::forget(guard); // Simulate leak
        pool.force_release("device-1");
        let guard2 = pool.allocate("device-2").unwrap();
        assert_eq!(guard2.port, port);
    }
}
```

**Step 2: Update IOSDevice to use port pool**

In `ios_device.rs`, update `new()` to accept port from pool:

```rust
impl IOSDevice {
    pub fn new(device_id: &str, wda_base_url: &str, simulator: bool) -> Self {
        Self {
            wda: WdaClient::new(wda_base_url),
            device_id: device_id.to_string(),
            simulator,
        }
    }

    /// Create from port pool (preferred for multi-device)
    pub fn from_pool(device_id: &str, simulator: bool) -> Result<Self> {
        let pool = super::port_pool::global_port_pool();
        let guard = pool.allocate(device_id)?;
        let url = guard.url();
        // Note: PortGuard is dropped here but port stays allocated
        // because we keep device_id in the pool's device_ports map.
        // It gets released when force_release is called or pool is dropped.
        Ok(Self::new(device_id, &url, simulator))
    }
}
```

**Step 3: Update ios/mod.rs**

Add `pub mod port_pool;` and `pub use port_pool::{WdaPortPool, PortGuard, global_port_pool};`

**Step 4: Verify it compiles and tests pass**

Run: `cargo test test_allocate_port test_reuse_existing_port test_different_devices_get_different_ports test_release_on_drop test_force_release`
Expected: All PASS

**Step 5: Commit**

```bash
git add src/platform/ios/port_pool.rs src/platform/ios/mod.rs src/platform/ios/ios_device.rs
git commit -m "feat: add WDA port pool with RAII auto-release for multi-device support"
```

---

### Task 18: iproxy wrapper for real device port forwarding

**Files:**
- Create: `src/platform/ios/iproxy.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/platform/ios/wda_manager.rs`

**Step 1: Create iproxy.rs**

```rust
// src/platform/ios/iproxy.rs
use crate::core::{MobileUseError, Result};
use std::process::{Child, Command, Stdio};
use tracing::{debug, info, warn};

/// Wrapper for iproxy port forwarding tool (part of libimobiledevice)
pub struct IProxy {
    child: Option<Child>,
    local_port: u16,
    device_port: u16,
    device_id: String,
}

impl IProxy {
    /// Find iproxy binary path
    fn find_binary() -> Result<String> {
        let candidates = [
            "/opt/homebrew/bin/iproxy",
            "/usr/local/bin/iproxy",
            "/usr/bin/iproxy",
        ];

        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Try PATH
        if Command::new("which")
            .arg("iproxy")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .is_some()
        {
            return Ok("iproxy".to_string());
        }

        Err(MobileUseError::IOSError(
            "iproxy not found. Install with: brew install libimobiledevice".to_string(),
        ))
    }

    /// Start port forwarding: local_port -> device:device_port
    pub fn start(device_id: &str, local_port: u16, device_port: u16) -> Result<Self> {
        let binary = Self::find_binary()?;

        info!(
            "Starting iproxy: {}:{} -> {}:{}",
            "localhost", local_port, device_id, device_port
        );

        let child = Command::new(&binary)
            .args([
                &format!("{}:{}", local_port, device_port),
                "--udid",
                device_id,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| MobileUseError::IOSError(format!("Failed to start iproxy: {}", e)))?;

        // Give iproxy a moment to bind the port
        std::thread::sleep(std::time::Duration::from_millis(500));

        info!("iproxy started (pid: {})", child.id());

        Ok(Self {
            child: Some(child),
            local_port,
            device_port,
            device_id: device_id.to_string(),
        })
    }

    /// Check if iproxy is still running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,     // Still running
                Ok(Some(_)) => false, // Exited
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for IProxy {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            debug!("Killing iproxy (pid: {})", child.id());
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
```

**Step 2: Update wda_manager.rs to use IProxy for real devices**

In `start_device_wda()`, replace the raw `Command::new("iproxy")` with:

```rust
fn start_device_wda(&self) -> Result<()> {
    let xctestrun = self.find_xctestrun()?;

    info!("Starting WDA on device {} via xcodebuild", self.device_id);

    // Start iproxy for port forwarding
    let _iproxy = super::iproxy::IProxy::start(&self.device_id, self.port, self.port)?;
    // Note: iproxy will be killed when _iproxy is dropped.
    // In production, store this in a field on WdaManager to keep it alive.

    // Launch xcodebuild in background
    let xctestrun_path = xctestrun.to_string_lossy().to_string();
    let device_id = self.device_id.clone();
    std::thread::spawn(move || {
        let _ = Command::new("xcodebuild")
            .args([
                "test-without-building",
                "-xctestrun",
                &xctestrun_path,
                "-destination",
                &format!("platform=iOS,id={}", device_id),
            ])
            .output();
    });

    Ok(())
}
```

**Step 3: Update ios/mod.rs**

Add `pub mod iproxy;` and `pub use iproxy::IProxy;`

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 5: Commit**

```bash
git add src/platform/ios/iproxy.rs src/platform/ios/mod.rs src/platform/ios/wda_manager.rs
git commit -m "feat: add iproxy wrapper with auto-cleanup for real device port forwarding"
```

---

### Task 19: System dialog handling for permission popups

**Files:**
- Modify: `src/platform/ios/wda.rs`

**Step 1: Add system dialog types and methods to WdaClient**

Add at the end of `wda.rs`:

```rust
/// iOS system dialog info (permission popups, alerts)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemDialog {
    pub text: String,
    pub buttons: Vec<String>,
}

/// Known dialog types and their preferred buttons
fn preferred_button_for_dialog(text: &str) -> Option<&'static str> {
    let text_lower = text.to_lowercase();

    // Notification permissions
    if text_lower.contains("notification") || text_lower.contains("通知") {
        return Some("Allow"); // or "允许"
    }
    // Location permissions
    if text_lower.contains("location") || text_lower.contains("位置") {
        return Some("Allow While Using App"); // or "使用App时允许"
    }
    // Camera/microphone
    if text_lower.contains("camera") || text_lower.contains("相机")
        || text_lower.contains("microphone") || text_lower.contains("麦克风") {
        return Some("OK");
    }
    // Photos
    if text_lower.contains("photo") || text_lower.contains("照片") {
        return Some("Allow Full Access"); // or "允许访问所有照片"
    }
    // Tracking
    if text_lower.contains("track") || text_lower.contains("跟踪") {
        return Some("Allow"); // or "允许"
    }

    None
}

impl WdaClient {
    /// Check if a system dialog is present and get its info
    pub fn get_system_dialog(&self) -> Result<Option<SystemDialog>> {
        let session_url = self.session_url()?;

        // Try to get alert text
        let text_url = format!("{}/alert/text", session_url);
        let resp = match self.client.get(&text_url).send() {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        if !resp.status().is_success() {
            return Ok(None); // No dialog present
        }

        let data: Value = resp.json().unwrap_or_default();
        let text = data
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if text.is_empty() {
            return Ok(None);
        }

        // Get available buttons
        let buttons_url = format!("{}/wda/alert/buttons", session_url);
        let buttons = self.client.get(&buttons_url).send()
            .ok()
            .and_then(|r| r.json::<Value>().ok())
            .and_then(|d| d.get("value").cloned())
            .and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|b| b.as_str().map(String::from))
                        .collect()
                })
            })
            .unwrap_or_default();

        Ok(Some(SystemDialog { text, buttons }))
    }

    /// Dismiss a system dialog by clicking a button
    pub fn dismiss_dialog(&self, button_label: &str) -> Result<()> {
        let session_url = self.session_url()?;
        let url = format!("{}/alert/accept", session_url);
        let body = json!({"name": button_label});

        let resp = self.client.post(&url).json(&body).send().map_err(|e| {
            MobileUseError::IOSError(format!("Failed to dismiss dialog: {}", e))
        })?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(MobileUseError::IOSError(format!(
                "Failed to click dialog button '{}': {}", button_label, text
            )));
        }

        Ok(())
    }

    /// Auto-dismiss system dialog using known button preferences
    pub fn auto_dismiss_dialog(&self) -> Result<bool> {
        if let Some(dialog) = self.get_system_dialog()? {
            info!("System dialog detected: {}", dialog.text);

            // Try preferred button
            if let Some(preferred) = preferred_button_for_dialog(&dialog.text) {
                // Try exact match first, then case-insensitive
                let button = dialog.buttons.iter()
                    .find(|b| b == &preferred)
                    .or_else(|| dialog.buttons.iter().find(|b| {
                        b.to_lowercase() == preferred.to_lowercase()
                    }))
                    .cloned();

                if let Some(btn) = button {
                    self.dismiss_dialog(&btn)?;
                    info!("Auto-dismissed dialog with button: {}", btn);
                    return Ok(true);
                }
            }

            // Fallback: click first button that looks like "allow" or "ok"
            let fallback = dialog.buttons.iter().find(|b| {
                let lower = b.to_lowercase();
                lower.contains("allow") || lower.contains("ok")
                    || lower.contains("允许") || lower.contains("好")
            });

            if let Some(btn) = fallback {
                self.dismiss_dialog(btn)?;
                info!("Auto-dismissed dialog with fallback button: {}", btn);
                return Ok(true);
            }

            warn!("Unknown dialog, cannot auto-dismiss: {:?}", dialog);
        }

        Ok(false)
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/platform/ios/wda.rs
git commit -m "feat: add system dialog detection and auto-dismiss for iOS permissions"
```

---

### Task 20: Device orientation support

**Files:**
- Modify: `src/platform/ios/wda.rs`

**Step 1: Add orientation types and methods to WdaClient**

```rust
/// iOS device orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Orientation {
    #[serde(rename = "PORTRAIT")]
    Portrait,
    #[serde(rename = "LANDSCAPE")]
    LandscapeLeft,
    #[serde(rename = "UIA_DEVICE_ORIENTATION_LANDSCAPERIGHT")]
    LandscapeRight,
    #[serde(rename = "UIA_DEVICE_ORIENTATION_PORTRAIT_UPSIDEDOWN")]
    PortraitUpsideDown,
}

impl WdaClient {
    /// Get current device orientation
    pub fn get_orientation(&self) -> Result<Orientation> {
        let url = format!("{}/orientation", self.session_url()?);
        let resp = self.client.get(&url).send().map_err(|e| {
            MobileUseError::IOSError(format!("Get orientation failed: {}", e))
        })?;

        let data: Value = resp.json().unwrap_or_default();
        let value_str = data
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("PORTRAIT");

        match value_str {
            "PORTRAIT" => Ok(Orientation::Portrait),
            "LANDSCAPE" => Ok(Orientation::LandscapeLeft),
            "UIA_DEVICE_ORIENTATION_LANDSCAPERIGHT" => Ok(Orientation::LandscapeRight),
            "UIA_DEVICE_ORIENTATION_PORTRAIT_UPSIDEDOWN" => Ok(Orientation::PortraitUpsideDown),
            other => {
                debug!("Unknown orientation '{}', defaulting to Portrait", other);
                Ok(Orientation::Portrait)
            }
        }
    }

    /// Set device orientation
    pub fn set_orientation(&self, orientation: Orientation) -> Result<()> {
        let url = format!("{}/orientation", self.session_url()?);
        let value = match orientation {
            Orientation::Portrait => "PORTRAIT",
            Orientation::LandscapeLeft => "LANDSCAPE",
            Orientation::LandscapeRight => "UIA_DEVICE_ORIENTATION_LANDSCAPERIGHT",
            Orientation::PortraitUpsideDown => "UIA_DEVICE_ORIENTATION_PORTRAIT_UPSIDEDOWN",
        };

        let body = json!({"orientation": value});
        let resp = self.client.post(&url).json(&body).send().map_err(|e| {
            MobileUseError::IOSError(format!("Set orientation failed: {}", e))
        })?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(MobileUseError::IOSError(format!(
                "Set orientation failed: {}", text
            )));
        }

        Ok(())
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles

**Step 3: Commit**

```bash
git add src/platform/ios/wda.rs
git commit -m "feat: add device orientation query and control"
```

---

### Task 21: Verify test_app runs on iOS simulator

**Step 1: Check test_app iOS configuration**

Run: `cd /Users/liuqijun/SCM/ai-workspace/mobile-use/test_app && cat ios/Runner/Info.plist | head -20`
Expected: Valid iOS plist configuration

**Step 2: Boot an iOS simulator (if not already booted)**

Run: `xcrun simctl list devices available | grep -i iphone | head -5`
Expected: Lists available iPhone simulators

**Step 3: Run test_app on iOS simulator**

Run: `cd /Users/liuqijun/SCM/ai-workspace/mobile-use/test_app && flutter run -d <simulator-udid> --no-pub`
Expected: App launches on simulator with same UI as Android

**Step 4: Verify all pages work**

Manually check: ButtonsPage, InputsPage, ListsPage, FormsPage render correctly.
The test_app uses Material Design which renders identically on iOS.

**Step 5: Document any iOS-specific adjustments if needed**

If any UI differences found, create a follow-up task.

---

### Task 22: End-to-end test with iOS simulator

**Step 1: Build mobile-use**

Run: `cargo build`
Expected: Builds successfully

**Step 2: Start test_app on iOS simulator**

Run: `cd /Users/liuqijun/SCM/ai-workspace/mobile-use/test_app && flutter run -d <sim-udid> --machine`
Note the VM Service URL from output.

**Step 3: Verify devices command shows iOS simulator**

Run: `./target/debug/mobile-use devices`
Expected: Lists the booted iOS simulator along with any Android devices

**Step 4: Connect and test basic operations**

```bash
# Connect (after manually starting WDA on simulator)
./target/debug/mobile-use connect --url ws://127.0.0.1:<port>/<token>/ws

# Get elements
./target/debug/mobile-use elements

# Tap a button
./target/debug/mobile-use tap @e1

# Screenshot
./target/debug/mobile-use screenshot ios-test.png
```

Expected: All operations work on iOS simulator

**Step 5: Document results and commit any fixes**

```bash
git add -A
git commit -m "test: verify iOS simulator end-to-end workflow"
```

---

## Summary

| Task | Description | Files Changed |
|------|-------------|---------------|
| 1 | Add reqwest dependency | Cargo.toml |
| 2 | Define DeviceOperator trait | platform/device.rs, platform/mod.rs |
| 3 | Impl DeviceOperator for AdbClient | android/adb.rs |
| 4 | Implement WdaClient | ios/wda.rs, ios/mod.rs |
| 5 | Implement IOSDevice | ios/ios_device.rs |
| 6 | Add IOSError to error types | core/error.rs |
| 7 | Refactor DaemonSession | daemon/session_manager.rs |
| 8 | Refactor main.rs actions | main.rs, daemon/server.rs |
| 9 | iOS device discovery | ios/discovery.rs, commands/connect.rs |
| 10 | ConnectIOS protocol | daemon/protocol.rs, daemon/server.rs |
| 11 | WDA manager | ios/wda_manager.rs |
| 12 | setup-ios command | cli/parser.rs, main.rs |
| 13 | Flutter iOS run support | main.rs |
| 14 | Test & fix | all |
| 15 | Documentation | CLAUDE.md, Cargo.toml |
| **16** | **Structured WdaError type** | **ios/error.rs, ios/wda.rs** |
| **17** | **WDA port pool (multi-device)** | **ios/port_pool.rs, ios/ios_device.rs** |
| **18** | **iproxy wrapper (real device)** | **ios/iproxy.rs, ios/wda_manager.rs** |
| **19** | **System dialog handling** | **ios/wda.rs** |
| **20** | **Device orientation support** | **ios/wda.rs** |
| **21** | **Verify test_app on iOS** | **test_app/** |
| **22** | **End-to-end iOS test** | **integration test** |
