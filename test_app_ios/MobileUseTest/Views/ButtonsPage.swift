import SwiftUI

struct ButtonsPage: View {
    @State private var lastAction = "None"
    @State private var tapCount = 0
    @State private var isEnabled = true
    @State private var toastMessage = ""
    @State private var showToast = false

    var body: some View {
        ZStack(alignment: .bottom) {
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
                    .shadow(color: .black.opacity(0.05), radius: 2, y: 1)

                    Spacer().frame(height: 8)

                    // Tap Me Button
                    Button(action: {
                        lastAction = "Single Tap"
                        tapCount += 1
                        showToastMessage(lastAction)
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
                            showToastMessage(lastAction)
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
                            showToastMessage(lastAction)
                        }

                    // Enabled/Disabled Button + Toggle
                    HStack {
                        Button(action: {
                            lastAction = "Enabled Button Tapped"
                            showToastMessage(lastAction)
                        }) {
                            Text(isEnabled ? "Enabled Button" : "Disabled Button")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                        .disabled(!isEnabled)
                        .accessibilityLabel(isEnabled ? "Enabled Button" : "Disabled Button")

                        Spacer().frame(width: 16)

                        Toggle("", isOn: $isEnabled)
                            .labelsHidden()
                            .accessibilityLabel("Toggle Button Enable")
                    }

                    // Reset Button
                    Button(action: {
                        lastAction = "Reset"
                        tapCount = 0
                        showToastMessage(lastAction)
                    }) {
                        Text("Reset Counter")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .accessibilityLabel("Reset Counter")
                }
                .padding(16)
            }

            // Snackbar
            if showToast {
                Text(toastMessage)
                    .foregroundColor(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .background(Color(.darkGray))
                    .cornerRadius(8)
                    .padding(.bottom, 16)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                    .animation(.easeInOut(duration: 0.3), value: showToast)
            }
        }
        .navigationTitle("Buttons & Taps")
    }

    private func showToastMessage(_ message: String) {
        toastMessage = message
        withAnimation { showToast = true }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
            withAnimation { showToast = false }
        }
    }
}
