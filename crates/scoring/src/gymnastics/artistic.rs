//! Artistic gymnastics scoring: final score = Difficulty (D) plus
//! Execution (E) minus Neutral Deductions (ND). This is the Code of
//! Points shape; other disciplines within gymnastics (rhythmic,
//! trampoline) would get their own sibling module rather than being
//! squeezed into this one.

use models::gymnastics::Deduction;

use crate::error::ScoringError;

#[derive(Debug, Clone)]
pub struct RoutineInput {
    pub difficulty_score: f64,
    /// Usually 10.0, but kept configurable (e.g. some age/level
    /// programs start execution below 10.0).
    pub execution_start_value: f64,
    pub deductions: Vec<Deduction>,
    pub neutral_deductions: f64,
}

impl RoutineInput {
    pub fn new(difficulty_score: f64) -> Self {
        Self {
            difficulty_score,
            execution_start_value: 10.0,
            deductions: Vec::new(),
            neutral_deductions: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutineScore {
    pub difficulty_score: f64,
    pub execution_score: f64,
    pub neutral_deductions: f64,
    pub final_score: f64,
}

pub fn calculate(input: &RoutineInput) -> Result<RoutineScore, ScoringError> {
    if input.difficulty_score < 0.0 {
        return Err(ScoringError::NegativeDifficulty);
    }
    if input.execution_start_value <= 0.0 {
        return Err(ScoringError::InvalidStartValue);
    }
    for d in &input.deductions {
        if d.value < 0.0 {
            return Err(ScoringError::NegativeDeduction(d.value));
        }
    }

    let total_deductions: f64 = input.deductions.iter().map(|d| d.value).sum();
    let execution_score = (input.execution_start_value - total_deductions).max(0.0);
    let final_score =
        (input.difficulty_score + execution_score - input.neutral_deductions).max(0.0);

    Ok(RoutineScore {
        difficulty_score: input.difficulty_score,
        execution_score: round2(execution_score),
        neutral_deductions: input.neutral_deductions,
        final_score: round2(final_score),
    })
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// One judge's raw deductions, combined with the others on the panel
/// via drop-high-low averaging -- the same technique
/// `scoring::judging` already offers, applied to a whole routine
/// instead of one already-computed number.
#[derive(Debug, Clone)]
pub struct JudgePanelInput {
    pub difficulty_score: f64,
    pub execution_start_value: f64,
    pub judge_deductions: Vec<Vec<Deduction>>,
    pub neutral_deductions: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanelRoutineScore {
    pub difficulty_score: f64,
    pub judge_execution_scores: Vec<f64>,
    pub execution_score: f64,
    pub neutral_deductions: f64,
    pub final_score: f64,
}

pub fn calculate_from_panel(input: &JudgePanelInput) -> Result<PanelRoutineScore, ScoringError> {
    if input.difficulty_score < 0.0 {
        return Err(ScoringError::NegativeDifficulty);
    }
    if input.execution_start_value <= 0.0 {
        return Err(ScoringError::InvalidStartValue);
    }
    if input.judge_deductions.is_empty() {
        return Err(ScoringError::NoScores);
    }

    let mut judge_execution_scores = Vec::with_capacity(input.judge_deductions.len());
    for deductions in &input.judge_deductions {
        for d in deductions {
            if d.value < 0.0 {
                return Err(ScoringError::NegativeDeduction(d.value));
            }
        }
        let total: f64 = deductions.iter().map(|d| d.value).sum();
        judge_execution_scores.push((input.execution_start_value - total).max(0.0));
    }

    let execution_score = crate::judging::average_drop_high_low(&judge_execution_scores)?;
    let final_score =
        (input.difficulty_score + execution_score - input.neutral_deductions).max(0.0);

    Ok(PanelRoutineScore {
        difficulty_score: input.difficulty_score,
        judge_execution_scores,
        execution_score: round2(execution_score),
        neutral_deductions: input.neutral_deductions,
        final_score: round2(final_score),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_routine_sums_difficulty_and_full_execution() {
        let input = RoutineInput::new(5.2);
        let result = calculate(&input).unwrap();
        assert_eq!(result.execution_score, 10.0);
        assert_eq!(result.final_score, 15.2);
    }

    #[test]
    fn deductions_reduce_execution_and_final_score() {
        let mut input = RoutineInput::new(5.6);
        input
            .deductions
            .push(Deduction::new("step on landing", 0.1));
        input.deductions.push(Deduction::new("bent knees", 0.3));
        let result = calculate(&input).unwrap();
        assert_eq!(result.execution_score, 9.6);
        assert_eq!(result.final_score, 15.2);
    }

    #[test]
    fn neutral_deductions_apply_after_execution() {
        let mut input = RoutineInput::new(5.0);
        input.neutral_deductions = 1.0; // e.g. out of bounds / late salute
        let result = calculate(&input).unwrap();
        assert_eq!(result.final_score, 14.0);
    }

    #[test]
    fn execution_score_cannot_go_negative() {
        let mut input = RoutineInput::new(4.0);
        input.deductions.push(Deduction::new("fall", 12.0));
        let result = calculate(&input).unwrap();
        assert_eq!(result.execution_score, 0.0);
        assert_eq!(result.final_score, 4.0);
    }

    #[test]
    fn negative_difficulty_is_rejected() {
        let input = RoutineInput::new(-1.0);
        assert!(matches!(
            calculate(&input),
            Err(ScoringError::NegativeDifficulty)
        ));
    }

    #[test]
    fn panel_drops_high_and_low_then_combines_with_difficulty() {
        let input = JudgePanelInput {
            difficulty_score: 5.0,
            execution_start_value: 10.0,
            judge_deductions: vec![
                vec![Deduction::new("a", 0.1)], // 9.9
                vec![Deduction::new("b", 0.3)], // 9.7 (dropped: low)
                vec![Deduction::new("c", 0.2)], // 9.8
                vec![],                         // 10.0 (dropped: high)
            ],
            neutral_deductions: 0.0,
        };
        let result = calculate_from_panel(&input).unwrap();
        // average of the middle two (9.9, 9.8)
        assert_eq!(result.execution_score, 9.85);
        assert_eq!(result.final_score, 14.85);
    }

    #[test]
    fn panel_requires_at_least_one_judge() {
        let input = JudgePanelInput {
            difficulty_score: 5.0,
            execution_start_value: 10.0,
            judge_deductions: vec![],
            neutral_deductions: 0.0,
        };
        assert!(matches!(
            calculate_from_panel(&input),
            Err(ScoringError::NoScores)
        ));
    }
}
