use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, TeamId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub competition_id: CompetitionId,
    pub name: String,
    pub abbreviation: Option<String>,
    /// Hex color, e.g. "#c8102e". Used to drive the scoreboard theme
    /// when a competition wants a per-team color split rather than
    /// (or in addition to) a single competition-wide theme.
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    /// Path to a logo image on disk (copied into the app's data dir),
    /// used on the scoreboard background/header.
    pub logo_path: Option<String>,
}

impl Team {
    pub fn new(competition_id: CompetitionId, name: impl Into<String>) -> Self {
        Self {
            id: TeamId::new(),
            competition_id,
            name: name.into(),
            abbreviation: None,
            primary_color: None,
            secondary_color: None,
            logo_path: None,
        }
    }
}
