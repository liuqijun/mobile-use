# iOS Support Design

## Goal

Add iOS device support (simulator + real device) to mobile-use, so AI agents can automate iOS apps with the same CLI interface as Android. Platform differences are transparent to the agent.

## Architecture

### Current State

```
DaemonSession {
    adb: AdbClient,              // hardcoded Android
    vm_service: VmServiceClient, // Flutter (platform-agnostic)
}
```

### Target State

```
DaemonSession {
    device: Box<dyn DeviceOperator>,  // Android or iOS
    vm_service: VmServiceClient,      // unchanged
    platform: Platform,
}
```

### Layered Architecture

```
CLI Commands (tap, text, elements, screenshot...)
        |  unchanged, platform-transparent
Daemon / Session
        |  holds DeviceOperator trait object
Flutter VM Service (platform-agnostic)
        |  unchanged
DeviceOperator trait
   /              \
AdbClient      IOSDevice
(existing)     (new: delegates to WdaClient)
```

## DeviceOperator Trait

```rust
// src/platform/device.rs

pub enum Platform {
    Android,
    IOS,
}

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

Design decisions:
- Synchronous interface (matching existing AdbClient style, WDA uses reqwest::blocking)
- Coordinates in physical pixels (WdaClient converts internally from logical points)
- Device discovery NOT in trait (platform-specific, called separately)

## iOS Implementation

### Module Structure

```
src/platform/
├── mod.rs              // DeviceOperator trait + Platform enum
├── android/
│   ├── adb.rs          // + impl DeviceOperator for AdbClient
│   ├── uiautomator.rs
│   └── gradle.rs
└── ios/                // all new
    ├── mod.rs
    ├── ios_device.rs   // IOSDevice: DeviceOperator impl
    ├── wda.rs          // WdaClient: HTTP client, W3C Actions
    ├── wda_manager.rs  // WDA lifecycle (start/stop/health)
    └── discovery.rs    // device discovery (simctl + idevice_id)
```

### IOSDevice

```rust
pub struct IOSDevice {
    wda: WdaClient,
    device_id: String,
    simulator: bool,
}
```

Both simulator and real device use WDA uniformly. Simulator doesn't need iproxy port forwarding; real device does.

### WdaClient

HTTP client talking to WebDriverAgent via W3C WebDriver Actions API.

```rust
pub struct WdaClient {
    base_url: String,           // e.g. "http://localhost:8100"
    client: reqwest::blocking::Client,
    session_id: Option<String>,
}
```

Operation mapping:

| Operation | WDA API |
|-----------|---------|
| tap | POST /session/{id}/actions — pointerMove + pointerDown + pointerUp |
| double_tap | Two taps with 100ms pause |
| long_press | pointerDown + pause(duration) + pointerUp |
| swipe | pointerDown + pointerMove(duration) + pointerUp |
| input_text | key actions: keyDown/keyUp per character |
| screenshot | GET /screenshot → base64 PNG |
| screen_size | GET /session/{id}/window/size (logical points, multiply by scale) |

### WDA Lifecycle Management

```
mobile-use run (iOS device)
  1. Check if WDA running (GET /status)
     ├─ running → reuse
     └─ not running → start WDA
  2. Start WDA:
     - Simulator: install bundled WDA.app, launch via simctl
     - Real device: xcodebuild test-without-building with cached build
  3. Port forwarding (real device only: iproxy)
  4. Wait for /status → 200
  5. Create WDA session (POST /session)
  6. Ready
```

## WDA Distribution

### Simulator: bundled pre-built app

- Pre-compile WDA.app (x86_64 + arm64)
- Bundle with mobile-use (or download on first use)
- Install: `xcrun simctl install <udid> WDA.app`
- Zero user configuration required

### Real device: user runs setup once

```bash
mobile-use setup-ios --device <UDID>
```

This command:
1. Downloads/extracts bundled WDA Xcode project
2. Runs `xcodebuild build-for-testing -allowProvisioningUpdates`
3. User's Apple ID handles signing automatically
4. Caches build artifacts to `~/.cache/mobile-use/wda/`
5. Installs WDA to device

Signing validity: 7 days (free Apple ID) or 1 year (paid developer). User re-runs setup when expired.

## Integration Changes

### DaemonSession

```rust
pub struct DaemonSession {
    pub name: String,
    pub device: Option<String>,
    pub vm_url: Option<String>,
    pub vm_service: VmServiceClient,
    pub device_op: Box<dyn DeviceOperator>,  // replaces `adb` field
    pub platform: Platform,
    pub ref_map: RefMap,
    pub has_flutter_process: bool,
    pub package: Option<String>,
}
```

### Daemon Protocol

```rust
pub enum DaemonRequest {
    // existing
    Connect { session, device, url, port },
    ConnectAndroid { session, device, package },

    // new
    ConnectIOS {
        session: String,
        device: Option<String>,
        simulator: bool,
        wda_port: Option<u16>,  // default 8100
    },
}
```

### CLI Changes

Project type detection:
```rust
enum ProjectType {
    FlutterAndroid,
    FlutterIOS,        // new
    NativeAndroid,
    Unknown,
}
```

`devices` command lists both platforms:
- Android: `adb devices`
- iOS simulators: `xcrun simctl list devices booted -j`
- iOS real devices: `idevice_id -l` (if libimobiledevice installed)

`run` command auto-detects platform from device ID format or flutter target.

### Cargo.toml

```toml
# new dependency
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

System tool dependencies:
- Simulator: Xcode Command Line Tools + iOS Simulator (macOS developers typically have this)
- Real device: `libimobiledevice` (`brew install libimobiledevice`), Apple ID

## Agent Experience

The agent sees no platform difference:

```bash
mobile-use devices                    # lists Android + iOS
mobile-use run -- -d <any_device>     # auto-detects platform
mobile-use elements                   # unified element tree
mobile-use tap @e1                    # same command
mobile-use screenshot screen.png      # same command
```

## Implementation Phases

### Phase 1: Core abstraction + WDA client

1. Define DeviceOperator trait in `src/platform/mod.rs`
2. Implement DeviceOperator for existing AdbClient
3. Implement WdaClient (HTTP, W3C Actions)
4. Implement IOSDevice (delegates to WdaClient)
5. Refactor DaemonSession to use `Box<dyn DeviceOperator>`

### Phase 2: Simulator support (zero-config)

1. iOS device discovery (simctl list)
2. WDA manager for simulator (install bundled app, launch, health check)
3. CLI: `devices` shows iOS simulators
4. CLI: `run` supports iOS simulator target
5. Bundle pre-built WDA.app

### Phase 3: Real device support

1. `setup-ios` command (xcodebuild + signing)
2. WDA manager for real device (iproxy, xcodebuild test-without-building)
3. iOS real device discovery (idevice_id)
4. Port forwarding via iproxy

### Phase 4: Robustness (learned from hawk_agent-rs)

1. Structured `WdaError` type — parse WDA error responses (error, message, traceback)
2. WDA port pool — global singleton, ports 8100-8199, RAII auto-release via `PortGuard`
3. `iproxy` wrapper — find binary, spawn with auto-kill on drop, health check
4. System dialog handling — detect permission popups, auto-dismiss with preferred buttons (EN/CN)
5. Device orientation — query and set via WDA `/orientation` endpoint

### Phase 5: Verification & Polish

1. Verify test_app runs on iOS simulator (Material Design, identical UI)
2. End-to-end test: devices → run → elements → tap → screenshot on iOS
3. Error messages for common issues (WDA not installed, signing expired)
4. Documentation update

## Reference

- hawk_agent-rs iOS implementation: `/Users/liuqijun/scm/netease/hawk/hawk_agent-rs/crates/device/src/ios/`
- WebDriverAgent W3C Actions spec: https://www.w3.org/TR/webdriver/#actions
- Appium WDA: https://github.com/appium/WebDriverAgent
