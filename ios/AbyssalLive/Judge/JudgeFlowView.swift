import SwiftUI

/// Switches between pairing and the paired judge's home screen based
/// on `ServerSession.isPaired`. Pairing state (token + judge) persists
/// across launches (Keychain + UserDefaults, see `ServerSession`), so
/// a judge who paired last week reopens straight into `JudgeHomeView`.
struct JudgeFlowView: View {
    @EnvironmentObject private var session: ServerSession
    let onExit: () -> Void

    var body: some View {
        Group {
            if session.isPaired {
                JudgeHomeView(onSignOut: { session.signOut() })
            } else {
                PairingView()
            }
        }
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Back", action: onExit)
            }
        }
    }
}
