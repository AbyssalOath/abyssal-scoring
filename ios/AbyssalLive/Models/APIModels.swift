import Foundation

// Request/response bodies not already covered by ScoreboardModels.swift.
// Encoded with a plain `JSONEncoder` (no snake_case conversion), so
// these Swift property names ARE the wire format - keep them
// camelCase to match `crates/live_server/src/routes.rs` exactly.

struct PairRequest: Encodable {
    let code: String
    let deviceName: String
}

struct PairResponse: Decodable {
    let token: String
    let judge: Judge
}

struct GymnasticsScoreRequest: Encodable {
    let eventId: UUID
    let participantId: UUID
    let apparatus: Apparatus
    let difficultyScore: Double
    let judgeScores: [JudgeScoreEntry]
    let neutralDeductions: Double
}

struct SwimmingScoreRequest: Encodable {
    let eventId: UUID
    let participantId: UUID
    let elapsedSeconds: Double
    let timePenaltySeconds: Double
}
