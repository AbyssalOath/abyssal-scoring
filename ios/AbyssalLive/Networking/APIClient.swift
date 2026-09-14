import Foundation

/// Errors surfaced to the UI. `errorDescription` is what
/// `Text(error.localizedDescription)` (or a caught `error` cast to
/// `LocalizedError`) shows the judge/spectator directly, so keep
/// these short and actionable.
enum APIError: Error, LocalizedError {
    case invalidURL
    case notPaired
    case server(status: Int, message: String)
    case decoding(Error)
    case network(Error)

    var errorDescription: String? {
        switch self {
        case .invalidURL:
            return "Enter the server's address first."
        case .notPaired:
            return "Not paired with a judge yet."
        case .server(let status, let message):
            return status == 401 ? "This device isn't paired (or was revoked). Pair again." : message
        case .decoding:
            return "The server sent something this app didn't understand. Is it running a compatible version?"
        case .network(let error):
            return "Couldn't reach the server: \(error.localizedDescription)"
        }
    }
}

private struct ServerErrorBody: Decodable {
    let error: String
}

/// Thin wrapper over the API in docs/LIVE_API.md. One instance is
/// created per call site (it's stateless beyond reading `session`) --
/// see `ServerSession` for the actual connection/auth state.
/// `@MainActor` because `ServerSession` is too, and every call site is
/// already on the main actor (SwiftUI view actions/`.task`) anyway.
@MainActor
final class APIClient {
    private let session: ServerSession

    init(session: ServerSession) {
        self.session = session
    }

    private func decoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }

    private func request(path: String, method: String, authenticated: Bool) throws -> URLRequest {
        guard let base = session.baseURL else { throw APIError.invalidURL }
        var request = URLRequest(url: base.appendingPathComponent(path))
        request.httpMethod = method
        if authenticated {
            guard let token = session.token else { throw APIError.notPaired }
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        return request
    }

    private func perform(_ request: URLRequest) async throws -> Data {
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await URLSession.shared.data(for: request)
        } catch {
            throw APIError.network(error)
        }
        guard let http = response as? HTTPURLResponse else {
            throw APIError.network(URLError(.badServerResponse))
        }
        guard (200..<300).contains(http.statusCode) else {
            let message = (try? decoder().decode(ServerErrorBody.self, from: data))?.error ?? "Request failed"
            throw APIError.server(status: http.statusCode, message: message)
        }
        return data
    }

    private func decode<T: Decodable>(_ type: T.Type, from data: Data) throws -> T {
        do {
            return try decoder().decode(type, from: data)
        } catch {
            throw APIError.decoding(error)
        }
    }

    // MARK: Public

    func health() async throws {
        let req = try request(path: "health", method: "GET", authenticated: false)
        _ = try await perform(req)
    }

    func listCompetitions() async throws -> [Competition] {
        let req = try request(path: "competitions", method: "GET", authenticated: false)
        return try decode([Competition].self, from: try await perform(req))
    }

    func listEvents(competitionId: UUID) async throws -> [MeetEvent] {
        let req = try request(path: "competitions/\(competitionId)/events", method: "GET", authenticated: false)
        return try decode([MeetEvent].self, from: try await perform(req))
    }

    func listParticipants(competitionId: UUID) async throws -> [Participant] {
        let req = try request(path: "competitions/\(competitionId)/participants", method: "GET", authenticated: false)
        return try decode([Participant].self, from: try await perform(req))
    }

    func scoreboardState(competitionId: UUID) async throws -> ScoreboardState {
        let req = try request(path: "scoreboard/\(competitionId)", method: "GET", authenticated: false)
        return try decode(ScoreboardState.self, from: try await perform(req))
    }

    // MARK: Pairing + authenticated writes

    func pair(code: String, deviceName: String) async throws -> PairResponse {
        var req = try request(path: "pair", method: "POST", authenticated: false)
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONEncoder().encode(PairRequest(code: code, deviceName: deviceName))
        return try decode(PairResponse.self, from: try await perform(req))
    }

    func submitGymnasticsScore(_ body: GymnasticsScoreRequest) async throws -> EventResult {
        var req = try request(path: "scores/gymnastics-panel", method: "POST", authenticated: true)
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONEncoder().encode(body)
        return try decode(EventResult.self, from: try await perform(req))
    }

    func submitSwimmingScore(_ body: SwimmingScoreRequest) async throws -> EventResult {
        var req = try request(path: "scores/swimming", method: "POST", authenticated: true)
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONEncoder().encode(body)
        return try decode(EventResult.self, from: try await perform(req))
    }
}
