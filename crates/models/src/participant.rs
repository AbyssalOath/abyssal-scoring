use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, ParticipantId, TeamId};

/// A competitor. Gymnastics UI can label this "Gymnast", swimming UI
/// can label it "Swimmer" -- the underlying model stays generic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub competition_id: CompetitionId,
    pub first_name: String,
    pub last_name: String,
    pub team_id: Option<TeamId>,
    /// Bib / lane assignment / competitor number, sport-agnostic label.
    pub bib_number: Option<String>,
    pub level_or_category: Option<String>,
}

impl Participant {
    pub fn new(
        competition_id: CompetitionId,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
    ) -> Self {
        Self {
            id: ParticipantId::new(),
            competition_id,
            first_name: first_name.into(),
            last_name: last_name.into(),
            team_id: None,
            bib_number: None,
            level_or_category: None,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
