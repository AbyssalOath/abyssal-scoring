use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Db(#[from] db::DbError),
    #[error(transparent)]
    Scoring(#[from] scoring::ScoringError),
    #[error("unsupported sport for this operation: {0:?}")]
    UnsupportedSport(models::Sport),
    #[error("routine not found for result")]
    RoutineNotFound,
    #[error(transparent)]
    RosterImport(#[from] roster_import::RosterImportError),
    #[error("pairing code not found or already used")]
    PairingCodeInvalid,
    #[error("pairing code expired")]
    PairingCodeExpired,
    #[error("invalid or revoked device token")]
    Unauthorized,
}

pub type ServiceResult<T> = Result<T, ServiceError>;
