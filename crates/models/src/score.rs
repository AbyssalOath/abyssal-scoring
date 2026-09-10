use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{JudgeId, ResultId, ScoreId};

/// One judge's/official's raw submission toward a result. A result can
/// have many scores (e.g. an execution panel of 3 judges); the scoring
/// crate decides how they combine (average, drop-high-low, sum, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub id: ScoreId,
    pub result_id: ResultId,
    pub judge_id: Option<JudgeId>,
    pub value: f64,
    /// Sport-specific breakdown behind this single value, e.g.
    /// individual deductions taken, split times, etc.
    pub components: serde_json::Value,
    pub submitted_at: DateTime<Utc>,
}

impl Score {
    pub fn new(result_id: ResultId, judge_id: Option<JudgeId>, value: f64) -> Self {
        Self {
            id: ScoreId::new(),
            result_id,
            judge_id,
            value,
            components: serde_json::Value::Null,
            submitted_at: Utc::now(),
        }
    }
}
