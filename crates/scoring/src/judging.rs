//! Small, genuinely sport-agnostic helpers for combining a panel of
//! judges' raw numbers into one value. This is NOT a "universal
//! scoring engine" -- what counts as a valid score, how deductions
//! work, and what the final formula looks like is still entirely up
//! to each sport module. This is just the common "how do N judges'
//! numbers become 1 number" step that keeps showing up.

use crate::error::ScoringError;

/// Simple mean.
pub fn average(scores: &[f64]) -> Result<f64, ScoringError> {
    if scores.is_empty() {
        return Err(ScoringError::NoScores);
    }
    Ok(scores.iter().sum::<f64>() / scores.len() as f64)
}

/// Drops the single highest and single lowest score, then averages
/// the rest. Common in gymnastics E-panels and diving. Falls back to
/// a plain average when there are 2 or fewer scores.
pub fn average_drop_high_low(scores: &[f64]) -> Result<f64, ScoringError> {
    if scores.is_empty() {
        return Err(ScoringError::NoScores);
    }
    if scores.len() <= 2 {
        return average(scores);
    }
    let mut sorted: Vec<f64> = scores.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let trimmed = &sorted[1..sorted.len() - 1];
    average(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn average_of_empty_errors() {
        assert!(average(&[]).is_err());
    }

    #[test]
    fn average_drop_high_low_trims_extremes() {
        // drops 9.5 and 8.0, averages the middle three
        let result = average_drop_high_low(&[9.5, 9.0, 8.8, 9.1, 8.0]).unwrap();
        assert!((result - ((9.0 + 8.8 + 9.1) / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn average_drop_high_low_falls_back_with_two_scores() {
        let result = average_drop_high_low(&[8.0, 9.0]).unwrap();
        assert!((result - 8.5).abs() < 1e-9);
    }
}
