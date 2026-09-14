use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, ParticipantId};

/// Extra gymnastics-only fields for a `Participant`. Most competitions
/// won't need this row at all -- `Participant` alone is enough -- but
/// it's here for things like competitive level (e.g. USAG level 8),
/// and for the TAAF-specific identifiers a roster import carries that
/// don't apply outside the sport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GymnastDetail {
    pub participant_id: ParticipantId,
    pub competition_id: CompetitionId,
    pub level: Option<String>,
    pub club: Option<String>,
    /// "TAAF Athlete Competition #" -- the number to use at meets
    /// (ProScore calls this same number just "Athlete #"), and the
    /// key roster re-imports upsert on.
    pub taaf_athlete_number: Option<String>,
    /// "TAAF Registration #" (e.g. "2026 TAAF Registration #").
    pub taaf_registration_number: Option<String>,
    /// "*PRELIMINARY* Qual State" column.
    pub qual_status: Option<String>,
    /// The roster's "A&T" column, stored verbatim. Confirmed with the
    /// coach: usually blank, and nobody's sure what it stands for.
    pub at_code: Option<String>,
}

impl GymnastDetail {
    pub fn new(participant_id: ParticipantId, competition_id: CompetitionId) -> Self {
        Self {
            participant_id,
            competition_id,
            level: None,
            club: None,
            taaf_athlete_number: None,
            taaf_registration_number: None,
            qual_status: None,
            at_code: None,
        }
    }
}
