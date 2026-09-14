import SwiftUI

/// Top-level mode switch. Deliberately one app for both roles (a
/// judge may also want to check the live view, and it's nearly the
/// same codebase either way) rather than two separate apps - see
/// ARCHITECTURE.md's "Networked live judging" section in the main
/// repo for why.
enum AppMode {
    case judge
    case spectator
}

struct RootView: View {
    @EnvironmentObject private var session: ServerSession
    @State private var mode: AppMode?

    var body: some View {
        NavigationStack {
            Group {
                switch mode {
                case .judge:
                    JudgeFlowView(onExit: { mode = nil })
                case .spectator:
                    SpectatorFlowView(onExit: { mode = nil })
                case nil:
                    modePicker
                }
            }
        }
    }

    private var modePicker: some View {
        VStack(spacing: 24) {
            Spacer()

            VStack(spacing: 8) {
                Text("Abyssal Live")
                    .font(.largeTitle.bold())
                Text("Connects to the scoring computer over your local Wi-Fi network.")
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal)

            VStack(spacing: 12) {
                Button {
                    mode = .judge
                } label: {
                    Label("I'm Judging", systemImage: "checkmark.seal")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)

                Button {
                    mode = .spectator
                } label: {
                    Label("I'm Watching", systemImage: "eye")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
            }
            .padding(.horizontal, 32)

            Spacer()
            Spacer()
        }
        .navigationTitle("Abyssal Live")
        .navigationBarTitleDisplayMode(.inline)
    }
}

#Preview {
    RootView()
        .environmentObject(ServerSession())
}
