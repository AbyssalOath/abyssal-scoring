use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, EventId, SessionId};

/// A single scored event within a competition, e.g. "Women's Floor
/// Exercise" or "100m Freestyle". Sport-agnostic: what makes it
/// gymnastics vs. swimming lives in the scoring crate and in the
/// sport-specific result payload, not here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub competition_id: CompetitionId,
    pub session_id: Option<SessionId>,
    pub name: String,
    /// Display order on the schedule / rotation order.
    pub order_index: i32,
    pub status: EventStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Scheduled,
    InProgress,
    Completed,
}

impl Event {
    pub fn new(competition_id: CompetitionId, name: impl Into<String>, order_index: i32) -> Self {
        Self {
            id: EventId::new(),
            competition_id,
            session_id: None,
            name: name.into(),
            order_index,
            status: EventStatus::Scheduled,
        }
    }
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventStatus::Scheduled => "scheduled",
            EventStatus::InProgress => "in_progress",
            EventStatus::Completed => "completed",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "scheduled" => Some(EventStatus::Scheduled),
            "in_progress" => Some(EventStatus::InProgress),
            "completed" => Some(EventStatus::Completed),
            _ => None,
        }
    }
}
