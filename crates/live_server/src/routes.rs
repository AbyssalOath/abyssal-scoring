use axum::extract::{Path, State};
use axum::Json;
use models::gymnastics::Apparatus;
use models::{
    Competition, CompetitionId, Event, EventId, EventResult, Judge, Participant, ParticipantId,
};
use serde::{Deserialize, Serialize};
use server::JudgeDeductions;
use uuid::Uuid;

use crate::auth::AuthedDevice;
use crate::error::ApiError;
use crate::AppState;

pub async fn health() -> &'static str {
    "ok"
}

/// Public -- lets a spectator/parent app show a competition picker
/// without needing to know a competition id up front. Same data a
/// judge's device would already be trusted to see, so no reason to
/// gate it behind auth.
pub async fn list_competitions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Competition>>, ApiError> {
    let competitions = state.service.list_competitions().await?;
    Ok(Json(competitions))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairRequest {
    pub code: String,
    pub device_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResponse {
    pub token: String,
    pub judge: Judge,
}

/// Exchanges a pairing code (shown on the desktop) for a device
/// token. Not authenticated -- the code itself is the credential.
pub async fn pair(
    State(state): State<AppState>,
    Json(body): Json<PairRequest>,
) -> Result<Json<PairResponse>, ApiError> {
    let (token, _device, judge) = state
        .service
        .redeem_pairing_code(&body.code, &body.device_name)
        .await?;
    Ok(Json(PairResponse { token, judge }))
}

/// Public -- the schedule is already visible on the TV/rotation sheet
/// at the venue, and a spectator app needs it to label the scoreboard
/// (`current_event_id` alone isn't human-readable).
pub async fn list_events(
    State(state): State<AppState>,
    Path(competition_id): Path<Uuid>,
) -> Result<Json<Vec<Event>>, ApiError> {
    let events = state
        .service
        .list_events(CompetitionId(competition_id))
        .await?;
    Ok(Json(events))
}

/// Public -- same reasoning as `list_events`: a spectator app needs
/// participant names to label the scoreboard, and these are already
/// announced/displayed publicly at the meet.
pub async fn list_participants(
    State(state): State<AppState>,
    Path(competition_id): Path<Uuid>,
) -> Result<Json<Vec<Participant>>, ApiError> {
    let participants = state
        .service
        .list_participants(CompetitionId(competition_id))
        .await?;
    Ok(Json(participants))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GymnasticsScoreRequest {
    pub event_id: Uuid,
    pub participant_id: Uuid,
    pub apparatus: Apparatus,
    pub difficulty_score: f64,
    pub judge_scores: Vec<JudgeDeductions>,
    pub neutral_deductions: f64,
}

/// A judge's iPad submits a full panel score in one request -- the
/// same shape (and the same `AppService` method, so the same scoring
/// math and ranking recompute) as the desktop control panel uses.
/// Mirrors the desktop's "Submit score" button exactly: computing the
/// result and putting it up on the live scoreboard are one action
/// there, so they're one action here too, rather than requiring a
/// separate "now show it" step from either the iPad or a human at the
/// desktop.
pub async fn submit_gymnastics_score(
    State(state): State<AppState>,
    AuthedDevice(device): AuthedDevice,
    Json(mut body): Json<GymnasticsScoreRequest>,
) -> Result<Json<EventResult>, ApiError> {
    // A single judge submitting from their own paired device doesn't
    // need to also pass their own judge id -- the auth token already
    // identifies them. Multi-judge submissions (e.g. a head judge's
    // iPad relaying a whole panel) still pass `judgeId` per entry
    // explicitly, so this only fills in the unambiguous case.
    if let [entry] = body.judge_scores.as_mut_slice() {
        entry.judge_id.get_or_insert(device.judge_id);
    }

    let event_id = EventId(body.event_id);
    let participant_id = ParticipantId(body.participant_id);

    let (result, _routine, _judge_scores) = state
        .service
        .submit_gymnastics_panel_score(
            event_id,
            participant_id,
            body.apparatus,
            body.difficulty_score,
            body.judge_scores,
            body.neutral_deductions,
        )
        .await?;

    let event = state.service.get_event(event_id).await?;
    state
        .service
        .show_result_on_scoreboard(
            event.competition_id,
            event_id,
            participant_id,
            result.detail.clone(),
        )
        .await?;

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwimmingScoreRequest {
    pub event_id: Uuid,
    pub participant_id: Uuid,
    pub elapsed_seconds: f64,
    pub time_penalty_seconds: f64,
}

pub async fn submit_swimming_score(
    State(state): State<AppState>,
    _auth: AuthedDevice,
    Json(body): Json<SwimmingScoreRequest>,
) -> Result<Json<EventResult>, ApiError> {
    let event_id = EventId(body.event_id);
    let participant_id = ParticipantId(body.participant_id);

    let result = state
        .service
        .submit_swimming_result(
            event_id,
            participant_id,
            body.elapsed_seconds,
            body.time_penalty_seconds,
        )
        .await?;

    let event = state.service.get_event(event_id).await?;
    state
        .service
        .show_result_on_scoreboard(
            event.competition_id,
            event_id,
            participant_id,
            result.detail.clone(),
        )
        .await?;

    Ok(Json(result))
}

/// Public (no auth) -- this is the same thing already shown on the TV
/// scoreboard, so a spectator/parent app reading it needs no
/// credential. Judges' apps can also poll/watch this to show "now
/// judging: X" without needing their own write path to see it.
pub async fn get_scoreboard(
    State(state): State<AppState>,
    Path(competition_id): Path<Uuid>,
) -> Result<Json<models::ScoreboardState>, ApiError> {
    let scoreboard = state
        .service
        .get_scoreboard_state(CompetitionId(competition_id))
        .await?;
    Ok(Json(scoreboard))
}
