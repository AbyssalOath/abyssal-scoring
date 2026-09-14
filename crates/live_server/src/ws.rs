use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use models::{CompetitionId, ScoreboardState};
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;

/// Matches the desktop scoreboard display's own poll interval
/// (`frontend/scoreboard.js`) -- no reason for a remote client to
/// demand fresher data than the TV itself gets.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsMessage {
    ScoreboardState { payload: ScoreboardState },
}

/// Public (no auth) -- push feed of the live scoreboard state for a
/// competition, for judge and spectator apps alike. Not a shared
/// `tokio::sync::broadcast` channel: this polls `AppService` per
/// connection and only sends when `updated_at` changes, which picks
/// up state changes regardless of whether they came from a judge's
/// iPad (via this API) or the desktop control panel (which never
/// talks to this crate at all) with no extra plumbing on either side.
pub async fn scoreboard_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(competition_id): Path<Uuid>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, CompetitionId(competition_id)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, competition_id: CompetitionId) {
    let mut last_updated_at = None;

    loop {
        match state.service.get_scoreboard_state(competition_id).await {
            Ok(scoreboard) => {
                if last_updated_at != Some(scoreboard.updated_at) {
                    last_updated_at = Some(scoreboard.updated_at);
                    let message = WsMessage::ScoreboardState {
                        payload: scoreboard,
                    };
                    let Ok(text) = serde_json::to_string(&message) else {
                        break;
                    };
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(_) => break,
        }

        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            incoming = socket.recv() => {
                match incoming {
                    None | Some(Ok(Message::Close(_))) | Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}
