//! Embedded HTTP + WebSocket API so judges' iPads (and, read-only,
//! parent/spectator devices) can talk to whichever machine is running
//! Abyssal Scoring, over the local network. Deliberately a separate
//! crate from `server` so axum/tokio-networking dependencies don't
//! leak into the orchestration layer, and so this can be unit tested
//! without booting Tauri at all.
//!
//! LAN-only by design: there's no port-forwarding/relay/public TLS
//! here. True internet-wide remote viewing would need a hosted relay
//! component -- a materially different, non-local-first architecture
//! -- and is out of scope for this crate.

mod auth;
mod error;
mod routes;
mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use server::AppService;

pub use error::ApiError;

/// Arbitrary but fixed, so the pairing screen can tell a judge "port
/// 7878" without the desktop needing to first ask the OS what port it
/// got and relay that back through another round trip.
pub const DEFAULT_PORT: u16 = 7878;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<AppService>,
}

pub fn router(service: Arc<AppService>) -> Router {
    Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/competitions", get(routes::list_competitions))
        .route("/api/v1/pair", post(routes::pair))
        .route(
            "/api/v1/competitions/{competition_id}/events",
            get(routes::list_events),
        )
        .route(
            "/api/v1/competitions/{competition_id}/participants",
            get(routes::list_participants),
        )
        .route(
            "/api/v1/scores/gymnastics-panel",
            post(routes::submit_gymnastics_score),
        )
        .route(
            "/api/v1/scores/swimming",
            post(routes::submit_swimming_score),
        )
        .route(
            "/api/v1/scoreboard/{competition_id}",
            get(routes::get_scoreboard),
        )
        .route(
            "/api/v1/ws/scoreboard/{competition_id}",
            get(ws::scoreboard_ws),
        )
        .with_state(AppState { service })
}

/// Binds and serves forever. Meant to be `tauri::async_runtime::spawn`ed
/// from the desktop app's `setup()`, reusing the tokio runtime Tauri
/// already owns rather than starting a second one.
pub async fn serve(service: Arc<AppService>, addr: SocketAddr) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router(service)).await
}

/// Best-effort LAN IP to show on the pairing screen -- `None` if it
/// can't be determined, in which case the coach can find it some
/// other way (System Settings, `ipconfig`/`ip addr`).
pub fn local_lan_ip() -> Option<String> {
    local_ip_address::local_ip().ok().map(|ip| ip.to_string())
}
