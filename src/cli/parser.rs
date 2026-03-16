use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon (usually auto-started)
    #[command(long_about = "Start the background daemon process.

Usually starts automatically on first command.
Manual start rarely needed.")]
    Start,

    /// Stop the daemon
    #[command(long_about = "Stop the background daemon and close all connections.

Use when:
- Freeing up resources
- Resetting connection state
- Before uninstalling mobile-use")]
    Stop,

    /// Show daemon status
    #[command(long_about = "Check if daemon is running and show details.

Shows:
- Running/stopped status
- Socket path
- Active sessions")]
    Status,
}

#[derive(Subcommand, Debug)]
pub enum FlutterCommands {
    /// Hot reload the app
    #[command(long_about = "Perform Flutter hot reload.

Applies code changes while preserving app state.
Same as pressing 'r' in flutter run terminal.

Fast iteration: change code, run 'mobile-use flutter reload', see changes.")]
    Reload,

    /// Hot restart the app
    #[command(long_about = "Perform Flutter hot restart.

Restarts the app from scratch, losing all state.
Same as pressing 'R' in flutter run terminal.

Use when hot reload doesn't pick up changes (e.g., initial state changes).")]
    Restart,

    /// Get widget tree
    #[command(long_about = "Get the Flutter widget tree.

Returns raw render tree from Flutter VM Service.
Very verbose - prefer 'elements' command for automation.

Useful for debugging widget hierarchy issues.")]
    Widgets,
}

#[derive(Parser, Debug)]
#[command(name = "mobile-use")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Cross-platform UI automation CLI for AI agents")]
#[command(long_about = "Cross-platform UI automation CLI for AI agents.

mobile-use enables AI agents to interact with mobile applications through a simple command-line interface.
It supports both Flutter apps (via VM Service) and native Android apps (via ADB/uiautomator).

WORKFLOW:
  1. Start your app: mobile-use run [options]
  2. Get UI elements: mobile-use elements
  3. Interact: mobile-use tap @e1, mobile-use text @e2 \"hello\"

ELEMENT REFERENCES:
  Elements are identified by refs like @e1, @e2. Get refs from 'elements' command.
  Refs are ephemeral - re-fetch after UI changes.

MODES:
  - Flutter mode: Uses VM Service for semantics tree (recommended)
  - Android mode: Uses uiautomator dump (fallback for non-Flutter apps)

EXAMPLES:
  mobile-use run -- -d emulator-5554    # Run Flutter app on device
  mobile-use elements                    # List all UI elements with refs
  mobile-use elements -i                 # List interactive elements only
  mobile-use tap @e3                     # Tap element @e3
  mobile-use text @e5 \"hello\"            # Input text to @e5
  mobile-use wait @e1                    # Wait for element to appear
  mobile-use screenshot output.png       # Take screenshot")]
pub struct Cli {
    /// Device identifier (e.g., emulator-5554, 192.168.1.100:5555)
    #[arg(short, long, help = "Device identifier",
        long_help = "Device identifier for ADB commands.

Can be:
- USB device serial: emulator-5554, 1234567890ABCDEF
- IP:port for wireless: 192.168.1.100:5555

List available devices: adb devices or mobile-use devices")]
    pub device: Option<String>,

    /// Session name for multi-app scenarios
    #[arg(short, long, default_value = "default",
        help = "Session name for multi-app scenarios",
        long_help = "Session name for managing multiple app connections.

Use different session names to control multiple apps simultaneously:
  mobile-use -s app1 connect --package com.example.app1
  mobile-use -s app2 connect --package com.example.app2
  mobile-use -s app1 elements  # Get elements from app1
  mobile-use -s app2 tap @e1   # Tap in app2")]
    pub session: String,

    /// Output in JSON format
    #[arg(long, global = true, default_value = "false",
        help = "Output in JSON format",
        long_help = "Output results in JSON format for programmatic parsing.

Useful for:
- Integrating with other tools
- Parsing element data programmatically
- Automation scripts")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run application (auto-detects Flutter or Android)
    #[command(long_about = "Run and connect to a mobile application.

FLUTTER MODE (default):
Runs 'flutter run --machine' in current directory, auto-connects to VM Service.
Supports hot reload (r), hot restart (R), and quit (q) via stdin.

  mobile-use run                        # Run in current Flutter project
  mobile-use run -- -d emulator-5554    # Specify device
  mobile-use run -- --release           # Run in release mode

ANDROID MODE:
Install and launch APK, connect via uiautomator.

  mobile-use run app.apk                      # Install and run APK
  mobile-use run --package com.example.app    # Launch installed package")]
    Run {
        /// APK file path for native Android apps
        #[arg(help = "APK file path",
            long_help = "Path to APK file for native Android apps.
If provided, installs APK via 'adb install' then launches it.
Not used for Flutter apps.")]
        apk: Option<String>,

        /// Package name to launch (for native Android)
        #[arg(long, help = "Package name to launch",
            long_help = "Android package name (e.g., com.example.app).
Required with APK, or use alone to launch already-installed app.")]
        package: Option<String>,

        /// Arguments to pass to flutter run (Flutter mode only)
        #[arg(last = true, help = "Flutter run arguments",
            long_help = "Arguments passed directly to 'flutter run'.
Must come after '--' separator.

Examples:
  mobile-use run -- -d emulator-5554
  mobile-use run -- --release --flavor prod
  mobile-use run -- --dart-define=ENV=staging")]
        args: Vec<String>,
    },

    /// Connect to target application
    #[command(long_about = "Connect to a running application for UI automation.

FLUTTER MODE:
Connect via WebSocket URL or port. URL from 'flutter run' output needs conversion:
  http://127.0.0.1:55370/abc123=/ -> ws://127.0.0.1:55370/abc123=/ws

  mobile-use connect --url ws://127.0.0.1:55370/abc123=/ws
  mobile-use connect --port 55370  # Auto-discover URL

ANDROID MODE:
Bind to package for uiautomator-based automation.

  mobile-use connect --package com.example.app

Note: Prefer 'mobile-use run' which handles connection automatically.")]
    Connect {
        /// WebSocket URL for Flutter VM Service
        #[arg(long, help = "WebSocket URL",
            long_help = "Flutter VM Service WebSocket URL.
Format: ws://host:port/token=/ws

Convert from flutter run output:
  http://127.0.0.1:55370/abc123=/ -> ws://127.0.0.1:55370/abc123=/ws")]
        url: Option<String>,

        /// Port number (Flutter mode)
        #[arg(long, help = "Port number",
            long_help = "Port for auto-discovering Flutter VM Service.
Attempts to find VM Service URL on this port.")]
        port: Option<u16>,

        /// Package name to bind (Android mode)
        #[arg(long, help = "Android package name",
            long_help = "Package name for native Android automation.
Uses uiautomator dump for element tree.")]
        package: Option<String>,
    },

    /// Disconnect from application
    #[command(long_about = "Disconnect from the currently connected application.

Closes WebSocket connection (Flutter) or clears package binding (Android).
Use before switching to a different app.")]
    Disconnect,

    /// Get element tree with UI details
    #[command(long_about = "Get the UI element tree from the connected application.

Returns hierarchical element tree with:
- ref_id: Reference for interaction (@e1, @e2, ...)
- element_type: button, textfield, text, image, etc.
- label: Accessible text/description
- bounds: Screen coordinates {x, y, width, height}
- properties: Additional semantic properties

OUTPUT FORMAT:
  @e1 [button] \"Login\" (100,200 300x50)
    @e2 [text] \"Username\" (110,210 280x30)

Use element refs with interaction commands:
  mobile-use tap @e1
  mobile-use text @e2 \"hello\"

FLUTTER MODE: Extracts from semantics tree with style info
ANDROID MODE: Uses uiautomator dump

EXAMPLE WORKFLOW:
  mobile-use elements           # Get all elements
  mobile-use elements -i        # Interactive elements only
  mobile-use tap @e3            # Tap the third element")]
    Elements {
        /// Show only interactive elements (buttons, text fields, etc.)
        #[arg(short, long, help = "Interactive elements only",
            long_help = "Filter to show only interactive elements.
Includes: buttons, text fields, checkboxes, switches, sliders, links.
Excludes: static text, images, containers.

Reduces output noise for automation tasks.")]
        interactive: bool,
    },

    /// Take screenshot
    #[command(long_about = "Take a screenshot of the current screen.

Saves screenshot to specified path or generates timestamped filename.
Uses ADB screencap internally.

Examples:
  mobile-use screenshot                  # Saves as screenshot_<timestamp>.png
  mobile-use screenshot output.png       # Saves as output.png
  mobile-use screenshot /tmp/screen.png  # Saves to specific path")]
    Screenshot {
        /// Output file path
        #[arg(help = "Output file path",
            long_help = "Path to save screenshot file.
If not specified, saves as screenshot_<timestamp>.png in current directory.
Supports .png format.")]
        path: Option<String>,
    },

    /// Tap on element
    #[command(long_about = "Tap on a UI element by its reference.

Performs a single tap at the center of the element's bounds.
Uses ADB input tap with coordinates from element bounds.

Example:
  mobile-use elements    # Get @e1, @e2, @e3...
  mobile-use tap @e1     # Tap on element @e1")]
    Tap {
        /// Element reference from 'elements' command (e.g., @e1)
        #[arg(help = "Element reference (e.g., @e1)")]
        reference: String,
    },

    /// Double tap on element
    #[command(long_about = "Double tap on a UI element.

Performs two quick taps at the element's center.
Useful for text selection or zoom gestures.

Example:
  mobile-use double-tap @e1")]
    DoubleTap {
        /// Element reference from 'elements' command
        #[arg(help = "Element reference (e.g., @e1)")]
        reference: String,
    },

    /// Long press on element
    #[command(long_about = "Long press (touch and hold) on a UI element.

Holds touch for specified duration. Useful for:
- Context menus
- Drag operations
- Delete confirmations

Default duration: 500ms

Example:
  mobile-use long-press @e1              # 500ms hold
  mobile-use long-press @e1 --duration 1000  # 1 second hold")]
    LongPress {
        /// Element reference
        #[arg(help = "Element reference (e.g., @e1)")]
        reference: String,

        /// Hold duration in milliseconds
        #[arg(long, default_value = "500", help = "Duration in ms",
            long_help = "How long to hold the touch in milliseconds.
Default: 500ms. Increase for drag operations or stubborn context menus.")]
        duration: u32,
    },

    /// Input text into element
    #[command(long_about = "Input text into a text field element.

Automatically taps the element first to focus, then inputs text.
Do NOT tap before this command - it handles focus automatically.

Example:
  mobile-use text @e2 \"hello world\"       # Input text
  mobile-use text @e2 \"new text\" --clear  # Clear first, then input

Note: Special characters are passed via ADB input.")]
    Text {
        /// Element reference (must be a text field)
        #[arg(help = "Element reference (e.g., @e2)")]
        reference: String,

        /// Text to input
        #[arg(help = "Text to input",
            long_help = "Text string to input.
Special characters may need escaping.
For passwords or sensitive input, the text is passed via ADB input.")]
        text: String,

        /// Clear existing text before input
        #[arg(long, help = "Clear before input",
            long_help = "Clear existing text before inputting new text.
Selects all existing text and replaces with new input.")]
        clear: bool,
    },

    /// Clear element content
    #[command(long_about = "Clear all text from a text field element.

Focuses the element, selects all text, and deletes it.

Example:
  mobile-use clear @e2")]
    Clear {
        /// Element reference (must be a text field)
        #[arg(help = "Element reference (e.g., @e2)")]
        reference: String,
    },

    /// Scroll in direction
    #[command(long_about = "Scroll the screen in a direction.

Performs a swipe gesture in the opposite direction to scroll content.
'scroll down' swipes up to show content below.

Directions: up, down, left, right

Example:
  mobile-use scroll down         # Scroll down by 300px (default)
  mobile-use scroll down 500     # Scroll down by 500px
  mobile-use scroll up 200       # Scroll up by 200px")]
    Scroll {
        /// Direction to scroll: up, down, left, right
        #[arg(help = "Scroll direction",
            long_help = "Direction to scroll content.
- down: Show content below (swipes up)
- up: Show content above (swipes down)
- left: Show content to the right (swipes left)
- right: Show content to the left (swipes right)")]
        direction: String,

        /// Distance in pixels
        #[arg(default_value = "300", help = "Distance in pixels",
            long_help = "How far to scroll in pixels.
Default: 300px. Increase for faster scrolling through long lists.")]
        distance: i32,
    },

    /// Scroll element into view
    #[command(long_about = "Scroll until a specific element becomes visible.

Repeatedly scrolls and checks element visibility.
Useful for finding elements in long lists.

Example:
  mobile-use scroll-to @e15   # Scroll until @e15 is visible")]
    ScrollTo {
        /// Element reference to scroll into view
        #[arg(help = "Element reference (e.g., @e15)")]
        reference: String,
    },

    /// Swipe gesture
    #[command(long_about = "Perform a swipe gesture on the screen.

Unlike scroll, swipe is for UI gestures like:
- Dismissing cards
- Pull-to-refresh
- Carousel navigation

Example:
  mobile-use swipe left                # Swipe from center
  mobile-use swipe right --from @e5    # Swipe starting from element")]
    Swipe {
        /// Swipe direction: up, down, left, right
        #[arg(help = "Swipe direction")]
        direction: String,

        /// Starting element (optional, defaults to screen center)
        #[arg(long, help = "Start from element",
            long_help = "Element to start swipe from.
If not specified, swipes from screen center.")]
        from: Option<String>,
    },

    /// Wait for condition
    #[command(long_about = "Wait for a condition before continuing.

WAIT FOR ELEMENT:
  mobile-use wait @e5                 # Wait for @e5 to appear
  mobile-use wait @e5 --timeout 5000  # Wait up to 5 seconds

WAIT FOR TEXT:
  mobile-use wait --text \"Success\"   # Wait for text to appear

WAIT FIXED TIME:
  mobile-use wait 2000                # Wait 2000 milliseconds

Useful in automation scripts between actions.")]
    Wait {
        /// Element reference OR milliseconds to wait
        #[arg(help = "Element ref or milliseconds",
            long_help = "What to wait for:
- Element ref (@e5): Wait for element to appear
- Number (2000): Wait fixed milliseconds")]
        target: Option<String>,

        /// Wait for text to appear anywhere on screen
        #[arg(long, help = "Wait for text",
            long_help = "Text string to wait for.
Scans element tree for matching text.")]
        text: Option<String>,

        /// Maximum time to wait in milliseconds
        #[arg(long, default_value = "30000", help = "Timeout in ms",
            long_help = "Maximum wait time in milliseconds.
Default: 30000 (30 seconds). Returns error if timeout exceeded.")]
        timeout: u32,
    },

    /// Get element property
    #[command(long_about = "Get a specific property from an element.

PROPERTY TYPES:
  text   - Get element's text content
  type   - Get element type (button, textfield, etc.)
  prop   - Get all properties as JSON
  bounds - Get element bounds {x, y, width, height}
  <key>  - Get any custom key from properties map

Example:
  mobile-use get text @e3      # Get text: \"Login\"
  mobile-use get type @e3      # Get type: \"button\"
  mobile-use get bounds @e3    # Get: {x:100, y:200, width:300, height:50}
  mobile-use get prop @e3      # Get all properties as JSON")]
    Get {
        /// Property type: text, type, prop, bounds, or custom key
        #[arg(help = "Property type",
            long_help = "Which property to retrieve:
- text: Element's displayed text
- type: Element type (button, textfield, etc.)
- prop: All semantic properties (JSON)
- bounds: Screen coordinates {x, y, width, height}
- <key>: Any custom key from the element's properties map")]
        property: String,

        /// Element reference
        #[arg(help = "Element reference (e.g., @e3)")]
        reference: String,
    },

    /// Check element state
    #[command(long_about = "Check the state of an element.

STATE TYPES:
  visible - Check if element is on screen
  enabled - Check if element is interactable
  checked - Check if element is checked (checkboxes, switches)
  focused - Check if element has input focus

Returns: true/false

Example:
  mobile-use is visible @e3   # Check if @e3 is visible
  mobile-use is enabled @e3   # Check if @e3 is enabled
  mobile-use is checked @e5   # Check if @e5 is checked
  mobile-use is focused @e2   # Check if @e2 has focus

Useful for conditional logic in automation.")]
    Is {
        /// State to check: visible, enabled, checked, focused
        #[arg(help = "State to check",
            long_help = "Which state to check:
- visible: Element is displayed on screen
- enabled: Element can be interacted with
- checked: Element is checked (checkboxes, switches)
- focused: Element has input focus")]
        state: String,

        /// Element reference
        #[arg(help = "Element reference (e.g., @e3)")]
        reference: String,
    },

    /// Show connection info
    #[command(long_about = "Show current connection status and details.

Displays:
- Connection mode (Flutter/Android)
- Device identifier
- Session name
- Flutter VM Service URL (if Flutter mode)
- Package name (if Android mode)

Example output:
  Mode: Flutter
  Device: emulator-5554
  Session: default
  VM Service: ws://127.0.0.1:55370/abc=/ws")]
    Info,

    /// List connected devices
    /// List connected devices (Android + iOS)
    #[command(long_about = "List all connected devices (Android + iOS).

Shows devices available for automation.
Android: via ADB (equivalent to 'adb devices')
iOS: via libimobiledevice (requires 'brew install libimobiledevice')

Use device ID with -d flag:
  mobile-use -d emulator-5554 run              # Android
  mobile-use -d UDID setup-ios --team-id ID    # iOS setup
  mobile-use -d UDID connect-ios --team-id ID  # iOS connect")]
    Devices,

    /// Setup iOS automation (build & install WebDriverAgent)
    #[command(name = "setup-ios", long_about = "Build and install WebDriverAgent on an iOS device.

Downloads the WebDriverAgent project and builds it with your Apple Developer certificate.
Run this once per device before using iOS automation.

Requires:
- Xcode installed
- Apple Developer account (free or paid)
- Device connected via USB
- libimobiledevice (brew install libimobiledevice)

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

    /// Daemon management
    #[command(subcommand)]
    #[command(long_about = "Manage the mobile-use background daemon.

The daemon maintains persistent WebSocket connections and session state.
It starts automatically when needed.

COMMANDS:
  start  - Start daemon (usually automatic)
  stop   - Stop daemon, closing all connections
  status - Show daemon status

Location: ~/.cache/mobile-use/daemon.sock")]
    Daemon(DaemonCommands),

    /// Stop the daemon (alias for daemon stop)
    #[command(long_about = "Stop the daemon. Shortcut for 'mobile-use daemon stop'.")]
    Stop,

    /// Quit the running application
    #[command(long_about = "Stop the running application started by 'mobile-use run'.

Sends a stop signal to the flutter process and waits for it to exit gracefully.
If the process doesn't exit within 5 seconds, it will be force-killed.

Use --all to perform a full reset: kill all processes, remove all state files,
and stop the daemon. Use this when mobile-use is in an inconsistent state.

Examples:
  mobile-use quit                  # Stop default session
  mobile-use -s mysession quit     # Stop specific session
  mobile-use quit --all            # Full reset - kill everything")]
    Quit {
        /// Kill all sessions, processes, and reset all state
        #[arg(long, help = "Full reset: kill all processes and clear all state",
            long_help = "Perform a full state reset:
- Kill all running flutter processes (all sessions)
- Stop the daemon
- Remove all PID files and socket files
- Clear legacy session files

Use this when mobile-use is in an inconsistent state.")]
        all: bool,
    },

    /// Flutter-specific commands
    #[command(subcommand)]
    #[command(long_about = "Commands specific to Flutter app development.

These commands require a Flutter app connection via VM Service.

COMMANDS:
  reload  - Hot reload (applies code changes, preserves state)
  restart - Hot restart (restarts app, loses state)
  widgets - Get raw widget tree (verbose)

Example:
  mobile-use flutter reload    # Hot reload after code change
  mobile-use flutter restart   # Full restart")]
    Flutter(FlutterCommands),
}

pub fn parse() -> Cli {
    Cli::parse()
}
