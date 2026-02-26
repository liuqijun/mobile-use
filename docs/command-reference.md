# mobile-use 命令参考手册

> Cross-platform UI automation CLI for AI agents

mobile-use 是一个跨平台的 UI 自动化命令行工具，专为 AI 代理设计。支持 Flutter 应用（通过 VM Service）和原生 Android 应用（通过 ADB/uiautomator）。

---

## 安装

```bash
# 通过 Homebrew (macOS)
brew tap liuqijun/tap
brew install mobile-use

# 通过 cargo
cargo install mobile-use

# 从源码
cargo install --git https://github.com/liuqijun/mobile-use.git
```

---

## 快速开始

```bash
# 1. 启动 Flutter 应用
mobile-use run -- -d emulator-5554

# 2. 获取 UI 元素列表
mobile-use elements

# 3. 与元素交互
mobile-use tap @e1              # 点击元素
mobile-use text @e2 "hello"     # 输入文本
```

---

## 全局选项

| 选项 | 说明 | 示例 |
|------|------|------|
| `-d, --device <DEVICE>` | 指定设备 ID | `-d emulator-5554` |
| `-s, --session <SESSION>` | 会话名称（多应用场景） | `-s app1` |
| `--json` | JSON 格式输出 | `--json` |
| `-h, --help` | 显示帮助 | `--help` |
| `-V, --version` | 显示版本 | `--version` |

### 设备标识符格式

```bash
# USB 连接
-d emulator-5554
-d 1234567890ABCDEF

# 无线连接
-d 192.168.1.100:5555
```

### 多会话示例

```bash
mobile-use -s app1 connect --package com.example.app1
mobile-use -s app2 connect --package com.example.app2
mobile-use -s app1 elements    # 获取 app1 的元素
mobile-use -s app2 tap @e1     # 在 app2 中点击
```

---

## 命令列表

### 连接管理

| 命令 | 说明 |
|------|------|
| `run` | 运行应用（自动检测 Flutter 或 Android） |
| `connect` | 连接到目标应用 |
| `disconnect` | 断开连接 |
| `info` | 显示连接信息 |
| `devices` | 列出已连接的设备 |

### 元素操作

| 命令 | 说明 |
|------|------|
| `elements` | 获取 UI 元素树 |
| `tap` | 点击元素 |
| `double-tap` | 双击元素 |
| `long-press` | 长按元素 |
| `text` | 输入文本 |
| `clear` | 清除元素内容 |

### 滚动与滑动

| 命令 | 说明 |
|------|------|
| `scroll` | 滚动屏幕 |
| `scroll-to` | 滚动直到元素可见 |
| `swipe` | 滑动手势 |

### 等待与查询

| 命令 | 说明 |
|------|------|
| `wait` | 等待条件满足 |
| `get` | 获取元素属性 |
| `is` | 检查元素状态 |

### 其他

| 命令 | 说明 |
|------|------|
| `screenshot` | 截图 |
| `flutter` | Flutter 专用命令 |
| `daemon` | 守护进程管理 |
| `stop` | 停止守护进程 |
| `quit` | 停止运行中的应用 |
| `quit --all` | 重置所有状态（杀死所有进程、清除文件） |

---

## 详细命令说明

### run - 运行应用

```bash
mobile-use run [OPTIONS] [APK] [-- <ARGS>...]
```

**Flutter 模式**（默认）：
```bash
mobile-use run                        # 在当前 Flutter 项目运行
mobile-use run -- -d emulator-5554    # 指定设备
mobile-use run -- --release           # Release 模式
mobile-use run -- --flavor prod       # 指定 flavor
```

**Android 模式**：
```bash
mobile-use run app.apk                      # 安装并运行 APK
mobile-use run --package com.example.app    # 启动已安装的应用
```

| 参数 | 说明 |
|------|------|
| `[APK]` | APK 文件路径（Android 模式） |
| `--package` | Android 包名 |
| `[ARGS]...` | 传递给 flutter run 的参数（需在 `--` 之后） |

---

### connect - 连接应用

```bash
mobile-use connect [OPTIONS]
```

**Flutter 模式**：
```bash
# 通过 WebSocket URL 连接
mobile-use connect --url ws://127.0.0.1:55370/abc123=/ws

# 通过端口自动发现
mobile-use connect --port 55370
```

**Android 模式**：
```bash
mobile-use connect --package com.example.app
```

| 参数 | 说明 |
|------|------|
| `--url` | Flutter VM Service WebSocket URL |
| `--port` | 端口号（自动发现 URL） |
| `--package` | Android 包名 |

**URL 转换**：
```
flutter run 输出:  http://127.0.0.1:55370/abc123=/
mobile-use 需要:     ws://127.0.0.1:55370/abc123=/ws
```

---

### elements - 获取元素树

```bash
mobile-use elements [OPTIONS]
```

| 参数 | 说明 |
|------|------|
| `-i, --interactive` | 仅显示可交互元素 |

**输出格式**：
```
@e1 [button] "Login" (100,200 300x50)
  @e2 [text] "Username" (110,210 280x30)
@e3 [textfield] "Password" (100,260 300x50)
```

**字段说明**：
- `@e1` - 元素引用 ID（用于后续操作）
- `[button]` - 元素类型
- `"Login"` - 元素标签/文本
- `(100,200 300x50)` - 坐标和尺寸

---

### tap - 点击元素

```bash
mobile-use tap <REFERENCE>
```

```bash
mobile-use tap @e1     # 点击元素 @e1
mobile-use tap @e3     # 点击元素 @e3
```

---

### double-tap - 双击元素

```bash
mobile-use double-tap <REFERENCE>
```

```bash
mobile-use double-tap @e1    # 双击，用于文本选择或缩放
```

**行为**：两次快速点击，间隔 50ms。

---

### long-press - 长按元素

```bash
mobile-use long-press [OPTIONS] <REFERENCE>
```

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--duration` | 按住时长（毫秒） | 500 |

```bash
mobile-use long-press @e1                  # 长按 500ms
mobile-use long-press @e1 --duration 1000  # 长按 1 秒
```

---

### text - 输入文本

```bash
mobile-use text [OPTIONS] <REFERENCE> <TEXT>
```

| 参数 | 说明 |
|------|------|
| `--clear` | 输入前清除现有文本 |

```bash
mobile-use text @e2 "hello world"        # 输入文本
mobile-use text @e2 "new text" --clear   # 清除后输入
```

**注意**：此命令会自动点击元素获取焦点，无需先执行 `tap`。

---

### clear - 清除内容

```bash
mobile-use clear <REFERENCE>
```

```bash
mobile-use clear @e2    # 清除文本框内容
```

**行为**：自动点击聚焦 → 移到文本末尾 → 逐个退格删除（最多 50 次）。

---

### scroll - 滚动屏幕

```bash
mobile-use scroll <DIRECTION> [DISTANCE]
```

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `<DIRECTION>` | 方向：up, down, left, right | - |
| `[DISTANCE]` | 滚动距离（像素） | 300 |

```bash
mobile-use scroll down         # 向下滚动 300px
mobile-use scroll down 500     # 向下滚动 500px
mobile-use scroll up 200       # 向上滚动 200px
mobile-use scroll left         # 向左滚动
```

**方向说明**：
- `down` - 显示下方内容（手指向上滑）
- `up` - 显示上方内容（手指向下滑）
- `left` - 显示右侧内容（手指向左滑）
- `right` - 显示左侧内容（手指向右滑）

**参数细节**：从屏幕中心开始滑动，持续 300ms。

---

### scroll-to - 滚动到元素

```bash
mobile-use scroll-to <REFERENCE>
```

```bash
mobile-use scroll-to @e15    # 滚动直到 @e15 可见
```

---

### swipe - 滑动手势

```bash
mobile-use swipe [OPTIONS] <DIRECTION>
```

| 参数 | 说明 |
|------|------|
| `--from` | 起始元素（可选，默认屏幕中心） |

```bash
mobile-use swipe left              # 从中心向左滑动
mobile-use swipe right --from @e5  # 从元素 @e5 向右滑动
```

**与 scroll 的区别**：`swipe` 用于 UI 手势（如删除卡片、下拉刷新），`scroll` 用于滚动内容。

**参数细节**：滑动距离固定 500px，持续 200ms。

---

### wait - 等待条件

```bash
mobile-use wait [OPTIONS] [TARGET]
```

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `[TARGET]` | 元素引用或毫秒数 | - |
| `--text` | 等待文本出现 | - |
| `--timeout` | 超时时间（毫秒） | 30000 |

**等待元素出现**：
```bash
mobile-use wait @e5                  # 等待 @e5 出现
mobile-use wait @e5 --timeout 5000   # 最多等 5 秒
```

**等待文本出现**：
```bash
mobile-use wait --text "Success"     # 等待 "Success" 出现
```

**等待固定时间**：
```bash
mobile-use wait 2000                 # 等待 2 秒
```

**行为细节**：元素/文本等待模式下每 500ms 轮询一次。超时默认 30 秒。

---

### get - 获取元素属性

```bash
mobile-use get <PROPERTY> <REFERENCE>
```

| 属性 | 说明 |
|------|------|
| `text` | 元素文本内容 |
| `type` | 元素类型（如 button、textfield） |
| `prop` | 所有语义属性（JSON） |
| `bounds` | 坐标和尺寸 |
| `<自定义>` | 从 properties 中查找任意属性键 |

```bash
mobile-use get text @e3      # 获取文本: "Login"
mobile-use get type @e3      # 获取类型: "button"
mobile-use get bounds @e3    # 获取: {x:100, y:200, width:300, height:50}
mobile-use get prop @e3      # 获取所有属性（JSON）
```

---

### is - 检查元素状态

```bash
mobile-use is <STATE> <REFERENCE>
```

| 状态 | 说明 |
|------|------|
| `visible` | 元素是否可见 |
| `enabled` | 元素是否可交互（非 disabled） |
| `checked` | 元素是否被选中（复选框等） |
| `focused` | 元素是否拥有焦点 |

```bash
mobile-use is visible @e3    # 检查是否可见
mobile-use is enabled @e3    # 检查是否可用
mobile-use is checked @e3    # 检查是否选中
mobile-use is focused @e3    # 检查是否聚焦
```

返回 `true` 或 `false`。

---

### screenshot - 截图

```bash
mobile-use screenshot [PATH]
```

```bash
mobile-use screenshot                  # 保存为 screenshot-<时间戳>.png
mobile-use screenshot output.png       # 保存为 output.png
mobile-use screenshot /tmp/screen.png  # 保存到指定路径
```

---

### info - 连接信息

```bash
mobile-use info
```

显示当前连接状态：
```
Mode: Flutter
Device: emulator-5554
Session: default
VM Service: ws://127.0.0.1:55370/abc=/ws
```

---

### devices - 设备列表

```bash
mobile-use devices
```

输出示例：
```
Found 2 device(s):

  [1] emulator-5554
      Model:   Android SDK built for x86_64 (Google)
      Android: 13 (SDK 33)
      Screen:  1080x2400

  [2] 192.168.1.100:5555
      Model:   Mi 10 (Xiaomi)
      Android: 13 (SDK 33)
      Screen:  1080x2340
```

---

### flutter - Flutter 命令

```bash
mobile-use flutter <COMMAND>
```

| 子命令 | 说明 |
|--------|------|
| `reload` | 热重载（保留状态） |
| `restart` | 热重启（重置状态） |
| `widgets` | 获取 Widget 树 |

```bash
mobile-use flutter reload     # 热重载
mobile-use flutter restart    # 热重启
mobile-use flutter widgets    # 获取 Widget 树（很详细）
```

---

### daemon - 守护进程管理

```bash
mobile-use daemon <COMMAND>
```

| 子命令 | 说明 |
|--------|------|
| `start` | 启动守护进程（通常自动启动） |
| `stop` | 停止守护进程 |
| `status` | 显示状态 |

```bash
mobile-use daemon status    # 查看状态
mobile-use daemon stop      # 停止
mobile-use stop             # 同上（快捷方式）
```

守护进程位置：`~/.cache/mobile-use/daemon.sock`

### quit - 停止应用 / 重置状态

```bash
mobile-use quit              # 停止当前 session 的 flutter 进程
mobile-use -s app1 quit      # 停止指定 session
mobile-use quit --all        # 完全重置：杀死所有进程、清除所有状态文件
```

**`quit --all` 执行的操作：**
1. 杀死所有 `mobile-use run` 进程（所有 session）
2. 杀死孤立的 `flutter run --machine` 进程
3. 停止守护进程
4. 删除所有 PID 文件和 socket 文件
5. 清除旧版 session 文件

**适用场景：** 当 mobile-use 状态混乱时（崩溃后残留进程、PID 文件过期等），使用 `quit --all` 一键恢复。

---

## 元素引用说明

### 获取元素引用

```bash
mobile-use elements
```

输出中的 `@e1`, `@e2` 等就是元素引用。

### 引用的生命周期

**重要**：元素引用是临时的，UI 变化后可能失效。

```bash
mobile-use elements      # 获取 @e1, @e2, @e3
mobile-use tap @e1       # 点击后 UI 变化
mobile-use elements      # 重新获取！@e1 可能指向不同元素
```

### 最佳实践

1. 操作前先获取最新元素列表
2. 使用 `--json` 输出便于程序解析
3. 结合 `wait` 确保元素可用

---

## 自动化脚本示例

### 登录流程

```bash
#!/bin/bash

# 启动应用
mobile-use run -- -d emulator-5554 &
sleep 10

# 等待登录页面
mobile-use wait --text "Login"

# 获取元素
mobile-use elements

# 输入用户名密码
mobile-use text @e2 "username"
mobile-use text @e3 "password"

# 点击登录
mobile-use tap @e4

# 等待登录成功
mobile-use wait --text "Welcome" --timeout 10000

# 截图
mobile-use screenshot login_success.png
```

### 列表滚动

```bash
#!/bin/bash

# 滚动查找特定项目
for i in {1..10}; do
    mobile-use elements | grep "Target Item" && break
    mobile-use scroll down 500
    sleep 0.5
done

# 点击找到的项目
mobile-use tap @e15
```

---

## 故障排除

### 连接问题

```bash
# 检查设备连接
adb devices

# 检查守护进程状态
mobile-use daemon status

# 重启守护进程
mobile-use daemon stop
mobile-use daemon start

# 状态混乱时一键重置
mobile-use quit --all
```

### 元素找不到

```bash
# 使用 -i 只显示可交互元素
mobile-use elements -i

# 使用 --json 获取详细信息
mobile-use elements --json
```

### 无线 ADB 限制

某些操作（scroll, swipe）在无线 ADB 下可能因权限问题失败。建议使用 USB 连接进行完整测试。

---

## 架构说明

```
CLI Command → Unix Socket → Daemon Process → WebSocket → Flutter VM Service
                              ↓
                         Session Manager (内存状态)
```

- **CLI**：无状态命令行接口
- **Daemon**：后台守护进程，维护 WebSocket 连接和会话状态
- **Session**：管理多应用连接

---

## 命令实现状态

| 命令 | 状态 | 备注 |
|------|------|------|
| `run` | ✅ | Flutter 模式完整，Android 待完善 |
| `connect` | ✅ | Flutter 完整，Android 部分实现 |
| `disconnect` | ✅ | |
| `quit` | ✅ | 含 `--all` 完全重置 |
| `elements` | ✅ | 含样式提取 |
| `tap` | ✅ | |
| `double-tap` | ✅ | |
| `long-press` | ✅ | |
| `text` | ✅ | 自动聚焦 |
| `clear` | ✅ | |
| `screenshot` | ✅ | |
| `scroll` | ✅ | |
| `scroll-to` | ❌ | 尚未实现 |
| `swipe` | ✅ | |
| `wait` | ✅ | |
| `get` | ✅ | |
| `is` | ✅ | |
| `info` | ✅ | |
| `devices` | ✅ | |
| `flutter reload` | ✅ | |
| `flutter restart` | ✅ | |
| `flutter widgets` | ✅ | |
| `daemon start/stop/status` | ✅ | |
| `stop` | ✅ | daemon stop 别名 |

---

## 版本

```bash
mobile-use --version
# mobile-use 0.1.0
```
