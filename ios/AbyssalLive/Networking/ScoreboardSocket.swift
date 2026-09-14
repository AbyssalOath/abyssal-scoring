import Foundation

/// Watches `GET /ws/scoreboard/:competitionId` (see docs/LIVE_API.md)
/// and republishes each push as `state`. Reconnects with a fixed
/// delay on any failure -- there's no exponential backoff or give-up
/// point, since a judge/spectator app should just keep trying for as
/// long as the screen showing it is open.
@MainActor
final class ScoreboardSocket: ObservableObject {
    @Published private(set) var state: ScoreboardState?
    @Published private(set) var isConnected = false

    private var task: URLSessionWebSocketTask?
    private var shouldReconnect = false
    private var connectionURL: URL?

    private let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }()

    func connect(session: ServerSession, competitionId: UUID) {
        guard let base = session.baseURL,
              var components = URLComponents(url: base, resolvingAgainstBaseURL: false)
        else { return }
        components.scheme = components.scheme == "https" ? "wss" : "ws"
        components.path += "/ws/scoreboard/\(competitionId.uuidString)"
        guard let url = components.url else { return }

        connectionURL = url
        shouldReconnect = true
        openSocket(url: url)
    }

    func disconnect() {
        shouldReconnect = false
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        isConnected = false
    }

    private func openSocket(url: URL) {
        let newTask = URLSession.shared.webSocketTask(with: url)
        task = newTask
        newTask.resume()
        isConnected = true
        listen()
    }

    private func listen() {
        let currentTask = task
        currentTask?.receive { [weak self] result in
            guard let self else { return }
            Task { @MainActor in
                self.handle(result: result)
            }
        }
    }

    private func handle(result: Result<URLSessionWebSocketTask.Message, Error>) {
        switch result {
        case .failure:
            isConnected = false
            scheduleReconnect()
        case .success(let message):
            apply(message: message)
            listen()
        }
    }

    private func apply(message: URLSessionWebSocketTask.Message) {
        let data: Data?
        switch message {
        case .data(let raw):
            data = raw
        case .string(let text):
            data = text.data(using: .utf8)
        @unknown default:
            data = nil
        }
        guard let data, let push = try? decoder.decode(ScoreboardPush.self, from: data) else { return }
        state = push.payload
    }

    private func scheduleReconnect() {
        guard shouldReconnect, let url = connectionURL else { return }
        Task {
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            guard self.shouldReconnect else { return }
            self.openSocket(url: url)
        }
    }
}
