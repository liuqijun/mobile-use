import SwiftUI

struct HomePage: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Test Pages")
                .font(.title)
                .fontWeight(.bold)
                .accessibilityLabel("Test Pages Header")

            NavButton(label: "Buttons & Taps", icon: "hand.tap", destination: ButtonsPage())
            NavButton(label: "Text Inputs", icon: "keyboard", destination: InputsPage())
            NavButton(label: "Scrollable Lists", icon: "list.bullet", destination: ListsPage())
            NavButton(label: "Form Controls", icon: "checkmark.square", destination: FormsPage())

            Spacer()
        }
        .padding(16)
        .navigationTitle("Mobile Use Test")
    }
}

struct NavButton<Destination: View>: View {
    let label: String
    let icon: String
    let destination: Destination

    var body: some View {
        NavigationLink(destination: destination) {
            HStack(spacing: 12) {
                Image(systemName: icon)
                    .font(.title3)
                Text(label)
                    .font(.body)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, 14)
            .padding(.horizontal, 16)
            .background(Color.accentColor.opacity(0.1))
            .cornerRadius(10)
        }
        .accessibilityLabel(label)
    }
}
