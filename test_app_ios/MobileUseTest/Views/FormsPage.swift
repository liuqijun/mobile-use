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
                        Text("Unchecked by default").font(.caption).foregroundColor(.gray)
                    }
                }
                .toggleStyle(CheckboxToggleStyle())
                .accessibilityLabel("Option 1")
                .padding(.vertical, 4)

                Toggle(isOn: $checkbox2) {
                    VStack(alignment: .leading) {
                        Text("Option 2")
                        Text("Checked by default").font(.caption).foregroundColor(.gray)
                    }
                }
                .toggleStyle(CheckboxToggleStyle())
                .accessibilityLabel("Option 2")
                .padding(.vertical, 4)

                Toggle(isOn: .constant(false)) {
                    VStack(alignment: .leading) {
                        Text("Option 3 (Disabled)")
                        Text("Cannot be changed").font(.caption).foregroundColor(.gray)
                    }
                }
                .toggleStyle(CheckboxToggleStyle())
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
                        Text("Off by default").font(.caption).foregroundColor(.gray)
                    }
                }
                .accessibilityLabel("Toggle 1")
                .padding(.vertical, 4)

                Toggle(isOn: $switch2) {
                    VStack(alignment: .leading) {
                        Text("Toggle 2")
                        Text("On by default").font(.caption).foregroundColor(.gray)
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

                RadioButton(label: "Choice A", isSelected: radioValue == 1) { radioValue = 1 }
                    .accessibilityLabel("Choice A")
                RadioButton(label: "Choice B", isSelected: radioValue == 2) { radioValue = 2 }
                    .accessibilityLabel("Choice B")
                RadioButton(label: "Choice C", isSelected: radioValue == 3) { radioValue = 3 }
                    .accessibilityLabel("Choice C")

                Divider().padding(.vertical, 16)

                // State Display Card
                let stateText = "Checkbox 1: \(checkbox1)\nCheckbox 2: \(checkbox2)\nSwitch 1: \(switch1)\nSwitch 2: \(switch2)\nSlider: \(Int(sliderValue))\nRadio: \(radioValue)"

                VStack(alignment: .leading, spacing: 4) {
                    Text("Current State:").fontWeight(.bold)
                    Text(stateText)
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(.systemGray6))
                .cornerRadius(12)
                .shadow(color: .black.opacity(0.05), radius: 2, y: 1)
                .accessibilityLabel("Current State: \(stateText)")
            }
            .padding(16)
        }
        .navigationTitle("Form Controls")
    }
}

struct RadioButton: View {
    let label: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 8) {
                Image(systemName: isSelected ? "largecircle.fill.circle" : "circle")
                    .foregroundColor(isSelected ? .blue : .gray)
                Text(label).foregroundColor(.primary)
            }
            .padding(.vertical, 4)
        }
    }
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
