use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, ThemeId};

/// The sport determines which rules/scoring module and which
/// sport-specific tables apply. It does NOT control the generic
/// competition/event/participant/result infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sport {
    Gymnastics,
    Swimming,
    TrackAndField,
}

impl Sport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Sport::Gymnastics => "gymnastics",
            Sport::Swimming => "swimming",
            Sport::TrackAndField => "track_and_field",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "gymnastics" => Some(Sport::Gymnastics),
            "swimming" => Some(Sport::Swimming),
            "track_and_field" => Some(Sport::TrackAndField),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub id: CompetitionId,
    pub name: String,
    pub sport: Sport,
    pub date: NaiveDate,
    pub location: Option<String>,
    /// Falls back to the sport's default theme when unset.
    pub theme_id: Option<ThemeId>,
}

impl Competition {
    pub fn new(name: impl Into<String>, sport: Sport, date: NaiveDate) -> Self {
        Self {
            id: CompetitionId::new(),
            name: name.into(),
            sport,
            date,
            location: None,
            theme_id: None,
        }
    }
}
