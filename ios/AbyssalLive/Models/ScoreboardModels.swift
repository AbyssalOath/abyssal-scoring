import Foundation

// Mirrors `models::Judge` / `models::JudgeRole` (crates/models/src/judge.rs).
// Field names here are what `JSONDecoder.keyDecodingStrategy =
// .convertFromSnakeCase` turns the wire's snake_case keys into - see
// docs/LIVE_API.md for the exact JSON shapes this must stay in sync with.
struct Judge: Codable, Identifiable, Hashable {
    let id: UUID
    let competitionId: UUID
    let name: String
    let role: String
    let assignment: String?
}

// Mirrors `models::Competition` / `models::Sport`.
struct Competition: Codable, Identifiable, Hashable {
    let id: UUID
    let name: String
    let sport: String
    let date: String
    let location: String?
    let themeId: UUID?
}

// Mirrors `models::Event`. Named `MeetEvent` to avoid any ambiguity
// with SwiftUI/AppKit's own `Event`-shaped types.
struct MeetEvent: Codable, Identifiable, Hashable {
    let id: UUID
    let competitionId: UUID
    let sessionId: UUID?
    let name: String
    let orderIndex: Int
    let status: String
}

// Mirrors `models::Participant`.
struct Participant: Codable, Identifiable, Hashable {
    let id: UUID
    let competitionId: UUID
    let firstName: String
    let lastName: String
    let teamId: UUID?
    let bibNumber: String?
    let levelOrCategory: String?
    let isAttending: Bool

    var fullName: String { "\(firstName) \(lastName)" }
}

// Mirrors `models::gymnastics::Apparatus`. Raw values are the exact
// snake_case strings the server expects/returns.
enum Apparatus: String, Codable, CaseIterable, Identifiable {
    case vault
    case unevenBars = "uneven_bars"
    case balanceBeam = "balance_beam"
    case floor
    case pommelHorse = "pommel_horse"
    case stillRings = "still_rings"
    case parallelBars = "parallel_bars"
    case horizontalBar = "horizontal_bar"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .vault: return "Vault"
        case .unevenBars: return "Uneven Bars"
        case .balanceBeam: return "Balance Beam"
        case .floor: return "Floor"
        case .pommelHorse: return "Pommel Horse"
        case .stillRings: return "Still Rings"
        case .parallelBars: return "Parallel Bars"
        case .horizontalBar: return "Horizontal Bar"
        }
    }
}

// Mirrors `models::gymnastics::Deduction` exactly (no camelCase
// conversion needed - both fields are already single words).
struct Deduction: Codable, Equatable {
    var reason: String
    var value: Double
}

// Mirrors `server::JudgeDeductions`, which IS camelCase-renamed on the
// Rust side (`#[serde(rename_all = "camelCase")]`) - `judgeId` here,
// not `judge_id`.
struct JudgeScoreEntry: Codable {
    var judgeId: UUID?
    var deductions: [Deduction]
}

// Mirrors `models::DisplayMode`.
enum DisplayMode: String, Codable {
    case idle
    case inProgress = "in_progress"
    case showingResult = "showing_result"
    case showingRanking = "showing_ranking"

    var label: String {
        switch self {
        case .idle: return "Idle"
        case .inProgress: return "In progress"
        case .showingResult: return "Result"
        case .showingRanking: return "Standings"
        }
    }
}

// Mirrors `models::ScoreboardState`. `liveValue`/`detail` (free-form
// per-sport JSON) is deliberately not modeled here - nothing in this
// app needs it beyond what's already on `EventResult.finalValue`.
struct ScoreboardState: Codable {
    let competitionId: UUID
    let currentEventId: UUID?
    let currentParticipantId: UUID?
    let displayMode: DisplayMode
    let updatedAt: String
}

// Mirrors `models::EventResult` (only the fields this app displays).
struct EventResult: Codable {
    let id: UUID
    let eventId: UUID
    let participantId: UUID?
    let finalValue: Double?
    let finalized: Bool
}

/// The one WebSocket message shape `live_server` ever sends -- see
/// `crates/live_server/src/ws.rs`.
struct ScoreboardPush: Codable {
    let type: String
    let payload: ScoreboardState
}
