use serde::{Deserialize, Serialize};

use crate::ids::{CompetitionId, JudgeId};

/// Officiating role. Gymnastics splits panels into difficulty (D) and
/// execution (E) judges; swimming/track have timers and referees.
/// `Other` keeps this from having to be revisited for every new sport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgeRole {
    Head,
    Difficulty,
    Execution,
    Line,
    Referee,
    Timer,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judge {
    pub id: JudgeId,
    pub competition_id: CompetitionId,
    pub name: String,
    pub role: JudgeRole,
    /// Optional panel/station label, e.g. "Beam Panel", "Lane 4 Timer".
    pub assignment: Option<String>,
}

impl Judge {
    pub fn new(competition_id: CompetitionId, name: impl Into<String>, role: JudgeRole) -> Self {
        Self {
            id: JudgeId::new(),
            competition_id,
            name: name.into(),
            role,
            assignment: None,
        }
    }
}

impl JudgeRole {
    pub fn as_str(&self) -> String {
        match self {
            JudgeRole::Head => "head".to_string(),
            JudgeRole::Difficulty => "difficulty".to_string(),
            JudgeRole::Execution => "execution".to_string(),
            JudgeRole::Line => "line".to_string(),
            JudgeRole::Referee => "referee".to_string(),
            JudgeRole::Timer => "timer".to_string(),
            JudgeRole::Other(label) => format!("other:{}", label),
        }
    }

    pub fn from_stored(s: &str) -> Self {
        match s {
            "head" => JudgeRole::Head,
            "difficulty" => JudgeRole::Difficulty,
            "execution" => JudgeRole::Execution,
            "line" => JudgeRole::Line,
            "referee" => JudgeRole::Referee,
            "timer" => JudgeRole::Timer,
            other => JudgeRole::Other(other.strip_prefix("other:").unwrap_or(other).to_string()),
        }
    }
}
