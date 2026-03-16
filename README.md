# mobile-use

Mobile UI automation from the command line. Inspect, interact with, and automate mobile app interfaces.

## Supported Platforms

| Mode | Target | Requirements | Build Type |
|------|--------|-------------|------------|
| **Flutter** (recommended) | Flutter apps | Flutter SDK, ADB | Debug / Profile |
| **Android** | Any Android app | ADB | Any (debug, release) |
| **iOS** | iOS apps | Xcode, libimobiledevice | Any |

- **Flutter mode** connects to the Flutter VM Service via WebSocket, providing a rich semantics tree with element types, labels, bounds, and style information. The VM Service is only available in debug and profile builds — release builds are not supported.
- **Android mode** uses ADB + uiautomator, works with any Android app regardless of build type.
- **iOS mode** uses WebDriverAgent (WDA) for real device automation. Requires one-time setup per device via `setup-ios`. Supports element tree, tap, text input, scroll, swipe, and screenshot.
- **Host OS**: macOS, Linux (requires ADB in PATH). iOS support requires macOS with Xcode.

## Installation

```bash
# macOS (Homebrew)
brew install liuqijun/tap/mobile-use

# From crates.io
cargo install mobile-use

# From source
git clone https://github.com/liuqijun/mobile-use.git
cd mobile-use
cargo install --path .
```

## Quick Start

```bash
# 1. List connected devices
mobile-use devices

# 2. Run Flutter app (auto-connects)
cd your_flutter_app
mobile-use run -- -d emulator-5554

# 3. Get UI elements (in another terminal)
mobile-use elements

# 4. Interact with elements
mobile-use tap @e1
mobile-use text @e2 "hello@example.com"
mobile-use screenshot output.png

# 5. Disconnect
mobile-use disconnect
```

### iOS Quick Start

```bash
# 1. Install dependencies
brew install libimobiledevice

# 2. List devices (find your iOS device UDID)
mobile-use devices

# 3. One-time setup: build & install WebDriverAgent
mobile-use -d DEVICE_UDID setup-ios --team-id YOUR_TEAM_ID

# 4. Connect to device
mobile-use -d DEVICE_UDID connect-ios --team-id YOUR_TEAM_ID

# 5. Use the same commands as Android/Flutter
mobile-use elements
mobile-use tap @e1
mobile-use screenshot ios.png

# 6. Disconnect
mobile-use disconnect
```

## Architecture

```
CLI Command → Unix Socket → Daemon Process → WebSocket → Flutter VM Service
                              ↓
                         Session Manager (in-memory state)
```

- **CLI**: Stateless command-line interface. Each command sends a request and prints the response.
- **Daemon**: Background process (`~/.cache/mobile-use/daemon.sock`) that maintains persistent WebSocket connections and session state. Auto-starts on first command.
- **Session Manager**: Manages multiple simultaneous app connections, each identified by a session name.

---

## Global Options

Every command accepts these options:

| Option | Description | Example |
|--------|-------------|---------|
| `-d, --device <ID>` | Device identifier (USB serial or IP:port) | `-d emulator-5554` |
| `-s, --session <NAME>` | Session name (default: "default") | `-s app1` |
| `--json` | Output in JSON format | `--json` |
| `-h, --help` | Show help | `--help` |
| `-V, --version` | Show version | `--version` |

### Device Identifiers

```bash
-d emulator-5554           # Android emulator
-d 1234567890ABCDEF        # USB-connected Android device
-d 192.168.1.100:5555      # Wireless ADB connection
-d 00008030-001A2D410EFA002E  # iOS device UDID
```

### Multi-Session Support

Control multiple apps simultaneously using session names:

```bash
mobile-use -s app1 connect --package com.example.app1
mobile-use -s app2 connect --package com.example.app2
mobile-use -s app1 elements    # Get elements from app1
mobile-use -s app2 tap @e1     # Tap in app2
```

---

## Element References

Element references (`@e1`, `@e2`, etc.) are temporary identifiers assigned to UI elements by the `elements` command. They are used to target elements in interaction commands.

### Getting Element References

```bash
mobile-use elements
```

Output:
```
@e1 [button] "Login" (100,200 300x50)
  @e2 [text] "Username" (110,210 280x30)
@e3 [textfield] "Password" (100,260 300x50)
```

Each line contains:
- `@e1` — Element reference ID (use in subsequent commands)
- `[button]` — Element type
- `"Login"` — Element label/text
- `(100,200 300x50)` — Position (x,y) and size (width x height) in pixels

### Lifecycle

**Element refs are ephemeral.** After any UI change (navigation, tap, text input), refs may point to different elements. Always re-fetch before operating:

```bash
mobile-use elements      # @e1 = "Login" button
mobile-use tap @e1       # UI changes to next screen
mobile-use elements      # Must re-fetch! @e1 is now a different element
mobile-use tap @e3       # Now safe to use new refs
```

---

## Commands Reference

### Connection Management

#### `run` — Run Application

Runs and auto-connects to a mobile application.

**Flutter mode** (default — runs in a Flutter project directory):
```bash
mobile-use run                        # Run in current Flutter project
mobile-use run -- -d emulator-5554    # Specify device
mobile-use run -- --flavor prod       # Specify flavor
mobile-use run -- --dart-define=ENV=staging
```

Wraps `flutter run --machine`, parses JSON output to extract VM Service URL, and auto-connects. Supports interactive input: `r` (hot reload), `R` (hot restart), `q` (quit).

**Android mode**:
```bash
mobile-use run app.apk                      # Install and run APK
mobile-use run --package com.example.app    # Launch installed package
```

| Parameter | Description |
|-----------|-------------|
| `[APK]` | APK file path (Android mode) |
| `--package <NAME>` | Android package name |
| `[ARGS]...` | Arguments passed to `flutter run` (after `--` separator) |

---

#### `connect` — Connect to Running App

Connect to an already-running application.

**Flutter mode**:
```bash
mobile-use connect --url ws://127.0.0.1:55370/abc123=/ws
mobile-use connect --port 55370    # Auto-discover URL on port
```

**Android mode**:
```bash
mobile-use connect --package com.example.app
```

| Parameter | Description |
|-----------|-------------|
| `--url <URL>` | Flutter VM Service WebSocket URL |
| `--port <PORT>` | Port number for auto-discovery |
| `--package <NAME>` | Android package name |

**URL conversion** (from `flutter run` output):
```
flutter run shows:  http://127.0.0.1:55370/abc123=/
mobile-use needs:   ws://127.0.0.1:55370/abc123=/ws
                    ↑                              ↑
                 http→ws                     append /ws
```

---

#### `disconnect` — Disconnect from App

```bash
mobile-use disconnect
```

Closes WebSocket connection (Flutter) or clears package binding (Android).

---

#### `info` — Show Connection Info

```bash
mobile-use info
```

Output:
```
Mode: Flutter
Device: emulator-5554
Session: default
VM Service: ws://127.0.0.1:55370/abc=/ws
```

---

#### `devices` — List Connected Devices

```bash
mobile-use devices
```

Lists Android devices (via ADB) and iOS devices (via libimobiledevice).

Output:
```
Found 3 device(s):

  [1] emulator-5554
      Model:   Android SDK built for x86_64 (Google)
      Android: 13 (SDK 33)
      Screen:  1080x2400

  [2] 192.168.1.100:5555
      Model:   Mi 10 (Xiaomi)
      Android: 13 (SDK 33)
      Screen:  1080x2340

  [3] 00008030-001A2D410EFA002E
      Name:    iPhone 15 Pro
      iOS:     17.4
```

---

### Element Interaction

#### `elements` — Get UI Element Tree

```bash
mobile-use elements           # All elements
mobile-use elements -i        # Interactive elements only (buttons, text fields, etc.)
mobile-use elements --json    # JSON output
```

| Option | Description |
|--------|-------------|
| `-i, --interactive` | Show only interactive elements (buttons, text fields, checkboxes, switches, sliders, links) |

Output format:
```
@e1 [button] "Login" (100,200 300x50)
  @e2 [text] "Username" (110,210 280x30)
@e3 [textfield] "Password" (100,260 300x50)
```

---

#### `tap` — Tap Element

```bash
mobile-use tap <REFERENCE>
```

Performs a single tap at the center of the element's bounds.

```bash
mobile-use tap @e1
mobile-use tap @e3
```

---

#### `double-tap` — Double Tap Element

```bash
mobile-use double-tap <REFERENCE>
```

Two rapid taps with 50ms interval. Useful for text selection or zoom gestures.

```bash
mobile-use double-tap @e1
```

---

#### `long-press` — Long Press Element

```bash
mobile-use long-press [OPTIONS] <REFERENCE>
```

| Option | Description | Default |
|--------|-------------|---------|
| `--duration <MS>` | Hold duration in milliseconds | 500 |

```bash
mobile-use long-press @e1                  # 500ms hold
mobile-use long-press @e1 --duration 1000  # 1 second hold
```

---

#### `text` — Input Text

```bash
mobile-use text [OPTIONS] <REFERENCE> <TEXT>
```

| Option | Description |
|--------|-------------|
| `--clear` | Clear existing text before input |

```bash
mobile-use text @e2 "hello world"        # Input text
mobile-use text @e2 "new text" --clear   # Clear then input
```

**Important**: This command automatically taps the element to focus it. Do NOT call `tap` before `text`.

---

#### `clear` — Clear Text Field

```bash
mobile-use clear <REFERENCE>
```

Clears all text from a text field. Automatically taps to focus, moves to end, and backspace-deletes (up to 50 characters).

```bash
mobile-use clear @e2
```

---

### Navigation

#### `scroll` — Scroll Screen

```bash
mobile-use scroll <DIRECTION> [DISTANCE]
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `<DIRECTION>` | `up`, `down`, `left`, `right` | — |
| `[DISTANCE]` | Distance in pixels | 300 |

```bash
mobile-use scroll down         # Scroll down 300px
mobile-use scroll down 500     # Scroll down 500px
mobile-use scroll up 200       # Scroll up 200px
mobile-use scroll left         # Scroll left 300px
```

**Direction semantics** (matches content movement, not finger movement):
- `down` — Shows content below (finger swipes up)
- `up` — Shows content above (finger swipes down)
- `left` — Shows content to the right (finger swipes left)
- `right` — Shows content to the left (finger swipes right)

Scrolls from screen center, duration 300ms.

---

#### `scroll-to` — Scroll Element Into View

```bash
mobile-use scroll-to <REFERENCE>
```

Repeatedly scrolls and checks visibility until the element appears.

```bash
mobile-use scroll-to @e15
```

> **Note**: This command is not yet implemented.

---

#### `swipe` — Swipe Gesture

```bash
mobile-use swipe [OPTIONS] <DIRECTION>
```

| Option | Description |
|--------|-------------|
| `--from <REF>` | Starting element (default: screen center) |

```bash
mobile-use swipe left              # Swipe left from center
mobile-use swipe right --from @e5  # Swipe right from element @e5
```

**vs. `scroll`**: Use `swipe` for UI gestures (dismissing cards, pull-to-refresh, carousel). Use `scroll` for scrolling content. Swipe distance is fixed at 500px, duration 200ms.

---

### Query & Wait

#### `get` — Get Element Property

```bash
mobile-use get <PROPERTY> <REFERENCE>
```

| Property | Description | Example Output |
|----------|-------------|----------------|
| `text` | Element's text content | `Login` |
| `type` | Element type | `button` |
| `bounds` | Coordinates and size | `{x:100, y:200, width:300, height:50}` |
| `prop` | All semantic properties (JSON) | `{"focusable":true, ...}` |
| `<custom>` | Any key from properties map | (varies) |

```bash
mobile-use get text @e3      # "Login"
mobile-use get type @e3      # "button"
mobile-use get bounds @e3    # {x:100, y:200, width:300, height:50}
mobile-use get prop @e3      # Full properties JSON
```

---

#### `is` — Check Element State

```bash
mobile-use is <STATE> <REFERENCE>
```

Returns `true` or `false`.

| State | Description |
|-------|-------------|
| `visible` | Element is displayed on screen |
| `enabled` | Element is interactable (not disabled) |
| `checked` | Element is checked (checkboxes, switches) |
| `focused` | Element has input focus |

```bash
mobile-use is visible @e3    # true
mobile-use is enabled @e3    # true
mobile-use is checked @e5    # false
mobile-use is focused @e2    # true
```

---

#### `wait` — Wait for Condition

```bash
mobile-use wait [OPTIONS] [TARGET]
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `[TARGET]` | Element reference or milliseconds | — |
| `--text <TEXT>` | Wait for text to appear on screen | — |
| `--timeout <MS>` | Maximum wait time | 30000 |

**Wait for element**:
```bash
mobile-use wait @e5                  # Wait for @e5 to appear
mobile-use wait @e5 --timeout 5000   # Wait up to 5 seconds
```

**Wait for text**:
```bash
mobile-use wait --text "Success"     # Wait for "Success" on screen
```

**Wait fixed time**:
```bash
mobile-use wait 2000                 # Wait 2 seconds
```

Element/text wait polls every 500ms. Default timeout is 30 seconds.

---

### Capture

#### `screenshot` — Take Screenshot

```bash
mobile-use screenshot [PATH]
```

```bash
mobile-use screenshot                  # screenshot-<timestamp>.png
mobile-use screenshot output.png       # output.png
mobile-use screenshot /tmp/screen.png  # Absolute path
```

Uses ADB screencap. Saves as PNG.

---

### Flutter Commands

Requires a Flutter app connection via VM Service (debug/profile builds only).

#### `flutter reload` — Hot Reload

```bash
mobile-use flutter reload
```

Applies code changes while preserving app state. Same as pressing `r` in the flutter run terminal.

#### `flutter restart` — Hot Restart

```bash
mobile-use flutter restart
```

Restarts the app from scratch, losing all state. Same as pressing `R`.

#### `flutter widgets` — Get Widget Tree

```bash
mobile-use flutter widgets
```

Returns the raw Flutter widget render tree. Very verbose — prefer `elements` for automation.

---

### iOS Setup & Automation

#### `setup-ios` — Setup iOS Automation

Build and install WebDriverAgent on an iOS device. Run once per device before iOS automation.

**Requirements**:
- macOS with Xcode installed
- Apple Developer account (free or paid)
- Device connected via USB
- libimobiledevice (`brew install libimobiledevice`)

| Parameter | Description |
|-----------|-------------|
| `--team-id <ID>` | Apple Developer Team ID (10-character alphanumeric) **[required]** |

```bash
mobile-use setup-ios --team-id YOUR_TEAM_ID
mobile-use -d DEVICE_UDID setup-ios --team-id YOUR_TEAM_ID
```

---

#### `connect-ios` — Connect to iOS Device

Launches WebDriverAgent on the device and establishes connection. Requires `setup-ios` to have been run first.

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--team-id <ID>` | Apple Developer Team ID **[required]** | — |
| `--port <PORT>` | WDA port | 8100 |

```bash
mobile-use connect-ios --team-id YOUR_TEAM_ID
mobile-use -d DEVICE_UDID connect-ios --team-id YOUR_TEAM_ID --port 8100
```

After connecting, use the same commands as Flutter/Android mode:

```bash
mobile-use elements           # Get iOS element tree
mobile-use tap @e1            # Tap element
mobile-use text @e2 "hello"   # Input text
mobile-use screenshot ios.png # Take screenshot
```

---

### Daemon Management

#### `daemon start` / `stop` / `status`

```bash
mobile-use daemon start     # Start daemon (usually auto-starts)
mobile-use daemon stop      # Stop daemon, close all connections
mobile-use daemon status    # Show daemon status
mobile-use stop             # Alias for daemon stop
```

Daemon socket: `~/.cache/mobile-use/daemon.sock`

---

#### `quit` — Stop App / Full Reset

```bash
mobile-use quit              # Stop current session's flutter process
mobile-use -s app1 quit      # Stop specific session
mobile-use quit --all        # Full reset
```

| Option | Description |
|--------|-------------|
| `--all` | Kill all processes, stop daemon, remove all state files |

**`quit --all` performs:**
1. Kill all `mobile-use run` processes (all sessions)
2. Kill orphaned `flutter run --machine` processes
3. Stop WDA and iproxy processes (iOS)
4. Stop the daemon
5. Delete all PID files and socket files
6. Clear legacy session files

Use `quit --all` when mobile-use is in an inconsistent state (crashed processes, stale PID files).

---

## JSON Output

All commands support `--json` for programmatic parsing:

```bash
mobile-use elements --json
mobile-use get text @e3 --json
mobile-use is visible @e3 --json
mobile-use devices --json
```

---

## Automation Examples

### Login Flow

```bash
#!/bin/bash

# Start app
mobile-use run -- -d emulator-5554 &
sleep 10

# Wait for login page
mobile-use wait --text "Login"

# Get elements
mobile-use elements

# Input credentials
mobile-use text @e2 "username"
mobile-use text @e3 "password"

# Tap login button
mobile-use tap @e4

# Wait for success
mobile-use wait --text "Welcome" --timeout 10000

# Screenshot
mobile-use screenshot login_success.png
```

### Scroll to Find Item

```bash
#!/bin/bash

for i in {1..10}; do
    mobile-use elements | grep "Target Item" && break
    mobile-use scroll down 500
    sleep 0.5
done

mobile-use tap @e15
```

### Form Validation

```bash
#!/bin/bash

mobile-use elements -i

# Fill form
mobile-use text @e1 "John Doe"
mobile-use text @e2 "john@example.com"
mobile-use text @e3 "password123"

# Check submit button state
mobile-use is enabled @e4    # true

# Submit
mobile-use tap @e4

# Verify result
mobile-use wait --text "Success" --timeout 5000
mobile-use screenshot form_result.png
```

---

## Troubleshooting

### Connection Issues

```bash
# Check device connection
adb devices

# Check daemon status
mobile-use daemon status

# Restart daemon
mobile-use daemon stop
mobile-use daemon start

# Full reset when state is inconsistent
mobile-use quit --all
```

### Elements Not Found

```bash
# Use -i for interactive elements only
mobile-use elements -i

# Use --json for detailed info
mobile-use elements --json
```

### Wireless ADB Limitations

Some operations (`scroll`, `swipe`) may fail with `INJECT_EVENTS` permission errors over wireless ADB. Use USB connection for full functionality.

### iOS Issues

```bash
# Device not detected - install libimobiledevice
brew install libimobiledevice

# WDA build fails - check Xcode and team ID
xcode-select -p                    # Ensure Xcode CLI tools are set
mobile-use setup-ios --team-id YOUR_TEAM_ID  # Re-run setup

# Connection fails after setup
mobile-use quit --all              # Full reset
mobile-use -d UDID connect-ios --team-id YOUR_TEAM_ID
```

### VM Service URL Changes

Each `flutter run` generates a new VM Service token. If the app restarts, you must reconnect with the new URL. Using `mobile-use run` handles this automatically.

---

## Command Status

| Command | Status | Notes |
|---------|--------|-------|
| `run` | Stable | Flutter mode complete, Android basic |
| `connect` | Stable | Flutter complete, Android partial |
| `disconnect` | Stable | |
| `quit` | Stable | Includes `--all` full reset |
| `elements` | Stable | With style extraction |
| `tap` | Stable | |
| `double-tap` | Stable | |
| `long-press` | Stable | |
| `text` | Stable | Auto-focus |
| `clear` | Stable | |
| `screenshot` | Stable | |
| `scroll` | Stable | |
| `scroll-to` | Not implemented | |
| `swipe` | Stable | |
| `wait` | Stable | |
| `get` | Stable | |
| `is` | Stable | |
| `info` | Stable | |
| `devices` | Stable | |
| `flutter reload` | Stable | |
| `flutter restart` | Stable | |
| `flutter widgets` | Stable | |
| `daemon start/stop/status` | Stable | |
| `stop` | Stable | Alias for `daemon stop` |
| `setup-ios` | Stable | One-time per device |
| `connect-ios` | Stable | |

## License

MIT
