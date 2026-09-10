//! Swimming (and, unchanged, track) scoring: lower elapsed time wins,
//! full stop. No difficulty, no execution, no deductions -- proving
//! that a second sport doesn't have to be squeezed into gymnastics'
//! shape. `models`/`db` don't gain a single swimming-specific field
//! for this: `EventResult.detail` already holds whatever a sport
//! wants to put there.

use crate::error::ScoringError;

#[derive(Debug, Clone)]
pub struct TimeTrialInput {
    pub elapsed_seconds: f64,
    /// An added time penalty (false start, illegal turn, etc.) -- on
    /// top of the raw time, not subtracted like a gymnastics
    /// deduction.
    pub time_penalty_seconds: f64,
}

impl TimeTrialInput {
    pub fn new(elapsed_seconds: f64) -> Self {
        Self {
            elapsed_seconds,
            time_penalty_seconds: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeTrialResult {
    pub raw_seconds: f64,
    pub penalty_seconds: f64,
    pub final_seconds: f64,
}

pub fn calculate(input: &TimeTrialInput) -> Result<TimeTrialResult, ScoringError> {
    if input.elapsed_seconds < 0.0 {
        return Err(ScoringError::NegativeElapsedTime);
    }
    if input.time_penalty_seconds < 0.0 {
        return Err(ScoringError::NegativePenalty(input.time_penalty_seconds));
    }

    let final_seconds = input.elapsed_seconds + input.time_penalty_seconds;
    Ok(TimeTrialResult {
        raw_seconds: round3(input.elapsed_seconds),
        penalty_seconds: round3(input.time_penalty_seconds),
        final_seconds: round3(final_seconds),
    })
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_time_passes_through() {
        let result = calculate(&TimeTrialInput::new(52.31)).unwrap();
        assert_eq!(result.final_seconds, 52.31);
    }

    #[test]
    fn penalty_adds_to_raw_time() {
        let mut input = TimeTrialInput::new(52.31);
        input.time_penalty_seconds = 0.5;
        let result = calculate(&input).unwrap();
        assert_eq!(result.final_seconds, 52.81);
    }

    #[test]
    fn negative_time_is_rejected() {
        assert!(matches!(
            calculate(&TimeTrialInput::new(-1.0)),
            Err(ScoringError::NegativeElapsedTime)
        ));
    }

    #[test]
    fn negative_penalty_is_rejected() {
        let mut input = TimeTrialInput::new(52.31);
        input.time_penalty_seconds = -0.5;
        assert!(matches!(
            calculate(&input),
            Err(ScoringError::NegativePenalty(_))
        ));
    }
}
