use serde::{Deserialize, Serialize};

use crate::ids::ParticipantId;

/// Extra gymnastics-only fields for a `Participant`. Most competitions
/// won't need this row at all -- `Participant` alone is enough -- but
/// it's here for things like competitive level (e.g. USAG level 8) that
/// don't apply outside the sport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GymnastDetail {
    pub participant_id: ParticipantId,
    pub level: Option<String>,
    pub club: Option<String>,
}
