use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, EventId, ParticipantId};

/// The single "what's on the display right now" pointer for a
/// competition. This is the ONLY thing "Clear scoreboard" touches --
/// events, participants, judges, results and scores are untouched.
/// Conceptually a singleton row per competition, upserted in place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreboardState {
    pub competition_id: CompetitionId,
    pub current_event_id: Option<EventId>,
    pub current_participant_id: Option<ParticipantId>,
    pub display_mode: DisplayMode,
    /// In-progress, not-yet-submitted score components being built up
    /// live on the control panel (e.g. difficulty entered, execution
    /// still being judged). Never written to `results`/`scores` until
    /// explicitly submitted.
    pub live_value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    /// Nothing live -- e.g. showing a title/sponsor screen.
    Idle,
    InProgress,
    ShowingResult,
    ShowingRanking,
}

impl ScoreboardState {
    pub fn idle(competition_id: CompetitionId) -> Self {
        Self {
            competition_id,
            current_event_id: None,
            current_participant_id: None,
            display_mode: DisplayMode::Idle,
            live_value: serde_json::Value::Null,
            updated_at: Utc::now(),
        }
    }

    /// "Clear scoreboard for the next event": drops the live pointer
    /// back to idle but keeps `competition_id` (and touches nothing
    /// else in the database).
    pub fn clear(&mut self) {
        self.current_event_id = None;
        self.current_participant_id = None;
        self.display_mode = DisplayMode::Idle;
        self.live_value = serde_json::Value::Null;
        self.updated_at = Utc::now();
    }
}

impl DisplayMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayMode::Idle => "idle",
            DisplayMode::InProgress => "in_progress",
            DisplayMode::ShowingResult => "showing_result",
            DisplayMode::ShowingRanking => "showing_ranking",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(DisplayMode::Idle),
            "in_progress" => Some(DisplayMode::InProgress),
            "showing_result" => Some(DisplayMode::ShowingResult),
            "showing_ranking" => Some(DisplayMode::ShowingRanking),
            _ => None,
        }
    }
}
