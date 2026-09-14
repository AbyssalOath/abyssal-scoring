use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use server::ServiceError;

/// Maps `ServiceError` to an HTTP status + a small JSON error body --
/// the same errors the desktop UI sees flattened to a string, given a
/// real status code here since HTTP clients (an iPad app in
/// particular) need to distinguish "bad request"/"unauthorized" from
/// "something broke" to behave sensibly (e.g. re-pairing on 401 vs.
/// just showing an error on 500).
pub struct ApiError(pub ServiceError);

impl From<ServiceError> for ApiError {
    fn from(e: ServiceError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            ServiceError::PairingCodeInvalid | ServiceError::PairingCodeExpired => {
                StatusCode::BAD_REQUEST
            }
            ServiceError::Db(db::DbError::NotFound) => StatusCode::NOT_FOUND,
            ServiceError::Scoring(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}
