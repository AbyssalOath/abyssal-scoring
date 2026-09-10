use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deduction {
    pub reason: String,
    pub value: f64,
}

impl Deduction {
    pub fn new(reason: impl Into<String>, value: f64) -> Self {
        Self {
            reason: reason.into(),
            value,
        }
    }
}
