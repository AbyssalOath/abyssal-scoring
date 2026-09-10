use serde::{Deserialize, Serialize};

use crate::ids::{EventId, ParticipantId, ResultId, TeamId};

/// The outcome of one participant (or team) in one event. This is the
/// piece every sport shares -- what fills `detail` is sport-specific
/// and lives as a JSON blob here plus, where useful, a dedicated
/// sport-specific table keyed by `result_id` (see e.g. gymnastics::Routine).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub id: ResultId,
    pub event_id: EventId,
    pub participant_id: Option<ParticipantId>,
    pub team_id: Option<TeamId>,
    /// The single number used for ranking (final score, time in
    /// seconds, distance, etc.) -- comparison direction is decided by
    /// the scoring module for that sport (higher-is-better vs
    /// lower-is-better), not by this struct.
    pub final_value: Option<f64>,
    pub place: Option<i32>,
    pub finalized: bool,
    /// Sport-specific breakdown, e.g. for gymnastics:
    /// {"difficulty": 5.2, "execution": 8.766, "deductions": 0.3}
    pub detail: serde_json::Value,
}

impl EventResult {
    pub fn new(event_id: EventId, participant_id: ParticipantId) -> Self {
        Self {
            id: ResultId::new(),
            event_id,
            participant_id: Some(participant_id),
            team_id: None,
            final_value: None,
            place: None,
            finalized: false,
            detail: serde_json::Value::Null,
        }
    }
}
