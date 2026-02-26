# Mobile Use Test App

Flutter测试应用，用于验收 mobile-use CLI 的所有命令。

## 运行测试应用

```bash
cd test_app
flutter run
```

## 测试页面

### 1. 首页 (Home)
- 4个导航按钮，测试页面跳转

### 2. Buttons & Taps 页面
测试命令：`tap`, `double-tap`, `long-press`, `is enabled`
- "Tap Me" 按钮 - 测试单击
- "Double Tap Area" - 测试双击
- "Long Press Area" - 测试长按
- "Enabled/Disabled Button" - 测试 `is enabled`

### 3. Text Inputs 页面
测试命令：`input`, `clear`, `get text`
- Username 输入框
- Password 输入框
- Email 输入框
- Search 输入框（带清除按钮）
- Submit/Clear All 按钮

### 4. Scrollable Lists 页面
测试命令：`scroll`, `swipe`, `wait`
- 50个列表项
- 测试上下滚动

### 5. Form Controls 页面
测试命令：`tap`, `is checked`, `get`
- Checkboxes (3个，包括禁用的)
- Switches (2个)
- Slider
- Radio Buttons (3个选项)

## 验收测试脚本

```bash
# 1. 连接到应用
mobile-use devices                    # 列出设备
mobile-use connect                    # 连接到应用

# 2. 测试首页
mobile-use elements -i                # 获取交互元素
mobile-use capture home.png           # 截图

# 3. 导航到 Buttons 页面
mobile-use tap @e1                    # 点击 "Buttons & Taps" 按钮
mobile-use elements -i
mobile-use tap @e2                    # 点击 "Tap Me"
mobile-use long-press @e3             # 长按 "Long Press Area"
mobile-use capture buttons.png

# 4. 返回并进入 Inputs 页面
mobile-use tap @back                  # 返回
mobile-use tap @e2                    # 点击 "Text Inputs"
mobile-use elements -i
mobile-use input @e1 "testuser"       # 输入用户名
mobile-use input @e2 "password123"    # 输入密码
mobile-use input @e3 "test@example.com"
mobile-use get text @e1               # 获取文本
mobile-use clear @e1                  # 清除输入
mobile-use capture inputs.png

# 5. 测试 Lists 页面
mobile-use tap @back
mobile-use tap @e3                    # 进入 Lists 页面
mobile-use scroll down 500            # 向下滚动
mobile-use scroll up 300              # 向上滚动
mobile-use capture lists.png

# 6. 测试 Forms 页面
mobile-use tap @back
mobile-use tap @e4                    # 进入 Forms 页面
mobile-use elements -i
mobile-use is checked @checkbox1      # 检查checkbox状态
mobile-use tap @checkbox1             # 点击checkbox
mobile-use is checked @checkbox1      # 再次检查
mobile-use capture forms.png

# 7. Flutter 命令
mobile-use flutter:hot-reload         # 热重载
mobile-use flutter:widgets            # 获取widget树

# 8. 断开连接
mobile-use disconnect
```

## 测试覆盖的命令

| 命令 | 测试页面 | 测试元素 |
|------|---------|---------|
| devices | - | 列出连接的设备 |
| connect | - | 连接到Flutter应用 |
| disconnect | - | 断开连接 |
| info | - | 显示连接信息 |
| elements | 所有页面 | 获取元素树 |
| capture | 所有页面 | 截图 |
| tap | Buttons, Forms | 按钮、复选框 |
| double-tap | Buttons | Double Tap Area |
| long-press | Buttons | Long Press Area |
| input | Inputs | 文本输入框 |
| clear | Inputs | 清除输入 |
| scroll | Lists | 列表滚动 |
| swipe | Lists | 滑动手势 |
| get | Inputs, Forms | 获取属性 |
| is | Buttons, Forms | 检查状态 |
| wait | Lists | 等待元素 |
| flutter:hot-reload | - | 热重载 |
| flutter:hot-restart | - | 热重启 |
| flutter:widgets | - | Widget树 |
