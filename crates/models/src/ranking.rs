use serde::{Deserialize, Serialize};

use crate::ids::{EventId, ParticipantId, RankingId, TeamId};

/// A computed standing, derived from `EventResult`s once an event (or
/// the whole competition, for an all-around) is finalized. Kept
/// separate from `EventResult` so re-ranking (e.g. after a score
/// correction) doesn't mean re-deriving raw results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingEntry {
    pub id: RankingId,
    /// None when this is an overall/all-around ranking across events.
    pub event_id: Option<EventId>,
    pub participant_id: Option<ParticipantId>,
    pub team_id: Option<TeamId>,
    pub rank: i32,
    pub score: f64,
}
