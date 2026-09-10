use serde::{Deserialize, Serialize};

use super::apparatus::Apparatus;
use super::deduction::Deduction;
use crate::ids::ResultId;

/// Gymnastics-specific detail behind an `EventResult`. Keyed by
/// `result_id` so the generic `results`/`scores` tables never need to
/// know a routine exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Routine {
    pub result_id: ResultId,
    pub apparatus: Apparatus,
    pub difficulty_score: f64,
    pub execution_start_value: f64,
    pub deductions: Vec<Deduction>,
    pub neutral_deductions: f64,
}

impl Routine {
    pub fn new(result_id: ResultId, apparatus: Apparatus) -> Self {
        Self {
            result_id,
            apparatus,
            difficulty_score: 0.0,
            execution_start_value: 10.0,
            deductions: Vec::new(),
            neutral_deductions: 0.0,
        }
    }
}
