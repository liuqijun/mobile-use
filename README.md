# mobile-use

Mobile UI automation CLI for AI agents — like [browser-use](https://github.com/browser-use/browser-use), but for mobile apps.

`mobile-use` enables AI agents to understand and interact with mobile application UIs through a simple command-line interface. It supports Flutter apps (via VM Service) and native Android apps (via ADB/uiautomator).

## Installation

```bash
# macOS (Homebrew)
brew tap liuqijun/tap
brew install mobile-use

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

## How It Works

```
CLI Command → Unix Socket → Daemon Process → WebSocket → Flutter VM Service
                              ↓
                         Session Manager (in-memory state)
```

mobile-use uses a background daemon to maintain persistent WebSocket connections and session state. CLI commands are stateless — they communicate with the daemon via Unix socket IPC.

## Commands

| Category | Commands |
|----------|----------|
| **Connection** | `run`, `connect`, `disconnect`, `info`, `devices` |
| **Elements** | `elements`, `tap`, `double-tap`, `long-press`, `text`, `clear` |
| **Navigation** | `scroll`, `scroll-to`, `swipe` |
| **Query** | `get`, `is`, `wait` |
| **Capture** | `screenshot` |
| **Flutter** | `flutter reload`, `flutter restart`, `flutter widgets` |
| **Management** | `daemon start/stop/status`, `stop`, `quit` |

## Documentation

- [Command Reference](docs/command-reference.md) — Full command documentation
- [Flutter Integration Guide](docs/flutter-mobile-use-guide.md) — Flutter app setup and best practices

## License

MIT
