# iOS Real Device Support Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add iOS real device support to mobile-use so the same CLI commands work transparently on both Android and iOS.

**Architecture:** Extract a `DeviceOperator` trait from `AdbClient`, then implement `WdaClient` (WebDriverAgent HTTP client) behind the same trait. The daemon session holds `Box<dyn DeviceOperator>` instead of a hardcoded `AdbClient`. WDA communicates with iOS devices via W3C Actions API over HTTP.

**Tech Stack:** Rust, reqwest (blocking HTTP), WebDriverAgent, iproxy (libimobiledevice), xcodebuild

**Spec:** `docs/superpowers/specs/2026-03-11-ios-real-device-support-design.md`

---

## Chunk 1: Core Abstraction

### Task 1: Add reqwest dependency

**Files:**
- Modify: `Cargo.toml:13-28`

- [ ] **Step 1: Add reqwest to dependencies**

In `Cargo.toml`, add after the `regex` line (line 28):

```toml
reqwest = { version = "0.11", features = ["blocking", "json"] }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add reqwest dependency for iOS WDA HTTP client"
```

---

### Task 2: Define DeviceOperator trait and Platform enum

**Files:**
- Modify: `src/core/types.rs:1-230`
- Test: existing tests in `src/core/types.rs` still pass

- [ ] **Step 1: Add Platform enum and DeviceOperator trait**

Add at the end of `src/core/types.rs` (before the closing, after line 230):

```rust
/// Device platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

/// Trait abstracting device operations across platforms (Android/iOS)
pub trait DeviceOperator: Send + Sync {
    /// Tap at physical pixel coordinates
    fn tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Double tap at physical pixel coordinates
    fn double_tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Long press at physical pixel coordinates for given duration
    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Swipe from (x1,y1) to (x2,y2) over duration
    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Input text string
    fn input_text(&self, text: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Send key event
    fn keyevent(&self, key: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Take screenshot and save to local path
    fn screenshot(&self, local_path: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Get screen size in physical pixels (width, height)
    fn get_screen_size(&self) -> std::result::Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>>;
    /// Get device platform
    fn platform(&self) -> Platform;
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test`
Expected: all existing tests pass, no regressions

- [ ] **Step 3: Commit**

```bash
git add src/core/types.rs
git commit -m "feat: define DeviceOperator trait and Platform enum"
```

---

### Task 3: Implement DeviceOperator for AdbClient

**Files:**
- Modify: `src/platform/android/adb.rs:1-328`

- [ ] **Step 1: Add DeviceOperator import and implementation**

Add at the end of `src/platform/android/adb.rs`, before the `#[cfg(test)]` block (before line 315):

```rust
use crate::core::types::{DeviceOperator, Platform};

impl DeviceOperator for AdbClient {
    fn tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::tap(self, x, y).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn double_tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tap(x, y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.tap(x, y)?;
        Ok(())
    }

    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::long_press(self, x, y, duration_ms).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::swipe(self, x1, y1, x2, y2, duration_ms).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn input_text(&self, text: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::input_text(self, text).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn keyevent(&self, key: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::keyevent(self, key).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn screenshot(&self, local_path: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::screenshot(self, local_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn get_screen_size(&self) -> std::result::Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::get_screen_size(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn platform(&self) -> Platform {
        Platform::Android
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 3: Commit**

```bash
git add src/platform/android/adb.rs
git commit -m "feat: implement DeviceOperator trait for AdbClient"
```

---

### Task 4: Refactor DaemonSession to use DeviceOperator

**Files:**
- Modify: `src/daemon/session_manager.rs:1-210`
- Modify: `src/daemon/server.rs` (all places using `session.adb`)

- [ ] **Step 1: Update DaemonSession struct**

In `src/daemon/session_manager.rs`, replace the struct and `new()`:

Change:
```rust
use crate::platform::android::AdbClient;
```
to:
```rust
use crate::core::types::{DeviceOperator, Platform};
use crate::platform::android::AdbClient;
```

Change the struct from:
```rust
pub struct DaemonSession {
    pub name: String,
    pub device: Option<String>,
    pub vm_url: Option<String>,
    pub vm_service: VmServiceClient,
    pub adb: AdbClient,
    pub ref_map: RefMap,
    pub has_flutter_process: bool,
    pub package: Option<String>,
}
```
to:
```rust
pub struct DaemonSession {
    pub name: String,
    pub device: Option<String>,
    pub vm_url: Option<String>,
    pub vm_service: VmServiceClient,
    pub device_op: Box<dyn DeviceOperator>,
    pub platform: Platform,
    pub ref_map: RefMap,
    pub has_flutter_process: bool,
    pub package: Option<String>,
}
```

Change `DaemonSession::new()` from:
```rust
    pub fn new(name: &str, device: Option<String>) -> Self {
        info!("Creating session: {} (device: {:?})", name, device);
        Self {
            name: name.to_string(),
            device: device.clone(),
            vm_url: None,
            vm_service: VmServiceClient::new(),
            adb: AdbClient::new(device),
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
        }
    }
```
to:
```rust
    pub fn new(name: &str, device: Option<String>) -> Self {
        info!("Creating session: {} (device: {:?})", name, device);
        let adb = AdbClient::new(device.clone());
        Self {
            name: name.to_string(),
            device: device,
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op: Box::new(adb),
            platform: Platform::Android,
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
        }
    }
```

Add a new constructor for iOS:
```rust
    pub fn new_ios(name: &str, device: Option<String>, device_op: Box<dyn DeviceOperator>) -> Self {
        info!("Creating iOS session: {} (device: {:?})", name, device);
        Self {
            name: name.to_string(),
            device,
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op,
            platform: Platform::IOS,
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
        }
    }
```

- [ ] **Step 2: Update server.rs references from session.adb to session.device_op**

Search `src/daemon/server.rs` for any direct use of `session.adb` or `sess.adb` and replace with `session.device_op` / `sess.device_op`. (Based on exploration, the server primarily delegates to the session manager, so these may be minimal or zero changes.)

- [ ] **Step 3: Update main.rs get_adb_from_session and action functions**

In `src/main.rs`, the `get_adb_from_session` function (line 1269) currently returns `AdbClient`. This needs to change to return a reference or use the daemon's device_op. However, the action functions currently reconstruct an `AdbClient` from session info.

**Approach:** Instead of reconstructing `AdbClient` on the CLI side, send action requests to the daemon which executes them via its `device_op`. But this is a larger refactor. For now, keep `get_adb_from_session` for Android backward compatibility, and add a new daemon request type `ExecuteAction` in a later task.

For this task, just ensure compilation succeeds by keeping `get_adb_from_session` as-is (it creates a fresh `AdbClient` from session info, which still works for Android).

- [ ] **Step 4: Fix tests in session_manager.rs**

Update tests that reference `session.adb` to use `session.device_op` instead. The tests only check session creation and management, so they should work as-is if the struct compiles.

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add src/daemon/session_manager.rs src/daemon/server.rs src/main.rs
git commit -m "refactor: replace AdbClient with Box<dyn DeviceOperator> in DaemonSession"
```

---

## Chunk 2: WDA Client Implementation

### Task 5: Create iOS platform module structure

**Files:**
- Create: `src/platform/ios/mod.rs`
- Create: `src/platform/ios/wda.rs`
- Modify: `src/platform/mod.rs:1-2`

- [ ] **Step 1: Create module files**

Create `src/platform/ios/mod.rs`:
```rust
pub mod wda;

pub use wda::WdaClient;
```

Create `src/platform/ios/wda.rs` with a minimal struct:
```rust
use crate::core::types::{DeviceOperator, Platform};

/// WebDriverAgent HTTP client for iOS device automation
pub struct WdaClient {
    /// Base URL for WDA HTTP API (e.g., "http://localhost:8100")
    base_url: String,
    /// WDA session ID
    session_id: Option<String>,
    /// Device scale factor (logical points to physical pixels)
    scale: f64,
    /// HTTP client
    client: reqwest::blocking::Client,
}
```

Update `src/platform/mod.rs`:
```rust
pub mod android;
pub mod flutter;
pub mod ios;
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles (WdaClient struct exists but no methods yet)

- [ ] **Step 3: Commit**

```bash
git add src/platform/mod.rs src/platform/ios/
git commit -m "feat: create iOS platform module structure with WdaClient stub"
```

---

### Task 6: Implement WdaClient core (session, status, scale detection)

**Files:**
- Modify: `src/platform/ios/wda.rs`

- [ ] **Step 1: Implement WdaClient new/connect/session management**

Replace `src/platform/ios/wda.rs` with:

```rust
use crate::core::types::{DeviceOperator, Platform};
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
                    info!("iOS scale={}, screen={}x{} physical pixels", scale, phys_w, phys_h);
                    return Ok(());
                }
            }
        }

        // Fallback: common scale factors by logical width
        self.scale = if logical_width > 400.0 { 3.0 } else { 2.0 };
        let phys_w = (logical_width * self.scale) as i32;
        let phys_h = (logical_height * self.scale) as i32;
        self.screen_size = Some((phys_w, phys_h));
        info!("iOS scale={} (fallback), screen={}x{}", self.scale, phys_w, phys_h);
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
    fn perform_actions(&self, actions: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/actions", self.session_url()?);
        let body = json!({ "actions": actions });
        debug!("WDA actions: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with warnings about unused code (expected)

- [ ] **Step 3: Commit**

```bash
git add src/platform/ios/wda.rs
git commit -m "feat: implement WdaClient core (session, status, scale detection)"
```

---

### Task 7: Implement DeviceOperator for WdaClient

**Files:**
- Modify: `src/platform/ios/wda.rs`

- [ ] **Step 1: Add DeviceOperator implementation**

Append to `src/platform/ios/wda.rs`:

```rust
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

    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                // On iOS, select all text first (Cmd+A equivalent not available via keys)
                // Use WDA's /wda/element/select endpoint instead - but for keyevent,
                // we send End key which moves cursor to end
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

    fn screenshot(&self, local_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        self.screen_size.ok_or_else(|| "Screen size not detected".into())
    }

    fn platform(&self) -> Platform {
        Platform::IOS
    }
}
```

- [ ] **Step 2: Add base64 import at top of file**

Add to the top of `src/platform/ios/wda.rs`:
```rust
use base64::Engine as _;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add src/platform/ios/wda.rs
git commit -m "feat: implement DeviceOperator for WdaClient (tap, swipe, text, screenshot)"
```

---

### Task 4.5: Refactor action functions to support DeviceOperator via daemon

**Files:**
- Modify: `src/daemon/protocol.rs`
- Modify: `src/daemon/server.rs`
- Modify: `src/main.rs:1269-1644` (all action functions)

**Why:** All action functions (`tap_action`, `double_tap_action`, etc.) currently call `get_adb_from_session()` which returns an `AdbClient`. For iOS, we need the daemon to execute actions via `session.device_op` (which could be AdbClient or WdaClient).

- [ ] **Step 1: Add ExecuteAction request to daemon protocol**

In `src/daemon/protocol.rs`, add to `DaemonRequest`:

```rust
    /// Execute device action (tap, swipe, etc.) via session's DeviceOperator
    ExecuteAction {
        session: String,
        action: DeviceAction,
    },
```

Add a new enum:

```rust
/// Device action to execute via DeviceOperator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum DeviceAction {
    Tap { x: i32, y: i32 },
    DoubleTap { x: i32, y: i32 },
    LongPress { x: i32, y: i32, duration_ms: u32 },
    Swipe { x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32 },
    InputText { text: String },
    Keyevent { key: String },
    Screenshot { path: String },
    GetScreenSize,
}
```

- [ ] **Step 2: Handle ExecuteAction in daemon server**

In `src/daemon/server.rs`, add handler:

```rust
        DaemonRequest::ExecuteAction { session, action } => {
            let mut sessions_guard = sessions.lock().await;
            match sessions_guard.get_mut(&session) {
                Some(sess) => {
                    // Run blocking device_op calls in spawn_blocking
                    let result = match action {
                        DeviceAction::Tap { x, y } => sess.device_op.tap(x, y),
                        DeviceAction::DoubleTap { x, y } => sess.device_op.double_tap(x, y),
                        DeviceAction::LongPress { x, y, duration_ms } => sess.device_op.long_press(x, y, duration_ms),
                        DeviceAction::Swipe { x1, y1, x2, y2, duration_ms } => sess.device_op.swipe(x1, y1, x2, y2, duration_ms),
                        DeviceAction::InputText { ref text } => sess.device_op.input_text(text),
                        DeviceAction::Keyevent { ref key } => sess.device_op.keyevent(key),
                        DeviceAction::Screenshot { ref path } => sess.device_op.screenshot(path),
                        DeviceAction::GetScreenSize => {
                            match sess.device_op.get_screen_size() {
                                Ok((w, h)) => return DaemonResponse::ok(Some(json!({"width": w, "height": h}))),
                                Err(e) => return DaemonResponse::error(e.to_string()),
                            }
                        }
                    };
                    match result {
                        Ok(()) => DaemonResponse::ok(None),
                        Err(e) => DaemonResponse::error(e.to_string()),
                    }
                }
                None => DaemonResponse::error(format!("Session not found: {}", session)),
            }
        }
```

- [ ] **Step 3: Replace get_adb_from_session usage in action functions**

In `src/main.rs`, replace `get_adb_from_session` with a helper that sends `ExecuteAction` to daemon:

```rust
async fn execute_device_action(
    client: &mut DaemonClient,
    session: &str,
    action: crate::daemon::DeviceAction,
) -> Result<Option<serde_json::Value>> {
    let request = DaemonRequest::ExecuteAction {
        session: session.to_string(),
        action,
    };
    match client.request(request).await? {
        DaemonResponse::Ok { data } => Ok(data),
        DaemonResponse::Error { message } => Err(anyhow::anyhow!(message)),
        _ => Err(anyhow::anyhow!("Unexpected response")),
    }
}
```

Then update each action function. For example, `tap_action`:

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

    let (x, y) = element.bounds.center();
    execute_device_action(client, session, DeviceAction::Tap { x: x as i32, y: y as i32 })
        .await
        .map_err(|e| { output.error(&e.to_string()); e })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!("Tapped: {} \"{}\"", element.element_type, element.label.as_deref().unwrap_or(""))),
        data: None,
    });
    Ok(())
}
```

Apply same pattern to: `double_tap_action`, `long_press_action`, `input_action`, `clear_action`, `capture_action`, `scroll_action`, `swipe_action`.

For `scroll_action` and `swipe_action` which need `get_screen_size`, use `DeviceAction::GetScreenSize`.

For `clear_action` and `input_action`, the `clear_text_field` helper needs to send `Keyevent` actions via daemon instead of calling `adb.keyevent()` directly.

- [ ] **Step 4: Update clear_text_field to use DeviceAction**

Replace the current `clear_text_field(&adb, output)` calls with daemon-based keyevents:

```rust
async fn clear_text_field_via_daemon(
    client: &mut DaemonClient,
    session: &str,
) -> Result<()> {
    // Move to end
    execute_device_action(client, session, DeviceAction::Keyevent { key: "MOVE_END".to_string() }).await?;
    // Delete backwards
    for _ in 0..MAX_CLEAR_DELETE_PRESSES {
        execute_device_action(client, session, DeviceAction::Keyevent { key: "DEL".to_string() }).await?;
    }
    Ok(())
}
```

- [ ] **Step 5: Remove get_adb_from_session and map_adb_err**

These are no longer needed since all device operations go through the daemon.

- [ ] **Step 6: Run tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 7: Commit**

```bash
git add src/daemon/protocol.rs src/daemon/server.rs src/main.rs
git commit -m "refactor: route all device actions through daemon's DeviceOperator"
```

---

## Chunk 3: WDA Setup and CLI Integration

### Task 8: Implement setup-ios command (WDA build & install)

**Files:**
- Create: `src/platform/ios/wda_manager.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/cli/parser.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create wda_manager.rs**

Create `src/platform/ios/wda_manager.rs`:

```rust
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, debug};

/// Get the WDA project directory (cloned repo)
pub fn wda_project_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("WebDriverAgent")
}

/// Get the path to the built WDA runner app
pub fn wda_runner_path() -> PathBuf {
    wda_project_dir()
        .join("build")
        .join("Build")
        .join("Products")
        .join("Debug-iphoneos")
        .join("WebDriverAgentRunner-Runner.app")
}

/// Clone or update WebDriverAgent repository
pub fn ensure_wda_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let wda_dir = wda_project_dir();

    if wda_dir.join("WebDriverAgent.xcodeproj").exists() {
        info!("WDA repo already exists at {:?}, updating...", wda_dir);
        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(&wda_dir)
            .status()?;
        if !status.success() {
            info!("Git pull failed, continuing with existing version");
        }
    } else {
        info!("Cloning WebDriverAgent to {:?}", wda_dir);
        if let Some(parent) = wda_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/appium/WebDriverAgent.git",
                wda_dir.to_str().unwrap(),
            ])
            .status()?;
        if !status.success() {
            return Err("Failed to clone WebDriverAgent".into());
        }
    }

    Ok(wda_dir)
}

/// Build and install WDA on a real device
pub fn build_and_install_wda(
    device_id: &str,
    team_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wda_dir = ensure_wda_repo()?;

    info!("Building WebDriverAgent for device {}...", device_id);
    info!("Using development team: {}", team_id);

    let status = Command::new("xcodebuild")
        .args([
            "build-for-testing",
            "-project", "WebDriverAgent.xcodeproj",
            "-scheme", "WebDriverAgentRunner",
            "-destination", &format!("id={}", device_id),
            "-derivedDataPath", "build",
            "-allowProvisioningUpdates",
            &format!("DEVELOPMENT_TEAM={}", team_id),
            "CODE_SIGNING_ALLOWED=YES",
        ])
        .current_dir(&wda_dir)
        .status()?;

    if !status.success() {
        return Err("xcodebuild failed. Make sure your Apple Developer Team ID is correct.".into());
    }

    info!("WDA built successfully");
    Ok(())
}

/// Launch WDA on device using xcodebuild test
/// Returns the WDA port (default 8100)
pub fn launch_wda(device_id: &str, team_id: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let wda_dir = wda_project_dir();

    if !wda_dir.join("build").exists() {
        return Err("WDA not built. Run 'mobile-use setup-ios' first.".into());
    }

    info!("Launching WDA on device {}...", device_id);

    // Start xcodebuild test in background
    let child = Command::new("xcodebuild")
        .args([
            "test-without-building",
            "-project", "WebDriverAgent.xcodeproj",
            "-scheme", "WebDriverAgentRunner",
            "-destination", &format!("id={}", device_id),
            "-derivedDataPath", "build",
            &format!("DEVELOPMENT_TEAM={}", team_id),
        ])
        .current_dir(&wda_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    debug!("WDA xcodebuild test process started (PID: {})", child.id());

    // Save PID for cleanup
    let pid_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("wda.pid");
    std::fs::write(&pid_path, child.id().to_string())?;

    let wda_port = 8100u16;

    // Start iproxy for port forwarding
    start_iproxy(device_id, wda_port)?;

    // Wait for WDA to be ready
    wait_for_wda(wda_port)?;

    Ok(wda_port)
}

/// Start iproxy for port forwarding
fn start_iproxy(device_id: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting iproxy {}:{} for device {}", port, port, device_id);

    let child = Command::new("iproxy")
        .args([
            &port.to_string(),
            &port.to_string(),
            "-u", device_id,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Save PID for cleanup
    let pid_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("iproxy.pid");
    std::fs::write(&pid_path, child.id().to_string())?;

    // Brief wait for iproxy to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

/// Wait for WDA to respond on given port
fn wait_for_wda(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://localhost:{}/status", port);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    info!("Waiting for WDA to be ready on port {}...", port);

    for i in 0..30 {
        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                info!("WDA is ready (attempt {})", i + 1);
                return Ok(());
            }
            _ => {
                debug!("WDA not ready yet (attempt {})", i + 1);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }

    Err("WDA did not start within 60 seconds".into())
}

/// Kill running WDA and iproxy processes
pub fn stop_wda() {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use");

    for name in ["wda.pid", "iproxy.pid"] {
        let pid_path = cache_dir.join(name);
        if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let _ = Command::new("kill").args(["-TERM", &pid.to_string()]).status();
                info!("Stopped {} (PID {})", name.replace(".pid", ""), pid);
            }
            let _ = std::fs::remove_file(&pid_path);
        }
    }
}
```

- [ ] **Step 2: Update ios/mod.rs**

```rust
pub mod wda;
pub mod wda_manager;

pub use wda::WdaClient;
pub use wda_manager::{build_and_install_wda, launch_wda, stop_wda, ensure_wda_repo};
```

- [ ] **Step 3: Add SetupIos and ConnectIos CLI commands**

In `src/cli/parser.rs`, add to the `Commands` enum (after `Devices`):

```rust
    /// Setup iOS automation (build & install WebDriverAgent)
    #[command(name = "setup-ios", long_about = "Build and install WebDriverAgent on an iOS device.

Downloads the WebDriverAgent project and builds it with your Apple Developer certificate.
Run this once per device before using iOS automation.

Requires:
- Xcode installed
- Apple Developer account (free or paid)
- Device connected via USB

Example:
  mobile-use setup-ios --team-id YOUR_TEAM_ID
  mobile-use -d DEVICE_UDID setup-ios --team-id YOUR_TEAM_ID")]
    SetupIos {
        /// Apple Developer Team ID (10-character alphanumeric)
        #[arg(long, help = "Apple Developer Team ID")]
        team_id: String,
    },

    /// Connect to iOS device via WebDriverAgent
    #[command(name = "connect-ios", long_about = "Connect to an iOS device for UI automation.

Launches WebDriverAgent on the device and establishes connection.
Run 'setup-ios' first to install WDA on the device.

Example:
  mobile-use connect-ios --team-id YOUR_TEAM_ID
  mobile-use -d DEVICE_UDID connect-ios --team-id YOUR_TEAM_ID --port 8100")]
    ConnectIos {
        /// Apple Developer Team ID
        #[arg(long, help = "Apple Developer Team ID")]
        team_id: String,

        /// WDA port (default: 8100)
        #[arg(long, default_value = "8100", help = "WDA port")]
        port: u16,
    },
```

- [ ] **Step 4: Handle setup-ios command in main.rs**

Add handling in `src/main.rs` in the early command dispatch section (near the `Commands::Devices` handling around line 258):

```rust
        Commands::SetupIos { team_id } => {
            let device_id = args.device.unwrap_or_else(|| {
                // Try to auto-detect iOS device
                match std::process::Command::new("idevice_id").arg("-l").output() {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        stdout.lines().next().unwrap_or("").trim().to_string()
                    }
                    Err(_) => String::new(),
                }
            });

            if device_id.is_empty() {
                output.error("No iOS device found. Connect a device via USB and try again.");
                std::process::exit(1);
            }

            output.info(&format!("Setting up WDA for device: {}", device_id));

            use platform::ios::wda_manager;
            if let Err(e) = wda_manager::ensure_wda_repo() {
                output.error(&format!("Failed to get WDA: {}", e));
                std::process::exit(1);
            }

            match wda_manager::build_and_install_wda(&device_id, &team_id) {
                Ok(()) => {
                    output.success("WebDriverAgent installed successfully!");
                    output.info("Now run: mobile-use connect-ios --team-id YOUR_TEAM_ID");
                }
                Err(e) => {
                    output.error(&format!("Setup failed: {}", e));
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
```

- [ ] **Step 5: Handle connect-ios command in main.rs**

Add in the daemon-connected section (near `Commands::Connect` around line 347):

```rust
        Commands::ConnectIos { team_id, port } => {
            let device_id = device.clone().unwrap_or_else(|| {
                match std::process::Command::new("idevice_id").arg("-l").output() {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        stdout.lines().next().unwrap_or("").trim().to_string()
                    }
                    Err(_) => String::new(),
                }
            });

            if device_id.is_empty() {
                output.error("No iOS device found.");
                std::process::exit(1);
            }

            output.info(&format!("Launching WDA on device {}...", device_id));

            use platform::ios::wda_manager;
            let wda_port = match wda_manager::launch_wda(&device_id, &team_id) {
                Ok(p) => p,
                Err(e) => {
                    output.error(&format!("Failed to launch WDA: {}", e));
                    output.info("Run 'mobile-use setup-ios --team-id YOUR_TEAM_ID' first.");
                    std::process::exit(1);
                }
            };

            // Send ConnectIos request to daemon (daemon creates the WdaClient)
            let request = DaemonRequest::ConnectIos {
                session: session_name.clone(),
                device: Some(device_id.clone()),
                wda_port,
            };
            match client.request(request).await {
                Ok(DaemonResponse::Ok { .. }) => {
                    output.success(&format!("Connected to iOS device: {}", device_id));
                }
                Ok(DaemonResponse::Error { message }) => {
                    output.error(&message);
                    std::process::exit(1);
                }
                _ => {
                    output.error("Unexpected response");
                    std::process::exit(1);
                }
            }
            Ok(())
        }
```

- [ ] **Step 6: Add ConnectIos to daemon protocol**

In `src/daemon/protocol.rs`, add to `DaemonRequest` enum:

```rust
    /// Connect to iOS device via WDA
    ConnectIos {
        session: String,
        device: Option<String>,
        wda_port: u16,
    },
```

- [ ] **Step 7: Handle ConnectIos in daemon server**

In `src/daemon/server.rs`, add handling for `ConnectIos` (near the `ConnectAndroid` handler):

```rust
        DaemonRequest::ConnectIos {
            session,
            device,
            wda_port,
        } => {
            let wda_url = format!("http://localhost:{}", wda_port);
            // Use spawn_blocking since WdaClient::new makes blocking HTTP calls
            let wda_result = tokio::task::spawn_blocking(move || {
                crate::platform::ios::WdaClient::new(&wda_url)
            }).await;

            match wda_result {
                Ok(Ok(wda_client)) => {
                    let mut sessions_guard = sessions.lock().await;
                    let daemon_session = sessions_guard.get_or_create(&session, device.clone());
                    // Extract metadata before boxing (type erasure loses concrete type)
                    daemon_session.wda_port = Some(wda_port);
                    daemon_session.wda_session_id = wda_client.session_id().map(|s| s.to_string());
                    daemon_session.wda_scale = Some(wda_client.scale());
                    daemon_session.device_op = Box::new(wda_client);
                    daemon_session.platform = crate::core::types::Platform::IOS;
                    DaemonResponse::ok(Some(json!({
                        "session": session,
                        "mode": "ios",
                        "wda_port": wda_port,
                        "connected": true
                    })))
                }
                Ok(Err(e)) => DaemonResponse::error(format!("Failed to connect to WDA: {}", e)),
                Err(e) => DaemonResponse::error(format!("WDA connection task failed: {}", e)),
            }
        }
```

- [ ] **Step 8: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 9: Commit**

```bash
git add src/platform/ios/ src/cli/parser.rs src/main.rs src/daemon/protocol.rs src/daemon/server.rs
git commit -m "feat: add setup-ios and connect-ios commands with WDA manager"
```

---

### Task 9: Extend devices command to list iOS devices

**Files:**
- Create: `src/platform/ios/discovery.rs`
- Modify: `src/platform/ios/mod.rs`
- Modify: `src/commands/connect.rs`

- [ ] **Step 1: Create iOS device discovery**

Create `src/platform/ios/discovery.rs`:

```rust
use serde::Serialize;
use std::process::Command;
use tracing::debug;

/// iOS device information
#[derive(Debug, Clone, Serialize)]
pub struct IosDeviceInfo {
    pub id: String,
    pub name: String,
    pub ios_version: String,
    pub platform: String,
}

/// List connected iOS devices via idevice_id + ideviceinfo
pub fn list_ios_devices() -> Vec<IosDeviceInfo> {
    let output = match Command::new("idevice_id").arg("-l").output() {
        Ok(o) => o,
        Err(_) => {
            debug!("idevice_id not found, skipping iOS device discovery");
            return vec![];
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for udid in stdout.lines() {
        let udid = udid.trim();
        if udid.is_empty() {
            continue;
        }

        let name = get_device_prop(udid, "DeviceName").unwrap_or_else(|| "iPhone".to_string());
        let version = get_device_prop(udid, "ProductVersion").unwrap_or_else(|| "Unknown".to_string());

        devices.push(IosDeviceInfo {
            id: udid.to_string(),
            name,
            ios_version: version,
            platform: "ios".to_string(),
        });
    }

    devices
}

fn get_device_prop(udid: &str, key: &str) -> Option<String> {
    let output = Command::new("ideviceinfo")
        .args(["-u", udid, "-k", key])
        .output()
        .ok()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}
```

- [ ] **Step 2: Update ios/mod.rs**

```rust
pub mod discovery;
pub mod wda;
pub mod wda_manager;

pub use discovery::list_ios_devices;
pub use wda::WdaClient;
pub use wda_manager::{build_and_install_wda, launch_wda, stop_wda, ensure_wda_repo};
```

- [ ] **Step 3: Update devices command in commands/connect.rs**

Update the `devices` function to also list iOS devices:

```rust
use crate::platform::ios::discovery;
```

After the Android device listing, add iOS devices:

```rust
    let ios_devices = discovery::list_ios_devices();

    if device_list.is_empty() && ios_devices.is_empty() {
        output.error("No devices connected.");
        return Ok(());
    }

    // ... existing Android display code ...

    if !ios_devices.is_empty() {
        output.raw(&format!("\niOS Devices ({}):", ios_devices.len()));
        for (i, device) in ios_devices.iter().enumerate() {
            output.raw(&format!("  [{}] {}", device_list.len() + i + 1, device.id));
            output.raw(&format!("      Name:    {}", device.name));
            output.raw(&format!("      iOS:     {}", device.ios_version));
            output.raw("");
        }
    }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add src/platform/ios/discovery.rs src/platform/ios/mod.rs src/commands/connect.rs
git commit -m "feat: list iOS devices in 'mobile-use devices' command"
```

---

## Chunk 4: iOS Element Tree and End-to-End

### Task 10: Implement iOS element tree parsing from WDA /source

**Files:**
- Create: `src/platform/ios/elements.rs`
- Modify: `src/platform/ios/mod.rs`

- [ ] **Step 1: Create elements.rs**

Create `src/platform/ios/elements.rs`:

```rust
use crate::core::types::{Bounds, ElementNode, ElementRef, RefMap, StyleInfo};
use std::collections::HashMap;
use tracing::debug;

/// Fetch and parse the iOS accessibility element tree via WDA
pub fn fetch_element_tree(
    base_url: &str,
    session_id: &str,
    scale: f64,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Result<ElementNode, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = format!("{}/session/{}/source?format=json", base_url, session_id);
    let resp = client.get(&url).send()?;
    let data: serde_json::Value = resp.json()?;

    let source = data.get("value")
        .ok_or("No 'value' in /source response")?;

    let tree = parse_wda_element(source, scale, ref_map, interactive_only, 0);
    tree.ok_or_else(|| "Failed to parse element tree".into())
}

/// Parse a single WDA element node recursively
fn parse_wda_element(
    node: &serde_json::Value,
    scale: f64,
    ref_map: &mut RefMap,
    interactive_only: bool,
    depth: u32,
) -> Option<ElementNode> {
    let element_type_raw = node.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Other");

    // Map XCUIElementType* to simplified types
    let element_type = map_element_type(element_type_raw);

    let label = node.get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let name = node.get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let value = node.get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // Parse bounds from rect
    let bounds = parse_rect(node, scale);

    let is_interactive = is_interactive_type(&element_type);

    // Parse children first
    let children_raw = node.get("children")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let children: Vec<ElementNode> = children_raw
        .iter()
        .filter_map(|child| parse_wda_element(child, scale, ref_map, interactive_only, depth + 1))
        .collect();

    // Skip non-interactive leaf nodes if filtering
    if interactive_only && !is_interactive && children.is_empty() {
        return None;
    }

    // Skip containers with no label and no interactive children
    if interactive_only && !is_interactive && label.is_none() && children.is_empty() {
        return None;
    }

    // Build display label (prefer label, fallback to name, then value)
    let display_label = label.clone()
        .or_else(|| name.clone())
        .or_else(|| value.clone());

    // Build properties
    let mut properties = HashMap::new();
    if let Some(ref n) = name {
        properties.insert("name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(ref v) = value {
        properties.insert("value".to_string(), serde_json::Value::String(v.clone()));
    }
    let enabled = node.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    if !enabled {
        properties.insert("enabled".to_string(), serde_json::Value::Bool(false));
    }

    // Create ElementRef for ref_map
    let ref_id = if let Some(ref b) = bounds {
        let elem_ref = ElementRef {
            ref_id: String::new(), // set by ref_map.add()
            element_type: element_type.clone(),
            label: display_label.clone(),
            bounds: b.clone(),
            properties: properties.clone(),
            style: None,
        };
        ref_map.add(elem_ref)
    } else {
        format!("e_skip_{}", depth)
    };

    Some(ElementNode {
        ref_id,
        element_type,
        label: display_label,
        bounds,
        properties,
        style: None,
        children,
    })
}

/// Parse rect from WDA element node, converting to physical pixels
fn parse_rect(node: &serde_json::Value, scale: f64) -> Option<Bounds> {
    let rect = node.get("rect")?;
    let x = rect.get("x")?.as_f64()?;
    let y = rect.get("y")?.as_f64()?;
    let width = rect.get("width")?.as_f64()?;
    let height = rect.get("height")?.as_f64()?;

    // WDA returns logical points; convert to physical pixels
    Some(Bounds {
        x: x * scale,
        y: y * scale,
        width: width * scale,
        height: height * scale,
    })
}

/// Map XCUIElementType names to simplified element types
fn map_element_type(raw: &str) -> String {
    let simplified = raw.strip_prefix("XCUIElementType").unwrap_or(raw);
    match simplified {
        "Button" => "Button",
        "StaticText" => "Text",
        "TextField" | "SearchField" => "TextField",
        "SecureTextField" => "SecureTextField",
        "Image" => "Image",
        "Switch" | "Toggle" => "Switch",
        "Slider" => "Slider",
        "ScrollView" => "ScrollView",
        "Table" | "CollectionView" => "List",
        "Cell" => "ListItem",
        "NavigationBar" => "NavigationBar",
        "TabBar" => "TabBar",
        "Alert" => "Alert",
        "Picker" | "PickerWheel" => "Picker",
        "Link" => "Link",
        "CheckBox" => "Checkbox",
        "RadioButton" => "RadioButton",
        "Window" | "Application" => "Container",
        "Other" => "Container",
        other => other,
    }.to_string()
}

/// Check if an element type is interactive
fn is_interactive_type(element_type: &str) -> bool {
    matches!(
        element_type,
        "Button" | "TextField" | "SecureTextField" | "Switch"
        | "Slider" | "Picker" | "Link" | "Checkbox" | "RadioButton"
        | "ListItem"
    )
}
```

- [ ] **Step 2: Update ios/mod.rs**

```rust
pub mod discovery;
pub mod elements;
pub mod wda;
pub mod wda_manager;

pub use discovery::list_ios_devices;
pub use elements::fetch_element_tree;
pub use wda::WdaClient;
pub use wda_manager::{build_and_install_wda, launch_wda, stop_wda, ensure_wda_repo};
```

- [ ] **Step 3: Add session_id() and base_url() getters to WdaClient**

In `src/platform/ios/wda.rs`, add public getters:

```rust
    /// Get the WDA session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the scale factor
    pub fn scale(&self) -> f64 {
        self.scale
    }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add src/platform/ios/elements.rs src/platform/ios/mod.rs src/platform/ios/wda.rs
git commit -m "feat: implement iOS element tree parsing from WDA /source"
```

---

### Task 11: Integrate iOS elements into the elements command

**Files:**
- Modify: `src/main.rs` (Elements command handler, ~line 487-621)
- Modify: `src/daemon/session_manager.rs` (add platform info to Info response)

- [ ] **Step 1: Add iOS mode detection to elements command**

In `src/main.rs`, in the `Commands::Elements` handler, after the `is_android` check (around line 500), add an iOS check:

```rust
            // Check for iOS mode
            let is_ios = match &session_info {
                Ok(DaemonResponse::Ok { data: Some(data) }) => {
                    data.get("mode").and_then(|v| v.as_str()) == Some("ios")
                }
                _ => false,
            };

            if is_ios {
                // iOS mode: fetch element tree via WDA
                let wda_info = match &session_info {
                    Ok(DaemonResponse::Ok { data: Some(data) }) => data.clone(),
                    _ => {
                        output.error("No iOS session info");
                        return Err(anyhow::anyhow!("No iOS session info"));
                    }
                };

                let wda_port = wda_info.get("wda_port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(8100) as u16;
                let wda_session_id = wda_info.get("wda_session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let scale = wda_info.get("scale")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(3.0);

                let base_url = format!("http://localhost:{}", wda_port);
                let mut ref_map = RefMap::new();
                let tree = match platform::ios::fetch_element_tree(
                    &base_url, wda_session_id, scale, &mut ref_map, interactive,
                ) {
                    Ok(t) => t,
                    Err(e) => {
                        output.error(&format!("Failed to get iOS elements: {}", e));
                        return Err(anyhow::anyhow!("iOS elements failed: {}", e));
                    }
                };

                // Store refs
                let store_request = DaemonRequest::StoreRefs {
                    session: session_name.clone(),
                    refs: ref_map.refs.clone(),
                };
                let _ = client.request(store_request).await;

                let refs_json = json!(ref_map.refs);
                output.element_tree(&tree, &refs_json);
                return Ok(());
            }
```

- [ ] **Step 2: Update Info response to include platform and WDA details**

In `src/daemon/server.rs`, find the `DaemonRequest::Info` handler and add platform info to the response:

```rust
            let platform = sess.platform.to_string();
            // ... existing info fields ...
            // Add platform to response:
            "platform": platform,
            "mode": if sess.platform == crate::core::types::Platform::IOS { "ios" } else if sess.is_android_mode() { "android" } else { "flutter" },
```

- [ ] **Step 3: Store WDA session info in DaemonSession for Info retrieval**

Add `wda_port`, `wda_session_id`, and `wda_scale` fields to `DaemonSession` or store them in a properties map. Simplest: add optional fields:

In `src/daemon/session_manager.rs`, add to `DaemonSession`:
```rust
    /// WDA port (iOS mode)
    pub wda_port: Option<u16>,
    /// WDA session ID (iOS mode)
    pub wda_session_id: Option<String>,
    /// Device scale factor (iOS mode)
    pub wda_scale: Option<f64>,
```

Update `new()` and `new_ios()` to initialize these as `None`.

Update the `ConnectIos` handler in `server.rs` to store these values.

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/daemon/server.rs src/daemon/session_manager.rs
git commit -m "feat: integrate iOS element tree into elements command"
```

---

### Task 12: End-to-end verification on real device

- [ ] **Step 1: Build mobile-use**

Run: `cargo build`

- [ ] **Step 2: List devices (should show both Android and iOS)**

Run: `./target/debug/mobile-use devices`

- [ ] **Step 3: Setup WDA on iOS device**

Run: `./target/debug/mobile-use -d DEVICE_UDID setup-ios --team-id 6WQKPSN47B`

Wait for xcodebuild to complete (this may take a few minutes the first time).

- [ ] **Step 4: Connect to iOS device**

Run: `./target/debug/mobile-use -d DEVICE_UDID connect-ios --team-id 6WQKPSN47B`

Expected: "Connected to iOS device: DEVICE_UDID"

- [ ] **Step 5: Get elements**

Run: `./target/debug/mobile-use elements`

Expected: element tree showing MobileUseTest app UI

- [ ] **Step 6: Test tap**

Run: `./target/debug/mobile-use tap @e<N>` (using a button ref from elements output)

Expected: button tapped on device

- [ ] **Step 7: Test screenshot**

Run: `./target/debug/mobile-use screenshot ios-test.png`

Expected: screenshot saved

- [ ] **Step 8: Test text input**

Navigate to Text Inputs page, then:
Run: `./target/debug/mobile-use text @e<N> "hello"`

Expected: text entered in field

- [ ] **Step 9: Commit final working state**

```bash
git add src/platform/ios/ src/daemon/ src/main.rs src/cli/parser.rs src/core/types.rs src/commands/ Cargo.toml Cargo.lock
git commit -m "feat: iOS real device support with WDA - end-to-end verified on real iPhone"
```
