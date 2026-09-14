import SwiftUI

@main
struct AbyssalLiveApp: App {
    @StateObject private var session = ServerSession()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(session)
        }
    }
}
