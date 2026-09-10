use tauri::Manager;

use models::gymnastics::Apparatus;
use models::{
    Competition, CompetitionId, EventId, Judge, JudgeRole, Participant, ParticipantId,
    ScoreboardState, ScoreboardTheme, Session, Sport, Team, TeamId,
};

use crate::AppState;

/// Every command maps `ServiceError` -> `String` because Tauri needs
/// the error type to be `Serialize`; the detail still reaches the
/// frontend via `err.message` in the rejected promise.
type CmdResult<T> = Result<T, String>;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn list_competitions(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Competition>> {
    let service = state.service.lock().await;
    service.list_competitions().await.map_err(map_err)
}

#[tauri::command]
pub async fn create_competition(
    state: tauri::State<'_, AppState>,
    name: String,
    sport: Sport,
    date: chrono::NaiveDate,
) -> CmdResult<Competition> {
    let service = state.service.lock().await;
    service
        .create_competition(&name, sport, date)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn add_team(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    name: String,
) -> CmdResult<Team> {
    let service = state.service.lock().await;
    service
        .add_team(competition_id, &name)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn set_team_colors(
    state: tauri::State<'_, AppState>,
    team_id: TeamId,
    primary: Option<String>,
    secondary: Option<String>,
    logo_path: Option<String>,
) -> CmdResult<()> {
    let service = state.service.lock().await;
    service
        .set_team_colors(team_id, primary, secondary, logo_path)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn add_participant(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    first_name: String,
    last_name: String,
) -> CmdResult<Participant> {
    let service = state.service.lock().await;
    service
        .add_participant(competition_id, &first_name, &last_name)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn add_judge(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    name: String,
    role: JudgeRole,
) -> CmdResult<Judge> {
    let service = state.service.lock().await;
    service
        .add_judge(competition_id, &name, role)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn add_session(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    name: String,
    order_index: i32,
) -> CmdResult<Session> {
    let service = state.service.lock().await;
    service
        .add_session(competition_id, &name, order_index)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn add_event(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    name: String,
    order_index: i32,
) -> CmdResult<models::Event> {
    let service = state.service.lock().await;
    service
        .add_event(competition_id, &name, order_index)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn list_teams(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<Vec<Team>> {
    let service = state.service.lock().await;
    service.list_teams(competition_id).await.map_err(map_err)
}

#[tauri::command]
pub async fn list_participants(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<Vec<Participant>> {
    let service = state.service.lock().await;
    service
        .list_participants(competition_id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn list_judges(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<Vec<Judge>> {
    let service = state.service.lock().await;
    service.list_judges(competition_id).await.map_err(map_err)
}

#[tauri::command]
pub async fn list_events(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<Vec<models::Event>> {
    let service = state.service.lock().await;
    service.list_events(competition_id).await.map_err(map_err)
}

#[tauri::command]
pub async fn get_scoreboard_state(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<ScoreboardState> {
    let service = state.service.lock().await;
    service
        .get_scoreboard_state(competition_id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn set_current_participant(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    event_id: EventId,
    participant_id: ParticipantId,
) -> CmdResult<ScoreboardState> {
    let service = state.service.lock().await;
    service
        .set_current_participant(competition_id, event_id, participant_id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn show_result_on_scoreboard(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    event_id: EventId,
    participant_id: ParticipantId,
    live_value: serde_json::Value,
) -> CmdResult<ScoreboardState> {
    let service = state.service.lock().await;
    service
        .show_result_on_scoreboard(competition_id, event_id, participant_id, live_value)
        .await
        .map_err(map_err)
}

/// "Clear scoreboard for the next event" button. Only resets the live
/// display pointer -- see `AppService::clear_scoreboard`.
#[tauri::command]
pub async fn clear_scoreboard(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<ScoreboardState> {
    let service = state.service.lock().await;
    service
        .clear_scoreboard(competition_id)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn submit_gymnastics_panel_score(
    state: tauri::State<'_, AppState>,
    event_id: EventId,
    participant_id: ParticipantId,
    apparatus: Apparatus,
    difficulty_score: f64,
    judge_scores: Vec<server::JudgeDeductions>,
    neutral_deductions: f64,
) -> CmdResult<models::EventResult> {
    let service = state.service.lock().await;
    let (result, _routine, _scores) = service
        .submit_gymnastics_panel_score(
            event_id,
            participant_id,
            apparatus,
            difficulty_score,
            judge_scores,
            neutral_deductions,
        )
        .await
        .map_err(map_err)?;
    Ok(result)
}

#[tauri::command]
pub async fn submit_swimming_result(
    state: tauri::State<'_, AppState>,
    event_id: EventId,
    participant_id: ParticipantId,
    elapsed_seconds: f64,
    time_penalty_seconds: f64,
) -> CmdResult<models::EventResult> {
    let service = state.service.lock().await;
    service
        .submit_swimming_result(
            event_id,
            participant_id,
            elapsed_seconds,
            time_penalty_seconds,
        )
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn get_participant(
    state: tauri::State<'_, AppState>,
    id: ParticipantId,
) -> CmdResult<Participant> {
    let service = state.service.lock().await;
    service.get_participant(id).await.map_err(map_err)
}

#[tauri::command]
pub async fn get_event(state: tauri::State<'_, AppState>, id: EventId) -> CmdResult<models::Event> {
    let service = state.service.lock().await;
    service.get_event(id).await.map_err(map_err)
}

#[tauri::command]
pub async fn resolve_theme(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
) -> CmdResult<ScoreboardTheme> {
    let service = state.service.lock().await;
    service.resolve_theme(competition_id).await.map_err(map_err)
}

#[tauri::command]
pub async fn set_theme(
    state: tauri::State<'_, AppState>,
    competition_id: CompetitionId,
    theme: ScoreboardTheme,
) -> CmdResult<()> {
    let service = state.service.lock().await;
    service
        .set_theme(competition_id, theme)
        .await
        .map_err(map_err)
}

#[tauri::command]
pub async fn export_data(state: tauri::State<'_, AppState>, path: String) -> CmdResult<()> {
    let service = state.service.lock().await;
    service.export_to(&path).await.map_err(map_err)
}

#[tauri::command]
pub async fn import_data(
    state: tauri::State<'_, AppState>,
    path: String,
    mode: String,
) -> CmdResult<()> {
    let import_mode = match mode.as_str() {
        "duplicate" => db::export::ImportMode::DuplicateAsNew,
        _ => db::export::ImportMode::Replace,
    };
    let service = state.service.lock().await;
    service
        .import_from(&path, import_mode)
        .await
        .map_err(map_err)
}

/// Opens (or focuses, if already open) the scoreboard display window
/// -- the thing you'd drag onto a second monitor or a projector, kept
/// separate from the control panel window.
#[tauri::command]
pub async fn open_scoreboard_window(app: tauri::AppHandle, fullscreen: bool) -> CmdResult<()> {
    if let Some(window) = app.get_webview_window("scoreboard") {
        window.set_focus().map_err(map_err)?;
        return Ok(());
    }
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "scoreboard",
        tauri::WebviewUrl::App("scoreboard.html".into()),
    )
    .title("Abyssal Scoring - Scoreboard")
    .inner_size(1280.0, 720.0)
    .build()
    .map_err(map_err)?;
    window.set_fullscreen(fullscreen).map_err(map_err)?;
    Ok(())
}
