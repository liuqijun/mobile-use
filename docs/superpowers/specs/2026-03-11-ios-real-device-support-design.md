# iOS Real Device Support Design

## Goal

Add iOS real device support to mobile-use, enabling the same CLI commands (tap, text, elements, screenshot, etc.) to work transparently on iOS devices via WebDriverAgent (WDA).

## Architecture

### DeviceOperator Trait

Extract a `DeviceOperator` trait to abstract platform-specific device operations. Both `AdbClient` (Android) and `WdaClient` (iOS) implement this trait.

```rust
pub enum Platform { Android, IOS }

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

### Session Changes

`DaemonSession` replaces `adb: AdbClient` with `device_op: Box<dyn DeviceOperator>`:

```rust
pub struct DaemonSession {
    pub device_op: Box<dyn DeviceOperator>,  // replaces adb: AdbClient
    pub platform: Platform,
    // ... other fields unchanged
}
```

### New Modules

```
src/platform/ios/
  mod.rs          - Module exports
  wda.rs          - WdaClient: HTTP client for WDA (W3C Actions API)
  wda_manager.rs  - WDA lifecycle: build, install, launch, health check
  discovery.rs    - iOS device discovery (idevice_id)
  elements.rs     - Parse WDA /source XML into ElementNode tree
```

## Implementation Phases

### Phase 1: Core Abstraction

1. Add `reqwest` dependency (blocking, json features)
2. Define `DeviceOperator` trait and `Platform` enum in `src/core/types.rs`
3. Implement `DeviceOperator` for `AdbClient` (adapter, no behavior change)
4. Refactor `DaemonSession`: `adb` -> `device_op: Box<dyn DeviceOperator>`
5. Update all action functions in `main.rs` to use `session.device_op.*()` instead of `session.adb.*()`
6. Verify all existing Android tests still pass

### Phase 2: WDA Real Device Support

1. **WdaClient** (`src/platform/ios/wda.rs`):
   - HTTP client using `reqwest::blocking`
   - Create/destroy WDA sessions
   - Detect device scale factor (logical points -> physical pixels)
   - Implement `DeviceOperator` trait methods via W3C Actions API:
     - tap -> pointerMove + pointerDown + pointerUp
     - double_tap -> two tap sequences
     - long_press -> pointerDown + pause(duration) + pointerUp
     - swipe -> pointerDown + pause + pointerMove + pointerUp
     - input_text -> /wda/keys endpoint
     - screenshot -> /screenshot endpoint (base64 PNG)
   - Coordinates: accept physical pixels, convert to logical points internally

2. **WDA Manager** (`src/platform/ios/wda_manager.rs`):
   - `setup-ios` CLI command: clone WDA repo, xcodebuild with user's signing team
   - Launch WDA on device via `xcodebuild test`
   - Port forwarding via `iproxy` (device WDA port -> localhost)
   - Health check via `GET /status`

3. **Daemon Protocol**:
   - Add `ConnectIOS { session, device, wda_port }` to `DaemonRequest`
   - Handle in `session_manager.rs`: create session with `WdaClient` as `device_op`

4. **CLI**:
   - Add `setup-ios` command to `parser.rs`
   - Extend `connect` command to accept iOS devices
   - Extend `devices` command to list iOS devices via `idevice_id`

### Phase 3: iOS Element Tree

1. **Elements parsing** (`src/platform/ios/elements.rs`):
   - Fetch accessibility tree from WDA `GET /source?format=xml`
   - Parse XML into `ElementNode` tree with matching structure
   - Map WDA element types to unified types (XCUIElementTypeButton -> "Button", etc.)
   - Extract bounds, labels, accessibility identifiers
   - Coordinate conversion (points -> physical pixels using scale factor)

2. Integrate into `elements` command in `main.rs`

## Key Design Decisions

- **Synchronous API**: WdaClient uses `reqwest::blocking` to match AdbClient's synchronous interface
- **Physical pixels everywhere**: All coordinates in the public API use physical pixels; WdaClient converts internally
- **Real device first**: Skip simulator support initially, add later
- **WDA as external dependency**: User runs `setup-ios` once to build and install WDA on their device

## Environment Requirements

- Xcode (for building WDA)
- libimobiledevice (`idevice_id`, `iproxy`)
- Apple Developer certificate (for code signing WDA onto real device)
