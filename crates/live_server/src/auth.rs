use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use models::JudgeDevice;

use crate::AppState;

/// Extracts and validates the bearer token on an authenticated
/// live-scoring request, resolving it to the `JudgeDevice` it belongs
/// to. Any handler that takes this as a parameter is authenticated
/// automatically -- the request never reaches the handler body
/// otherwise.
pub struct AuthedDevice(pub JudgeDevice);

impl FromRequestParts<AppState> for AuthedDevice {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "missing Authorization header"))?;
        let token = header
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "expected a Bearer token"))?;
        let device = state
            .service
            .authenticate_device(token)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or revoked device token"))?;
        Ok(AuthedDevice(device))
    }
}
