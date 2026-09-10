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
}

pub type ServiceResult<T> = Result<T, ServiceError>;
