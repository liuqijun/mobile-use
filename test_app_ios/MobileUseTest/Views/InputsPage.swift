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
                // Username field
                IconTextField(
                    icon: "person",
                    label: "Username",
                    placeholder: "Enter username",
                    text: $username,
                    contentType: .username,
                    accessLabel: "Username Input"
                )

                // Password field
                IconSecureField(
                    icon: "lock",
                    label: "Password",
                    placeholder: "Enter password",
                    text: $password,
                    accessLabel: "Password Input"
                )

                // Email field
                IconTextField(
                    icon: "envelope",
                    label: "Email",
                    placeholder: "Enter email",
                    text: $email,
                    keyboardType: .emailAddress,
                    contentType: .emailAddress,
                    accessLabel: "Email Input"
                )

                // Search field with clear
                VStack(alignment: .leading, spacing: 4) {
                    Text("Search")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    HStack(spacing: 8) {
                        Image(systemName: "magnifyingglass")
                            .foregroundColor(.gray)
                        TextField("Search...", text: $search)
                        if !search.isEmpty {
                            Button(action: { search = "" }) {
                                Image(systemName: "xmark.circle.fill")
                                    .foregroundColor(.gray)
                            }
                            .accessibilityLabel("Clear Search")
                        }
                    }
                    .padding(10)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color(.systemGray3), lineWidth: 1)
                    )
                }
                .accessibilityLabel("Search Input")

                Spacer().frame(height: 8)

                // Submit button
                Button(action: {
                    submittedData = "Username: \(username)\nEmail: \(email)\nSearch: \(search)"
                }) {
                    Text("Submit")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .accessibilityLabel("Submit Button")

                // Clear All button
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

                // Result card
                if !submittedData.isEmpty {
                    VStack(alignment: .leading) {
                        Text(submittedData)
                    }
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color(.systemGray6))
                    .cornerRadius(12)
                    .shadow(color: .black.opacity(0.05), radius: 2, y: 1)
                    .accessibilityLabel("Submitted Data: \(submittedData)")
                }
            }
            .padding(16)
        }
        .navigationTitle("Text Inputs")
    }
}

// MARK: - Reusable input components

struct IconTextField: View {
    let icon: String
    let label: String
    let placeholder: String
    @Binding var text: String
    var keyboardType: UIKeyboardType = .default
    var contentType: UITextContentType? = nil
    var accessLabel: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            HStack(spacing: 8) {
                Image(systemName: icon)
                    .foregroundColor(.gray)
                TextField(placeholder, text: $text)
                    .keyboardType(keyboardType)
                    .textContentType(contentType)
            }
            .padding(10)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(.systemGray3), lineWidth: 1)
            )
        }
        .accessibilityLabel(accessLabel)
    }
}

struct IconSecureField: View {
    let icon: String
    let label: String
    let placeholder: String
    @Binding var text: String
    var accessLabel: String

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            HStack(spacing: 8) {
                Image(systemName: icon)
                    .foregroundColor(.gray)
                SecureField(placeholder, text: $text)
                    .textContentType(.password)
            }
            .padding(10)
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color(.systemGray3), lineWidth: 1)
            )
        }
        .accessibilityLabel(accessLabel)
    }
}
