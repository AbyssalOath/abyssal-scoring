use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, SessionId};

/// A block of the schedule (e.g. "Session 1 - Morning", "Rotation 2"),
/// grouping events that run together. Optional: small competitions can
/// skip sessions entirely and just order events directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub competition_id: CompetitionId,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub order_index: i32,
}

impl Session {
    pub fn new(competition_id: CompetitionId, name: impl Into<String>, order_index: i32) -> Self {
        Self {
            id: SessionId::new(),
            competition_id,
            name: name.into(),
            start_time: None,
            order_index,
        }
    }
}
