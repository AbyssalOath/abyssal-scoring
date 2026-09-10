use uuid::Uuid;

use crate::error::{DbError, DbResult};

pub fn parse_uuid(field: &'static str, s: &str) -> DbResult<Uuid> {
    Uuid::parse_str(s).map_err(|_| DbError::InvalidStoredValue {
        field,
        value: s.to_string(),
    })
}

pub fn parse_json(field: &'static str, s: &str) -> DbResult<serde_json::Value> {
    serde_json::from_str(s).map_err(|_| DbError::InvalidStoredValue {
        field,
        value: s.to_string(),
    })
}
