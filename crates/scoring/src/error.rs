use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoringError {
    #[error("difficulty score cannot be negative")]
    NegativeDifficulty,
    #[error("execution start value must be positive")]
    InvalidStartValue,
    #[error("deduction cannot be negative: {0}")]
    NegativeDeduction(f64),
    #[error("no scores were provided to combine")]
    NoScores,
    #[error("elapsed time cannot be negative")]
    NegativeElapsedTime,
    #[error("time penalty cannot be negative: {0}")]
    NegativePenalty(f64),
}
