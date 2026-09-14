import Foundation

/// Single source of truth for "which server, paired as which judge."
/// Survives app relaunch: the server address is in `UserDefaults`,
/// the device token is in the Keychain (see `KeychainStore`), and the
/// paired `Judge` record is cached in `UserDefaults` as JSON (not
/// secret, just convenient to have without a network round trip).
@MainActor
final class ServerSession: ObservableObject {
    @Published private(set) var host: String
    @Published private(set) var port: Int
    @Published private(set) var token: String?
    @Published private(set) var judge: Judge?

    private let keychain = KeychainStore()
    private let defaults = UserDefaults.standard

    private enum Keys {
        static let host = "server.host"
        static let port = "server.port"
        static let judge = "session.judge"
    }

    init() {
        host = defaults.string(forKey: Keys.host) ?? ""
        let storedPort = defaults.integer(forKey: Keys.port)
        port = storedPort == 0 ? 7878 : storedPort
        token = keychain.readToken()
        judge = Self.loadJudge(defaults: defaults)
    }

    var baseURL: URL? {
        guard !host.isEmpty else { return nil }
        return URL(string: "http://\(host):\(port)/api/v1")
    }

    var isPaired: Bool { token != nil && judge != nil }

    func saveConnection(host: String, port: Int) {
        self.host = host
        self.port = port
        defaults.set(host, forKey: Keys.host)
        defaults.set(port, forKey: Keys.port)
    }

    func completePairing(token: String, judge: Judge) {
        self.token = token
        self.judge = judge
        keychain.saveToken(token)
        if let data = try? JSONEncoder().encode(judge) {
            defaults.set(data, forKey: Keys.judge)
        }
    }

    func signOut() {
        token = nil
        judge = nil
        keychain.deleteToken()
        defaults.removeObject(forKey: Keys.judge)
    }

    private static func loadJudge(defaults: UserDefaults) -> Judge? {
        guard let data = defaults.data(forKey: Keys.judge) else { return nil }
        return try? JSONDecoder().decode(Judge.self, from: data)
    }
}
