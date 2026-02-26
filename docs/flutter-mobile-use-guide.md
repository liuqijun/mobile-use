# Flutter Mobile-Use 使用指南

Mobile-Use 是一个 UI 自动化 CLI 工具，专为 AI 代理设计，用于理解和操作 Flutter 应用 UI。

## 目录

1. [快速开始](#快速开始)
2. [获取 VM Service URL（最佳实践）](#获取-vm-service-url最佳实践)
3. [Flutter 应用开发规范](#flutter-应用开发规范)
4. [命令参考](#命令参考)
5. [输出格式](#输出格式)
6. [AI 集成工作流](#ai-集成工作流)
7. [常见问题](#常见问题)

---

## 快速开始

### 方式一：使用 mobile-use run（推荐）

```bash
# 1. 查看连接的设备
mobile-use devices

# 2. 在 Flutter 项目目录下启动应用（自动连接）
cd your_flutter_app
mobile-use run -- -d <device_id>

# 应用启动后自动连接，可以在另一个终端执行命令：
# 3. 获取 UI 元素树
mobile-use elements

# 4. 执行操作
mobile-use tap @e5
mobile-use input @e3 "hello"

# 5. 截图
mobile-use capture screenshot.png

# 6. 在 run 终端中按 'r' 热重载，'R' 热重启，'q' 退出
```

### 方式二：手动连接（需要 VM Service URL）

```bash
# 1. 查看连接的设备
mobile-use devices

# 2. 连接到 Flutter 应用（需要先获取 VM Service URL，见下节）
mobile-use -d <device_id> connect --url "ws://127.0.0.1:<port>/<token>/ws"

# 3. 获取 UI 元素树
mobile-use elements

# 4. 执行操作
mobile-use tap @e5
mobile-use input @e3 "hello"

# 5. 截图
mobile-use capture screenshot.png
```

---

## 获取 VM Service URL

如果使用 `mobile-use run` 命令，VM Service URL 会自动获取和连接，无需手动操作。

以下是手动获取 VM Service URL 的方法（用于 `mobile-use connect` 命令）：

### 方法一：flutter run 直接获取

```bash
# 在 Flutter 项目目录下运行
flutter run -d <device_id>
```

启动后，在输出中查找：
```
A Dart VM Service on Mi 10 is available at: http://127.0.0.1:55370/jcsm3VFShF0=/
```

**URL 转换规则：**
```
http://127.0.0.1:55370/jcsm3VFShF0=/     # flutter run 输出
  ↓
ws://127.0.0.1:55370/jcsm3VFShF0=/ws    # mobile-use 需要的格式
```

转换步骤：
1. `http://` → `ws://`
2. 末尾添加 `ws`

### 方法二：flutter attach 连接已运行的应用

如果应用已经在运行（不是从 flutter run 启动）：

```bash
# 连接到设备上正在运行的 Flutter 应用
flutter attach -d <device_id>
```

同样会输出 VM Service URL。

### 方法三：从设备日志获取

```bash
# 清空并监控日志
adb -s <device_id> logcat -c
adb -s <device_id> logcat | grep -E "Observatory|VM Service|ws://"
```

然后启动或重启 Flutter 应用。

### 远程设备端口转发

如果设备是通过无线 ADB 连接的（如 `192.168.1.100:5555`），需要转发端口：

```bash
# 端口转发（本地端口 → 设备端口）
adb -s 192.168.1.100:5555 forward tcp:55370 tcp:55370

# 验证转发
adb forward --list
```

**注意：** 每次应用重启，VM Service URL 会变化，需要重新获取。

### 完整连接流程示例

```bash
# 1. 查看设备
$ mobile-use devices
Found 1 device(s):
  [1] 192.168.1.100:5555
      Model: Mi 10 (Xiaomi)

# 2. 启动 Flutter 应用
$ cd my_flutter_app
$ flutter run -d 192.168.1.100:5555
...
A Dart VM Service on Mi 10 is available at: http://127.0.0.1:55370/jcsm3VFShF0=/

# 3. 端口转发（新终端）
$ adb -s 192.168.1.100:5555 forward tcp:55370 tcp:55370

# 4. 连接 mobile-use
$ mobile-use -d 192.168.1.100:5555 connect --url "ws://127.0.0.1:55370/jcsm3VFShF0=/ws"
Connected to ws://127.0.0.1:55370/jcsm3VFShF0=/ws

# 5. 验证连接
$ mobile-use info
Platform: flutter
Device: 192.168.1.100:5555
VM Service: ws://127.0.0.1:55370/jcsm3VFShF0=/ws
Connected: true
```

---

## Flutter 应用开发规范

为了让 Mobile-Use 能够正确识别和操作 UI 元素，Flutter 应用需要遵循以下规范。

### 自动生成语义的组件

以下 Flutter 内置组件**自动生成语义信息**，无需额外配置：

| 组件 | 自动获取的语义 |
|------|--------------|
| `Text("Hello")` | label: "Hello" |
| `ElevatedButton(child: Text("OK"))` | isButton: true, label: "OK" |
| `TextField(labelText: "Email")` | isTextField: true, label: "Email" |
| `IconButton(tooltip: "Search")` | isButton: true, label: "Search" |
| `Checkbox` | isChecked: true/false |
| `Switch` | toggled state |
| `Slider` | value, min, max |

### 需要添加语义的情况

对于自定义组件，需要使用 `Semantics` widget：

```dart
// 不推荐：没有语义的自定义按钮
GestureDetector(
  onTap: () {},
  child: Container(
    child: Icon(Icons.add),
  ),
)

// 推荐：添加语义标签
Semantics(
  label: 'Add Item Button',
  button: true,
  child: GestureDetector(
    onTap: () {},
    child: Container(
      child: Icon(Icons.add),
    ),
  ),
)
```

### 语义标签命名规范

| 元素类型 | 推荐命名格式 | 示例 |
|---------|-------------|------|
| 按钮 | `<动作> Button` | `Submit Button`, `Login Button` |
| 输入框 | `<字段名> Input` | `Username Input`, `Email Input` |
| 列表项 | `<内容> Item` | `User Profile Item` |
| 图标按钮 | 使用 tooltip | `tooltip: 'Search'` |

### 调试语义树

开发时可以启用语义调试查看边界：

```dart
MaterialApp(
  showSemanticsDebugger: true,  // 显示语义边界
  // ...
)
```

---

## 命令参考

### 全局参数

| 参数 | 说明 | 示例 |
|------|------|------|
| `-d, --device` | 指定设备 ID | `-d 192.168.1.100:5555` |
| `-s, --session` | 会话名称（多应用场景） | `-s myapp` |
| `--json` | JSON 输出格式 | `elements --json` |
| `-p, --platform` | 平台类型 | `-p flutter` (默认) |

### 启动和连接

```bash
# 列出设备
mobile-use devices

# 方式一：使用 run 命令启动应用（推荐，自动连接）
cd your_flutter_app
mobile-use run -- -d <device>
mobile-use run -- -d <device> --release        # Release 模式
mobile-use run -- -d <device> --flavor dev     # 指定 flavor

# 方式二：手动连接到已运行的应用
mobile-use -d <device> connect --url "ws://127.0.0.1:<port>/<token>/ws"

# 断开连接
mobile-use disconnect

# 查看连接信息
mobile-use info
```

**run 命令说明：**
- `--` 后的参数会直接传递给 `flutter run`
- 应用启动后自动连接到 daemon
- 在 run 终端中可以按 `r` 热重载、`R` 热重启、`q` 退出
- 与原生 `flutter run` 行为完全一致

### 元素查询

```bash
# 获取元素树（默认 styled 模式，包含样式）
mobile-use elements

# 仅交互元素
mobile-use elements -i

# 不同输出模式
mobile-use elements --mode styled    # 包含颜色、字体（默认）
mobile-use elements --mode semantic  # 仅语义结构
mobile-use elements --mode widget    # 完整 Widget 树
mobile-use elements --mode raw       # 原始调试输出

# JSON 格式输出
mobile-use elements --json
mobile-use elements --json > ui.json
```

### 交互操作

```bash
# 点击
mobile-use tap @e5

# 双击
mobile-use double-tap @e5

# 长按
mobile-use long-press @e5 --duration 1000

# 输入文本
mobile-use input @e3 "hello world"

# 清空后输入
mobile-use input @e3 "new text" --clear

# 清空输入框
mobile-use clear @e3
```

### 滚动与滑动

```bash
# 滚动（方向: up, down, left, right）
mobile-use scroll down 500

# 滚动到元素可见
mobile-use scroll-to @e10

# 滑动手势
mobile-use swipe left
```

### 截图

```bash
# 截图（默认文件名）
mobile-use capture

# 指定文件名
mobile-use capture screenshot.png

# 完整页面截图
mobile-use capture --full page.png
```

### 元素属性查询

```bash
# 获取元素文本
mobile-use get text @e3

# 获取元素属性
mobile-use get prop @e3

# 获取元素边界
mobile-use get bounds @e3

# 检查可见性
mobile-use is visible @e3

# 检查是否启用
mobile-use is enabled @e3
```

### 等待

```bash
# 等待元素出现
mobile-use wait @e5

# 等待文本出现
mobile-use wait --text "Loading complete"

# 等待指定时间（毫秒）
mobile-use wait 2000

# 设置超时
mobile-use wait @e5 --timeout 10000
```

### Flutter 特有命令

```bash
# 热重载
mobile-use flutter:hot-reload

# 热重启
mobile-use flutter:hot-restart

# 获取 Widget 树
mobile-use flutter:widgets
```

**Hot Reload/Restart 行为说明：**

| 连接方式 | hot-reload | hot-restart |
|---------|------------|-------------|
| `mobile-use run` | 提示按 'r' | 提示按 'R' |
| `mobile-use connect` | VM Service 调用 | 显示解决方案 |

使用 `mobile-use run` 启动时，热重载/重启通过 flutter 进程 stdin 触发，需要在 run 终端中按键操作。

### 守护进程管理

```bash
# 启动守护进程
mobile-use daemon start

# 停止守护进程
mobile-use daemon stop
mobile-use stop  # 别名

# 查看状态
mobile-use daemon status
```

---

## 输出格式

### 人类可读格式（默认）

```
- container [ref=@e15] (0,0 3024x6429)
  - header "Mobile Use Test" [ref=@e1] (45,131 378x78) {color:#191C20 font:14px} [isHeader]
  - button "Submit" [ref=@e5] (45,433 1010x146) {color:#36618E font:14px weight:500} [isButton] [isEnabled]
    - button "Submit" [ref=@e4] (45,433 1010x146) [isButton] [isEnabled]
```

格式说明：
```
- <类型> "标签" [ref=@<ID>] (x,y width×height) {样式} [标志]
```

| 部分 | 说明 |
|------|------|
| 类型 | button, text, header, textField, container 等 |
| 标签 | 语义标签文本 |
| ref | 元素引用 ID，用于命令操作（如 `tap @e5`） |
| 坐标 | 物理像素坐标 (x,y width×height) |
| 样式 | `{color:#xxx font:Npx weight:N}` |
| 标志 | `[isButton]` `[isEnabled]` `[isHeader]` 等 |

### JSON 格式

```bash
mobile-use elements --json
```

```json
{
  "success": true,
  "data": {
    "tree": {
      "ref_id": "e15",
      "element_type": "container",
      "label": null,
      "bounds": { "x": 0, "y": 0, "width": 3024, "height": 6429 },
      "children": [...]
    },
    "refs": {
      "e5": {
        "ref_id": "e5",
        "element_type": "button",
        "label": "Submit",
        "bounds": { "x": 45, "y": 433, "width": 1010, "height": 146 },
        "properties": { "isButton": true, "isEnabled": true },
        "style": {
          "textColor": "#36618E",
          "fontSize": 14.0,
          "fontWeight": "500"
        }
      }
    }
  }
}
```

### 样式属性

| 属性 | 说明 | 示例 |
|------|------|------|
| `textColor` | 文字颜色 | `#191C20` |
| `backgroundColor` | 背景颜色 | `#F2F4F8` |
| `fontSize` | 字体大小（逻辑像素） | `14.0` |
| `fontWeight` | 字重 (400/500/700) | `500` |
| `borderRadius` | 圆角半径 | `8.0` |

### 元素类型

| 类型 | 说明 | 对应 Flutter 组件 |
|------|------|------------------|
| `container` | 容器 | Scaffold, Column, Row, Container |
| `button` | 按钮 | ElevatedButton, TextButton, IconButton |
| `text` | 文本 | Text |
| `header` | 标题 | AppBar title |
| `textField` | 输入框 | TextField, TextFormField |
| `image` | 图片 | Image |
| `scrollable` | 滚动区域 | ListView, SingleChildScrollView |
| `checkbox` | 复选框 | Checkbox |
| `switch` | 开关 | Switch |

---

## AI 集成工作流

### 典型使用流程

```bash
# 1. 连接到应用
mobile-use connect --url "ws://127.0.0.1:55370/xxx=/ws"

# 2. 获取 UI 结构（JSON 格式便于解析）
mobile-use elements --json > ui.json

# 3. 截图
mobile-use capture screen.png

# 4. AI 分析
# - 解析 ui.json 理解 UI 结构
# - 查看 screen.png 了解视觉效果
# - 与设计稿对比

# 5. 执行操作
mobile-use tap @e5
mobile-use input @e3 "test@example.com"

# 6. 验证结果
mobile-use capture result.png
mobile-use elements --json > after.json
```

### 视觉对比示例

设计稿要求：
- 按钮文字: "Submit", 颜色 #36618E, 14px medium

UI 树输出：
```
button "Submit" [ref=@e5] {color:#36618E font:14px weight:500}
```

验证结果：
- 文字内容: "Submit" ✓
- 颜色: #36618E ✓
- 字体: 14px ✓
- 字重: 500 (medium) ✓

---

## 常见问题

### Q: VM Service URL 每次都会变化吗？

是的，每次 Flutter 应用重启都会生成新的 URL。需要重新获取并连接。

### Q: 连接断开后如何重新连接？

1. 检查 Flutter 应用是否还在运行
2. 如果应用重启了，重新获取 VM Service URL
3. 重新执行端口转发（如果是远程设备）
4. 使用新 URL 执行 `mobile-use connect`

### Q: 元素引用 ID (@e5) 会变化吗？

会。以下情况 ID 会变化：
- 页面跳转
- UI 状态更新
- 重新执行 `elements` 命令

**建议：** 每次操作前先执行 `elements` 获取最新引用。

### Q: 为什么有些自定义组件无法识别？

自定义组件（如 GestureDetector + Container）需要添加 `Semantics` widget 才能被识别。参见"Flutter 应用开发规范"章节。

### Q: 无线调试时某些操作失败？

部分 Android 操作（如滚动、滑动）需要 `INJECT_EVENTS` 权限。解决方案：
1. 使用 USB 连接替代无线调试
2. 在设备设置中启用"USB调试（安全设置）"

### Q: 如何查看更详细的 Widget 信息？

使用 widget 模式获取完整渲染树：
```bash
mobile-use elements --mode widget
```

这会返回完整的 Flutter Widget 树结构，信息最全但数据量较大。
