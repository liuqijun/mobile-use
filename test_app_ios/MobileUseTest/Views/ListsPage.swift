import SwiftUI

struct ListsPage: View {
    @State private var tappedItem: Int? = nil
    @State private var showSnackbar = false

    var body: some View {
        ZStack(alignment: .bottom) {
            List(1...50, id: \.self) { index in
                Button(action: {
                    tappedItem = index
                    withAnimation { showSnackbar = true }
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
                        withAnimation { showSnackbar = false }
                    }
                }) {
                    HStack(spacing: 16) {
                        ZStack {
                            Circle()
                                .fill(Color.blue)
                                .frame(width: 40, height: 40)
                            Text("\(index)")
                                .foregroundColor(.white)
                                .font(.callout)
                        }

                        VStack(alignment: .leading, spacing: 4) {
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
                            .font(.caption)
                    }
                    .padding(.vertical, 4)
                }
                .accessibilityLabel("List Item \(index)")
            }
            .listStyle(.plain)

            // Snackbar
            if showSnackbar, let item = tappedItem {
                Text("Tapped item \(item)")
                    .foregroundColor(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .background(Color(.darkGray))
                    .cornerRadius(8)
                    .padding(.bottom, 16)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                    .animation(.easeInOut(duration: 0.3), value: showSnackbar)
            }
        }
        .navigationTitle("Scrollable Lists")
    }
}
