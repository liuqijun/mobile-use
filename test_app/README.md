# Mobile Use Test App

Flutter test application for verifying all mobile-use CLI commands.

## Running the Test App

```bash
cd test_app
flutter run
```

## Test Pages

### 1. Home
- 4 navigation buttons for page navigation

### 2. Buttons & Taps Page
Tests: `tap`, `double-tap`, `long-press`, `is enabled`
- "Tap Me" button - test single tap
- "Double Tap Area" - test double tap
- "Long Press Area" - test long press
- "Enabled/Disabled Button" - test `is enabled`

### 3. Text Inputs Page
Tests: `text`, `clear`, `get text`
- Username input field
- Password input field
- Email input field
- Search input field (with clear button)
- Submit/Clear All buttons

### 4. Scrollable Lists Page
Tests: `scroll`, `swipe`, `wait`
- 50 list items
- Test vertical scrolling

### 5. Form Controls Page
Tests: `tap`, `is checked`, `get`
- Checkboxes (3, including a disabled one)
- Switches (2)
- Slider
- Radio Buttons (3 options)

## Acceptance Test Script

```bash
# 1. Connect to the app
mobile-use devices                    # List devices
mobile-use connect                    # Connect to the app

# 2. Test home page
mobile-use elements -i                # Get interactive elements
mobile-use screenshot home.png        # Take screenshot

# 3. Navigate to Buttons page
mobile-use tap @e1                    # Tap "Buttons & Taps" button
mobile-use elements -i
mobile-use tap @e2                    # Tap "Tap Me"
mobile-use long-press @e3             # Long press "Long Press Area"
mobile-use screenshot buttons.png

# 4. Go back and enter Inputs page
mobile-use tap @back                  # Go back
mobile-use tap @e2                    # Tap "Text Inputs"
mobile-use elements -i
mobile-use text @e1 "testuser"        # Input username
mobile-use text @e2 "password123"     # Input password
mobile-use text @e3 "test@example.com"
mobile-use get text @e1               # Get text
mobile-use clear @e1                  # Clear input
mobile-use screenshot inputs.png

# 5. Test Lists page
mobile-use tap @back
mobile-use tap @e3                    # Enter Lists page
mobile-use scroll down 500            # Scroll down
mobile-use scroll up 300              # Scroll up
mobile-use screenshot lists.png

# 6. Test Forms page
mobile-use tap @back
mobile-use tap @e4                    # Enter Forms page
mobile-use elements -i
mobile-use is checked @checkbox1      # Check checkbox state
mobile-use tap @checkbox1             # Tap checkbox
mobile-use is checked @checkbox1      # Check again
mobile-use screenshot forms.png

# 7. Flutter commands
mobile-use flutter reload             # Hot reload
mobile-use flutter widgets            # Get widget tree

# 8. Disconnect
mobile-use disconnect
```

## Command Coverage

| Command | Test Page | Test Target |
|---------|-----------|-------------|
| devices | — | List connected devices |
| connect | — | Connect to Flutter app |
| disconnect | — | Disconnect |
| info | — | Show connection info |
| elements | All pages | Get element tree |
| screenshot | All pages | Take screenshot |
| tap | Buttons, Forms | Buttons, checkboxes |
| double-tap | Buttons | Double Tap Area |
| long-press | Buttons | Long Press Area |
| text | Inputs | Text input fields |
| clear | Inputs | Clear input |
| scroll | Lists | List scrolling |
| swipe | Lists | Swipe gesture |
| get | Inputs, Forms | Get properties |
| is | Buttons, Forms | Check state |
| wait | Lists | Wait for element |
| flutter reload | — | Hot reload |
| flutter restart | — | Hot restart |
| flutter widgets | — | Widget tree |
