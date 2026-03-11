# iOS Native Test App Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a native iOS (SwiftUI) test app (`test_app_ios/`) with identical UI structure, accessibility labels, and interactive behavior as the existing Flutter (`test_app/`) and Android Compose (`test_app_android/app-compose/`) test apps, for verifying mobile-use iOS automation.

**Architecture:** Single-target SwiftUI app with NavigationStack, 5 screens (Home, Buttons, Inputs, Lists, Forms). All interactive elements carry `.accessibilityIdentifier` and `.accessibilityLabel` matching the `contentDescription` / `Semantics.label` values used in Flutter and Android. State changes update accessibility labels dynamically (e.g., "Last Action: Single Tap").

**Tech Stack:** Swift 5.9+, SwiftUI, iOS 16+ deployment target, Xcode project (no SPM dependencies needed)

---

## UI Parity Reference

All three apps (Flutter, Android Compose, iOS SwiftUI) must expose **identical accessibility labels** so that `mobile-use elements` returns the same semantic tree regardless of platform.

| Screen | Accessibility Labels |
|--------|---------------------|
| Home | "Test Pages Header", "Buttons & Taps", "Text Inputs", "Scrollable Lists", "Form Controls" |
| Buttons | "Last Action: {value}", "Tap Count: {value}", "Tap Me Button", "Double Tap Area", "Long Press Area", "Enabled Button"/"Disabled Button", "Toggle Button Enable", "Reset Counter" |
| Inputs | "Username Input", "Password Input", "Email Input", "Search Input", "Clear Search", "Submit Button", "Clear All Button", "Submitted Data: {value}" |
| Lists | "List Item {1..50}" (each row) |
| Forms | "Option 1", "Option 2", "Option 3 (Disabled)", "Toggle 1", "Toggle 2", "Volume Slider", "Slider Value: {n}", "Choice A", "Choice B", "Choice C", "Current State: {text}" |

---

### Task 1: Create Xcode project skeleton

**Files:**
- Create: `test_app_ios/MobileUseTest.xcodeproj/` (via xcodebuild or Xcode template)
- Create: `test_app_ios/MobileUseTest/MobileUseTestApp.swift`
- Create: `test_app_ios/MobileUseTest/ContentView.swift`
- Create: `test_app_ios/MobileUseTest/Assets.xcassets/`

**Step 1: Create the project directory structure**

```bash
mkdir -p test_app_ios/MobileUseTest/Assets.xcassets/AppIcon.appiconset
mkdir -p test_app_ios/MobileUseTest/Assets.xcassets/AccentColor.colorset
```

**Step 2: Create the Xcode project file**

Create `test_app_ios/MobileUseTest.xcodeproj/project.pbxproj` with:
- Target: MobileUseTest (iOS Application)
- Deployment target: iOS 16.0
- Swift version: 5.9
- Bundle identifier: `com.example.mobileusetest`
- Product name: MobileUseTest

**Step 3: Create the app entry point**

Create `test_app_ios/MobileUseTest/MobileUseTestApp.swift`:

```swift
import SwiftUI

@main
struct MobileUseTestApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
```

**Step 4: Create the root navigation view**

Create `test_app_ios/MobileUseTest/ContentView.swift`:

```swift
import SwiftUI

struct ContentView: View {
    var body: some View {
        NavigationStack {
            HomePage()
        }
    }
}
```

**Step 5: Build to verify project compiles**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 6: Commit**

```bash
git add test_app_ios/
git commit -m "feat: create iOS native test app Xcode project skeleton"
```

---

### Task 2: Home Page

**Files:**
- Create: `test_app_ios/MobileUseTest/Views/HomePage.swift`

**Step 1: Implement HomePage**

```swift
import SwiftUI

struct HomePage: View {
    var body: some View {
        List {
            Section {
                Text("Test Pages")
                    .font(.title)
                    .fontWeight(.bold)
                    .accessibilityLabel("Test Pages Header")
            }

            NavigationLink(destination: ButtonsPage()) {
                Label("Buttons & Taps", systemImage: "hand.tap")
            }
            .accessibilityLabel("Buttons & Taps")

            NavigationLink(destination: InputsPage()) {
                Label("Text Inputs", systemImage: "keyboard")
            }
            .accessibilityLabel("Text Inputs")

            NavigationLink(destination: ListsPage()) {
                Label("Scrollable Lists", systemImage: "list.bullet")
            }
            .accessibilityLabel("Scrollable Lists")

            NavigationLink(destination: FormsPage()) {
                Label("Form Controls", systemImage: "checkmark.square")
            }
            .accessibilityLabel("Form Controls")
        }
        .navigationTitle("Mobile Use Test")
    }
}
```

**Step 2: Build to verify**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 3: Commit**

```bash
git add test_app_ios/MobileUseTest/Views/HomePage.swift
git commit -m "feat: add iOS test app Home page with navigation"
```

---

### Task 3: Buttons & Taps Page

**Files:**
- Create: `test_app_ios/MobileUseTest/Views/ButtonsPage.swift`

**Step 1: Implement ButtonsPage**

```swift
import SwiftUI

struct ButtonsPage: View {
    @State private var lastAction = "None"
    @State private var tapCount = 0
    @State private var isEnabled = true

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Status Card
                VStack(alignment: .leading, spacing: 8) {
                    Text("Last Action: \(lastAction)")
                        .font(.title3)
                        .accessibilityLabel("Last Action: \(lastAction)")
                    Text("Tap Count: \(tapCount)")
                        .font(.title3)
                        .accessibilityLabel("Tap Count: \(tapCount)")
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Tap Me Button
                Button(action: {
                    lastAction = "Single Tap"
                    tapCount += 1
                }) {
                    Text("Tap Me")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .accessibilityLabel("Tap Me Button")

                // Double Tap Area
                Text("Double Tap Area")
                    .frame(maxWidth: .infinity)
                    .padding(24)
                    .background(Color.blue.opacity(0.2))
                    .cornerRadius(8)
                    .accessibilityLabel("Double Tap Area")
                    .onTapGesture(count: 2) {
                        lastAction = "Double Tap"
                        tapCount += 2
                    }

                // Long Press Area
                Text("Long Press Area")
                    .frame(maxWidth: .infinity)
                    .padding(24)
                    .background(Color.green.opacity(0.2))
                    .cornerRadius(8)
                    .accessibilityLabel("Long Press Area")
                    .onLongPressGesture {
                        lastAction = "Long Press"
                        tapCount += 5
                    }

                // Enabled/Disabled Button + Toggle
                HStack {
                    Button(action: {
                        lastAction = "Enabled Button Tapped"
                    }) {
                        Text(isEnabled ? "Enabled Button" : "Disabled Button")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .disabled(!isEnabled)
                    .accessibilityLabel(isEnabled ? "Enabled Button" : "Disabled Button")

                    Toggle("", isOn: $isEnabled)
                        .labelsHidden()
                        .accessibilityLabel("Toggle Button Enable")
                }

                // Reset Button
                Button(action: {
                    lastAction = "Reset"
                    tapCount = 0
                }) {
                    Text("Reset Counter")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .accessibilityLabel("Reset Counter")
            }
            .padding(16)
        }
        .navigationTitle("Buttons & Taps")
    }
}
```

**Step 2: Build to verify**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 3: Commit**

```bash
git add test_app_ios/MobileUseTest/Views/ButtonsPage.swift
git commit -m "feat: add iOS test app Buttons & Taps page"
```

---

### Task 4: Text Inputs Page

**Files:**
- Create: `test_app_ios/MobileUseTest/Views/InputsPage.swift`

**Step 1: Implement InputsPage**

```swift
import SwiftUI

struct InputsPage: View {
    @State private var username = ""
    @State private var password = ""
    @State private var email = ""
    @State private var search = ""
    @State private var submittedData = ""

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                // Username
                TextField("Enter username", text: $username)
                    .textFieldStyle(.roundedBorder)
                    .textContentType(.username)
                    .accessibilityLabel("Username Input")

                // Password
                SecureField("Enter password", text: $password)
                    .textFieldStyle(.roundedBorder)
                    .textContentType(.password)
                    .accessibilityLabel("Password Input")

                // Email
                TextField("Enter email", text: $email)
                    .textFieldStyle(.roundedBorder)
                    .keyboardType(.emailAddress)
                    .textContentType(.emailAddress)
                    .accessibilityLabel("Email Input")

                // Search with clear
                HStack {
                    TextField("Search...", text: $search)
                        .textFieldStyle(.roundedBorder)
                        .accessibilityLabel("Search Input")

                    if !search.isEmpty {
                        Button(action: { search = "" }) {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundColor(.gray)
                        }
                        .accessibilityLabel("Clear Search")
                    }
                }

                // Submit Button
                Button(action: {
                    submittedData = "Username: \(username)\nEmail: \(email)\nSearch: \(search)"
                }) {
                    Text("Submit")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .accessibilityLabel("Submit Button")

                // Clear All Button
                Button(action: {
                    username = ""
                    password = ""
                    email = ""
                    search = ""
                    submittedData = "All fields cleared"
                }) {
                    Text("Clear All")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
                .accessibilityLabel("Clear All Button")

                // Submitted Data Card
                if !submittedData.isEmpty {
                    Text(submittedData)
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color(.systemGray6))
                        .cornerRadius(12)
                        .accessibilityLabel("Submitted Data: \(submittedData)")
                }
            }
            .padding(16)
        }
        .navigationTitle("Text Inputs")
    }
}
```

**Step 2: Build to verify**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 3: Commit**

```bash
git add test_app_ios/MobileUseTest/Views/InputsPage.swift
git commit -m "feat: add iOS test app Text Inputs page"
```

---

### Task 5: Scrollable Lists Page

**Files:**
- Create: `test_app_ios/MobileUseTest/Views/ListsPage.swift`

**Step 1: Implement ListsPage**

```swift
import SwiftUI

struct ListsPage: View {
    @State private var tappedItem: Int? = nil
    @State private var showSnackbar = false

    var body: some View {
        ZStack(alignment: .bottom) {
            List(1...50, id: \.self) { index in
                Button(action: {
                    tappedItem = index
                    showSnackbar = true
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
                        showSnackbar = false
                    }
                }) {
                    HStack(spacing: 16) {
                        // Avatar
                        ZStack {
                            Circle()
                                .fill(Color.blue)
                                .frame(width: 40, height: 40)
                            Text("\(index)")
                                .foregroundColor(.white)
                                .font(.callout)
                        }

                        // Text content
                        VStack(alignment: .leading) {
                            Text("List Item \(index)")
                                .font(.body)
                                .fontWeight(.semibold)
                                .foregroundColor(.primary)
                            Text("Description for item \(index)")
                                .font(.subheadline)
                                .foregroundColor(.gray)
                        }

                        Spacer()

                        Image(systemName: "chevron.right")
                            .foregroundColor(.gray)
                    }
                }
                .accessibilityLabel("List Item \(index)")
            }
            .listStyle(.plain)

            // Snackbar
            if showSnackbar, let item = tappedItem {
                Text("Tapped item \(item)")
                    .padding()
                    .background(Color(.systemGray2))
                    .cornerRadius(8)
                    .padding(.bottom, 16)
                    .transition(.move(edge: .bottom))
                    .animation(.easeInOut, value: showSnackbar)
            }
        }
        .navigationTitle("Scrollable Lists")
    }
}
```

**Step 2: Build to verify**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 3: Commit**

```bash
git add test_app_ios/MobileUseTest/Views/ListsPage.swift
git commit -m "feat: add iOS test app Scrollable Lists page"
```

---

### Task 6: Form Controls Page

**Files:**
- Create: `test_app_ios/MobileUseTest/Views/FormsPage.swift`

**Step 1: Implement FormsPage**

```swift
import SwiftUI

struct FormsPage: View {
    @State private var checkbox1 = false
    @State private var checkbox2 = true
    @State private var checkbox3 = false
    @State private var switch1 = false
    @State private var switch2 = true
    @State private var sliderValue: Double = 50
    @State private var radioValue = 1

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                // Checkboxes Section
                Text("Checkboxes")
                    .font(.title3)
                    .fontWeight(.bold)
                    .padding(.bottom, 8)

                Toggle(isOn: $checkbox1) {
                    VStack(alignment: .leading) {
                        Text("Option 1")
                        Text("Unchecked by default")
                            .font(.caption)
                            .foregroundColor(.gray)
                    }
                }
                .toggleStyle(.checkbox)
                .accessibilityLabel("Option 1")
                .padding(.vertical, 4)

                Toggle(isOn: $checkbox2) {
                    VStack(alignment: .leading) {
                        Text("Option 2")
                        Text("Checked by default")
                            .font(.caption)
                            .foregroundColor(.gray)
                    }
                }
                .toggleStyle(.checkbox)
                .accessibilityLabel("Option 2")
                .padding(.vertical, 4)

                Toggle(isOn: .constant(false)) {
                    VStack(alignment: .leading) {
                        Text("Option 3 (Disabled)")
                        Text("Cannot be changed")
                            .font(.caption)
                            .foregroundColor(.gray)
                    }
                }
                .toggleStyle(.checkbox)
                .disabled(true)
                .accessibilityLabel("Option 3 (Disabled)")
                .padding(.vertical, 4)

                Divider().padding(.vertical, 16)

                // Switches Section
                Text("Switches")
                    .font(.title3)
                    .fontWeight(.bold)
                    .padding(.bottom, 8)

                Toggle(isOn: $switch1) {
                    VStack(alignment: .leading) {
                        Text("Toggle 1")
                        Text("Off by default")
                            .font(.caption)
                            .foregroundColor(.gray)
                    }
                }
                .accessibilityLabel("Toggle 1")
                .padding(.vertical, 4)

                Toggle(isOn: $switch2) {
                    VStack(alignment: .leading) {
                        Text("Toggle 2")
                        Text("On by default")
                            .font(.caption)
                            .foregroundColor(.gray)
                    }
                }
                .accessibilityLabel("Toggle 2")
                .padding(.vertical, 4)

                Divider().padding(.vertical, 16)

                // Slider Section
                Text("Slider")
                    .font(.title3)
                    .fontWeight(.bold)
                    .padding(.bottom, 8)

                Slider(value: $sliderValue, in: 0...100, step: 10)
                    .accessibilityLabel("Volume Slider")

                Text("Value: \(Int(sliderValue))")
                    .frame(maxWidth: .infinity)
                    .accessibilityLabel("Slider Value: \(Int(sliderValue))")

                Divider().padding(.vertical, 16)

                // Radio Buttons Section
                Text("Radio Buttons")
                    .font(.title3)
                    .fontWeight(.bold)
                    .padding(.bottom, 8)

                RadioButton(label: "Choice A", isSelected: radioValue == 1) {
                    radioValue = 1
                }
                .accessibilityLabel("Choice A")

                RadioButton(label: "Choice B", isSelected: radioValue == 2) {
                    radioValue = 2
                }
                .accessibilityLabel("Choice B")

                RadioButton(label: "Choice C", isSelected: radioValue == 3) {
                    radioValue = 3
                }
                .accessibilityLabel("Choice C")

                Divider().padding(.vertical, 16)

                // State Display Card
                let stateText = """
                Checkbox 1: \(checkbox1)
                Checkbox 2: \(checkbox2)
                Switch 1: \(switch1)
                Switch 2: \(switch2)
                Slider: \(Int(sliderValue))
                Radio: \(radioValue)
                """

                VStack(alignment: .leading, spacing: 4) {
                    Text("Current State:")
                        .fontWeight(.bold)
                    Text(stateText)
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(.systemGray6))
                .cornerRadius(12)
                .accessibilityLabel("Current State: \(stateText)")
            }
            .padding(16)
        }
        .navigationTitle("Form Controls")
    }
}

// SwiftUI doesn't have native radio buttons, so we build one
struct RadioButton: View {
    let label: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 8) {
                Image(systemName: isSelected ? "largecircle.fill.circle" : "circle")
                    .foregroundColor(isSelected ? .blue : .gray)
                Text(label)
                    .foregroundColor(.primary)
            }
            .padding(.vertical, 4)
        }
    }
}

// iOS doesn't have a checkbox toggle style by default before iOS 16+
// We use a custom one for visual parity
extension ToggleStyle where Self == CheckboxToggleStyle {
    static var checkbox: CheckboxToggleStyle { CheckboxToggleStyle() }
}

struct CheckboxToggleStyle: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        Button(action: { configuration.isOn.toggle() }) {
            HStack(spacing: 8) {
                Image(systemName: configuration.isOn ? "checkmark.square.fill" : "square")
                    .foregroundColor(configuration.isOn ? .blue : .gray)
                    .font(.title3)
                configuration.label
            }
        }
        .buttonStyle(.plain)
    }
}
```

**Step 2: Build to verify**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 3: Commit**

```bash
git add test_app_ios/MobileUseTest/Views/FormsPage.swift
git commit -m "feat: add iOS test app Form Controls page"
```

---

### Task 7: Asset catalog and Info.plist

**Files:**
- Create: `test_app_ios/MobileUseTest/Assets.xcassets/Contents.json`
- Create: `test_app_ios/MobileUseTest/Assets.xcassets/AccentColor.colorset/Contents.json`
- Create: `test_app_ios/MobileUseTest/Assets.xcassets/AppIcon.appiconset/Contents.json`

**Step 1: Create asset catalog JSON files**

`Assets.xcassets/Contents.json`:
```json
{
  "info": { "version": 1, "author": "xcode" }
}
```

`AccentColor.colorset/Contents.json`:
```json
{
  "colors": [{ "idiom": "universal" }],
  "info": { "version": 1, "author": "xcode" }
}
```

`AppIcon.appiconset/Contents.json`:
```json
{
  "images": [{ "idiom": "universal", "platform": "ios", "size": "1024x1024" }],
  "info": { "version": 1, "author": "xcode" }
}
```

**Step 2: Commit**

```bash
git add test_app_ios/MobileUseTest/Assets.xcassets/
git commit -m "feat: add iOS test app asset catalog"
```

---

### Task 8: Full build and simulator test

**Step 1: Build for simulator**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Expected: BUILD SUCCEEDED

**Step 2: Run on simulator (manual verification)**

```bash
cd test_app_ios
xcodebuild -scheme MobileUseTest -destination 'platform=iOS Simulator,name=iPhone 16' \
  -derivedDataPath build/ build
xcrun simctl install booted build/Build/Products/Debug-iphonesimulator/MobileUseTest.app
xcrun simctl launch booted com.example.mobileusetest
```

Manual check:
- [ ] Home page shows 4 navigation links
- [ ] Buttons page: tap/double-tap/long-press all update status
- [ ] Inputs page: text fields accept input, submit/clear work
- [ ] Lists page: 50 items scroll, tap shows snackbar
- [ ] Forms page: checkboxes, switches, slider, radio all work

**Step 3: Final commit**

```bash
git add -A test_app_ios/
git commit -m "feat: complete iOS native test app with all test pages"
```

---

## Accessibility Label Parity Checklist

After implementation, verify these labels match across all three platforms by running:

```bash
# Android
mobile-use -d <android_device> elements

# iOS (once iOS support is implemented)
mobile-use -d <ios_device> elements

# Flutter
mobile-use -d <flutter_device> elements
```

All three should produce matching `label` fields for equivalent UI elements.
