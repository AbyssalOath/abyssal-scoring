use std::path::{Path, PathBuf};

use chrono::Utc;
use db::Db;
use models::gymnastics::{Apparatus, Deduction, GymnastDetail, Routine};
use models::{
    Competition, CompetitionId, DisplayMode, Event, EventId, EventResult, Judge, JudgeDevice,
    JudgeDeviceId, JudgeId, JudgeRole, PairingCode, Participant, ParticipantId, RankingEntry,
    Score, ScoreboardState, ScoreboardTheme, Session, Sport, Team, TeamId,
};
use rand::distributions::Alphanumeric;
use rand::Rng;
use roster_import::{ParsedRosterPreview, RosterRow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{ServiceError, ServiceResult};
use crate::ranking::{higher_is_better, rank_results};

/// Everything the Tauri commands / any future UI (desktop, tablet,
/// web) call through. Deliberately thin: it wires `db` + `scoring`
/// together and enforces the "server knows nothing about how scoring
/// works" rule -- it hands input to `scoring`, gets a number back, and
/// persists it.
pub struct AppService {
    pub db: Db,
    db_path: PathBuf,
}

/// What one judge on an execution panel submits: their own deduction
/// list. `judge_id` is optional so a solo scorer with no roster judge
/// assigned still works.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeDeductions {
    pub judge_id: Option<JudgeId>,
    pub deductions: Vec<Deduction>,
}

/// One row of the rotation sheet / roster view: a participant plus
/// whatever gymnastics-specific detail (TAAF #s, division, qual
/// status) has been recorded for them, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterEntry {
    pub participant: Participant,
    pub gymnast_detail: Option<GymnastDetail>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterImportSummary {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub warnings: Vec<String>,
}

impl AppService {
    pub async fn open(db_path: impl AsRef<Path>) -> ServiceResult<Self> {
        let db = Db::open(&db_path).await?;
        Ok(Self {
            db,
            db_path: db_path.as_ref().to_path_buf(),
        })
    }

    /// For tests (in this crate and downstream, e.g. `live_server`)
    /// that need a real `AppService` without a database file on disk.
    /// `export_to`/`import_from` won't resolve a sensible asset
    /// directory against an empty path, but nothing else cares.
    pub async fn open_in_memory() -> ServiceResult<Self> {
        Ok(Self {
            db: Db::open_in_memory().await?,
            db_path: PathBuf::new(),
        })
    }

    // ---- Setup: competitions, teams, participants, judges, schedule ----

    pub async fn create_competition(
        &self,
        name: &str,
        sport: Sport,
        date: chrono::NaiveDate,
    ) -> ServiceResult<Competition> {
        let competition = Competition::new(name, sport, date);
        self.db.create_competition(&competition).await?;
        self.db.get_or_init_scoreboard_state(competition.id).await?;
        Ok(competition)
    }

    pub async fn list_competitions(&self) -> ServiceResult<Vec<Competition>> {
        Ok(self.db.list_competitions().await?)
    }

    pub async fn add_team(&self, competition_id: CompetitionId, name: &str) -> ServiceResult<Team> {
        let team = Team::new(competition_id, name);
        self.db.create_team(&team).await?;
        Ok(team)
    }

    pub async fn set_team_colors(
        &self,
        team_id: TeamId,
        primary: Option<String>,
        secondary: Option<String>,
        logo_path: Option<String>,
    ) -> ServiceResult<()> {
        self.db
            .update_team_branding(team_id, primary, secondary, logo_path)
            .await?;
        Ok(())
    }

    pub async fn add_participant(
        &self,
        competition_id: CompetitionId,
        first_name: &str,
        last_name: &str,
    ) -> ServiceResult<Participant> {
        let participant = Participant::new(competition_id, first_name, last_name);
        self.db.create_participant(&participant).await?;
        Ok(participant)
    }

    pub async fn add_judge(
        &self,
        competition_id: CompetitionId,
        name: &str,
        role: JudgeRole,
    ) -> ServiceResult<Judge> {
        let judge = Judge::new(competition_id, name, role);
        self.db.create_judge(&judge).await?;
        Ok(judge)
    }

    pub async fn add_session(
        &self,
        competition_id: CompetitionId,
        name: &str,
        order_index: i32,
    ) -> ServiceResult<Session> {
        let session = Session::new(competition_id, name, order_index);
        self.db.create_session(&session).await?;
        Ok(session)
    }

    pub async fn add_event(
        &self,
        competition_id: CompetitionId,
        name: &str,
        order_index: i32,
    ) -> ServiceResult<Event> {
        let event = Event::new(competition_id, name, order_index);
        self.db.create_event(&event).await?;
        Ok(event)
    }

    // ---- Roster import ----

    /// Parses a roster file into a reviewable preview -- never writes
    /// anything to the database. Blocking (file IO + in-memory
    /// parsing only), so it's a plain function rather than `async`.
    pub fn parse_roster_file(&self, path: impl AsRef<Path>) -> ServiceResult<ParsedRosterPreview> {
        Ok(roster_import::parse_file(path)?)
    }

    /// Imports reviewed roster rows, upserting by TAAF athlete # so a
    /// weekly re-send of the same roster updates existing athletes
    /// instead of duplicating them. Rows with no TAAF # are always
    /// created fresh, since there's nothing to match them against.
    pub async fn import_roster(
        &self,
        competition_id: CompetitionId,
        rows: Vec<RosterRow>,
    ) -> ServiceResult<RosterImportSummary> {
        let mut summary = RosterImportSummary::default();

        for row in rows {
            let first_name = row.first_name.trim().to_string();
            let last_name = row.last_name.trim().to_string();
            if first_name.is_empty() || last_name.is_empty() {
                summary.skipped += 1;
                summary
                    .warnings
                    .push("Skipped a row with no first/last name.".to_string());
                continue;
            }

            let taaf_number = row
                .taaf_athlete_number
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);

            let existing = match &taaf_number {
                Some(number) => {
                    self.db
                        .find_gymnast_detail_by_taaf_number(competition_id, number)
                        .await?
                }
                None => None,
            };

            match existing {
                Some(mut detail) => {
                    let mut participant = self.db.get_participant(detail.participant_id).await?;
                    participant.first_name = first_name;
                    participant.last_name = last_name;
                    participant.level_or_category = row.division.clone();
                    participant.is_attending = row.is_attending;
                    self.db.update_participant(&participant).await?;

                    detail.club = row.gym.clone();
                    detail.taaf_registration_number = row.taaf_registration_number.clone();
                    detail.qual_status = row.qual_status.clone();
                    detail.at_code = row.at_code.clone();
                    self.db.update_gymnast_detail(&detail).await?;

                    summary.updated += 1;
                }
                None => {
                    let mut participant = Participant::new(competition_id, first_name, last_name);
                    participant.level_or_category = row.division.clone();
                    participant.is_attending = row.is_attending;
                    self.db.create_participant(&participant).await?;

                    let mut detail = GymnastDetail::new(participant.id, competition_id);
                    detail.club = row.gym.clone();
                    detail.taaf_athlete_number = taaf_number.clone();
                    detail.taaf_registration_number = row.taaf_registration_number.clone();
                    detail.qual_status = row.qual_status.clone();
                    detail.at_code = row.at_code.clone();
                    self.db.create_gymnast_detail(&detail).await?;

                    if taaf_number.is_none() {
                        summary.warnings.push(format!(
                            "{} has no TAAF athlete # -- a future re-import can't match this row automatically.",
                            participant.full_name()
                        ));
                    }
                    summary.created += 1;
                }
            }
        }

        Ok(summary)
    }

    /// Every participant in a competition alongside their gymnastics
    /// detail (if any) -- the rotation sheet's data source, and
    /// generally useful anywhere the roster needs to show TAAF/level
    /// fields instead of just name + bib.
    pub async fn list_roster_with_gymnast_detail(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<Vec<RosterEntry>> {
        let participants = self.db.list_participants(competition_id).await?;
        let mut entries = Vec::with_capacity(participants.len());
        for participant in participants {
            let gymnast_detail = match self.db.get_gymnast_detail(participant.id).await {
                Ok(detail) => Some(detail),
                Err(db::DbError::NotFound) => None,
                Err(e) => return Err(e.into()),
            };
            entries.push(RosterEntry {
                participant,
                gymnast_detail,
            });
        }
        Ok(entries)
    }

    // ---- Judge device pairing (networked live scoring) ----

    /// Generates a 10-minute pairing code for a judge to enter on
    /// their iPad. A 6-digit code space is more than enough headroom
    /// for the handful of judges concurrently pairing at a meet - no
    /// collision-retry loop is worth the complexity here.
    pub async fn create_pairing_code(&self, judge_id: JudgeId) -> ServiceResult<PairingCode> {
        let pairing = PairingCode {
            code: generate_pairing_code(),
            judge_id,
            expires_at: Utc::now() + chrono::Duration::minutes(10),
            consumed: false,
        };
        self.db.create_pairing_code(&pairing).await?;
        Ok(pairing)
    }

    /// Exchanges a still-valid pairing code for a long-lived device
    /// token. The plaintext token is only ever handed back here -
    /// only its hash is persisted, so it can't be recovered later even
    /// from the database file itself.
    pub async fn redeem_pairing_code(
        &self,
        code: &str,
        device_name: &str,
    ) -> ServiceResult<(String, JudgeDevice, Judge)> {
        let pairing = self.db.get_pairing_code(code).await.map_err(|e| match e {
            db::DbError::NotFound => ServiceError::PairingCodeInvalid,
            other => other.into(),
        })?;
        if pairing.consumed {
            return Err(ServiceError::PairingCodeInvalid);
        }
        if pairing.expires_at < Utc::now() {
            return Err(ServiceError::PairingCodeExpired);
        }
        self.db.consume_pairing_code(code).await?;

        let token = generate_device_token();
        let device = JudgeDevice {
            id: JudgeDeviceId::new(),
            judge_id: pairing.judge_id,
            device_name: device_name.to_string(),
            token_hash: hash_token(&token),
            paired_at: Utc::now(),
            last_seen_at: None,
        };
        self.db.create_judge_device(&device).await?;
        let judge = self.db.get_judge(pairing.judge_id).await?;

        Ok((token, device, judge))
    }

    /// Validates a bearer token presented by a paired device and
    /// records that it was just used. Called on every authenticated
    /// live-scoring API request.
    pub async fn authenticate_device(&self, token: &str) -> ServiceResult<JudgeDevice> {
        let device = self
            .db
            .find_judge_device_by_token_hash(&hash_token(token))
            .await?
            .ok_or(ServiceError::Unauthorized)?;
        self.db.touch_judge_device(device.id, Utc::now()).await?;
        Ok(device)
    }

    pub async fn list_judge_devices(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<Vec<JudgeDevice>> {
        Ok(self
            .db
            .list_judge_devices_for_competition(competition_id)
            .await?)
    }

    pub async fn revoke_judge_device(&self, id: JudgeDeviceId) -> ServiceResult<()> {
        Ok(self.db.revoke_judge_device(id).await?)
    }

    // ---- Live scoreboard: what's currently on screen ----

    pub async fn get_scoreboard_state(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<ScoreboardState> {
        Ok(self.db.get_or_init_scoreboard_state(competition_id).await?)
    }

    /// Puts a participant "up" on the board without any score yet.
    pub async fn set_current_participant(
        &self,
        competition_id: CompetitionId,
        event_id: EventId,
        participant_id: ParticipantId,
    ) -> ServiceResult<ScoreboardState> {
        let mut state = self.db.get_or_init_scoreboard_state(competition_id).await?;
        state.current_event_id = Some(event_id);
        state.current_participant_id = Some(participant_id);
        state.display_mode = DisplayMode::InProgress;
        state.live_value = serde_json::Value::Null;
        state.updated_at = Utc::now();
        self.db.upsert_scoreboard_state(&state).await?;
        Ok(state)
    }

    /// "Stays where they left off": just re-reads the current state --
    /// there's nothing to do on startup besides NOT clearing it.
    pub async fn resume_scoreboard(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<ScoreboardState> {
        self.get_scoreboard_state(competition_id).await
    }

    /// Puts a finished result on the display -- distinct from
    /// `set_current_participant`, which only announces who's up next
    /// with no score yet.
    pub async fn show_result_on_scoreboard(
        &self,
        competition_id: CompetitionId,
        event_id: EventId,
        participant_id: ParticipantId,
        live_value: serde_json::Value,
    ) -> ServiceResult<ScoreboardState> {
        let mut state = self.db.get_or_init_scoreboard_state(competition_id).await?;
        state.current_event_id = Some(event_id);
        state.current_participant_id = Some(participant_id);
        state.display_mode = DisplayMode::ShowingResult;
        state.live_value = live_value;
        state.updated_at = Utc::now();
        self.db.upsert_scoreboard_state(&state).await?;
        Ok(state)
    }

    /// "Clear for the next event": resets only the live pointer.
    /// Events, participants, judges, teams, results and scores are
    /// untouched.
    pub async fn clear_scoreboard(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<ScoreboardState> {
        Ok(self.db.clear_scoreboard(competition_id).await?)
    }

    // ---- Scoring (each sport gets its own `submit_*_score` method
    //      rather than one generic function) ----

    pub async fn submit_gymnastics_panel_score(
        &self,
        event_id: EventId,
        participant_id: ParticipantId,
        apparatus: Apparatus,
        difficulty_score: f64,
        judge_scores: Vec<JudgeDeductions>,
        neutral_deductions: f64,
    ) -> ServiceResult<(EventResult, Routine, Vec<f64>)> {
        let input = scoring::gymnastics::JudgePanelInput {
            difficulty_score,
            execution_start_value: 10.0,
            judge_deductions: judge_scores.iter().map(|j| j.deductions.clone()).collect(),
            neutral_deductions,
        };
        let calculated = scoring::gymnastics::calculate_from_panel(&input)?;

        let mut result = EventResult::new(event_id, participant_id);
        result.finalized = true;
        result.final_value = Some(calculated.final_score);
        result.detail = serde_json::json!({
            "difficulty": calculated.difficulty_score,
            "execution": calculated.execution_score,
            "judge_execution_scores": calculated.judge_execution_scores,
            "neutral_deductions": calculated.neutral_deductions,
            "final_score": calculated.final_score,
        });
        self.db.create_result(&result).await?;

        let all_deductions: Vec<Deduction> = judge_scores
            .iter()
            .flat_map(|j| j.deductions.clone())
            .collect();
        let routine = Routine {
            result_id: result.id,
            apparatus,
            difficulty_score,
            execution_start_value: input.execution_start_value,
            deductions: all_deductions,
            neutral_deductions,
        };
        self.db.create_gymnastics_routine(&routine).await?;

        for (judge, judge_score) in judge_scores
            .iter()
            .zip(calculated.judge_execution_scores.iter())
        {
            let score = Score::new(result.id, judge.judge_id, *judge_score);
            self.db.add_score(&score).await?;
        }

        self.recompute_rankings(event_id).await?;

        Ok((result, routine, calculated.judge_execution_scores))
    }

    pub async fn submit_swimming_result(
        &self,
        event_id: EventId,
        participant_id: ParticipantId,
        elapsed_seconds: f64,
        time_penalty_seconds: f64,
    ) -> ServiceResult<EventResult> {
        let input = scoring::swimming::TimeTrialInput {
            elapsed_seconds,
            time_penalty_seconds,
        };
        let calculated = scoring::swimming::calculate(&input)?;

        let mut result = EventResult::new(event_id, participant_id);
        result.finalized = true;
        result.final_value = Some(calculated.final_seconds);
        result.detail = serde_json::json!({
            "raw_seconds": calculated.raw_seconds,
            "penalty_seconds": calculated.penalty_seconds,
            "final_seconds": calculated.final_seconds,
        });
        self.db.create_result(&result).await?;

        let score = Score::new(result.id, None, calculated.final_seconds);
        self.db.add_score(&score).await?;

        self.recompute_rankings(event_id).await?;

        Ok(result)
    }

    /// Re-derives standings for an event from its finalized results.
    /// Cheap: rankings are just derived data, never the source of
    /// truth.
    pub async fn recompute_rankings(&self, event_id: EventId) -> ServiceResult<Vec<RankingEntry>> {
        let event = self.db.get_event(event_id).await?;
        let competition = self.db.get_competition(event.competition_id).await?;
        let results = self.db.list_results_for_event(event_id).await?;

        let entries: Vec<_> = results
            .iter()
            .filter(|r| r.finalized)
            .filter_map(|r| r.final_value.map(|v| (r.participant_id, r.team_id, v)))
            .collect();

        let ranked = rank_results(event_id, entries, higher_is_better(competition.sport));
        self.db.set_rankings(event_id, &ranked).await?;
        Ok(ranked)
    }

    pub async fn list_teams(&self, competition_id: CompetitionId) -> ServiceResult<Vec<Team>> {
        Ok(self.db.list_teams(competition_id).await?)
    }

    pub async fn list_participants(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<Vec<Participant>> {
        Ok(self.db.list_participants(competition_id).await?)
    }

    pub async fn list_judges(&self, competition_id: CompetitionId) -> ServiceResult<Vec<Judge>> {
        Ok(self.db.list_judges(competition_id).await?)
    }

    pub async fn get_judge(&self, id: JudgeId) -> ServiceResult<Judge> {
        Ok(self.db.get_judge(id).await?)
    }

    pub async fn list_events(&self, competition_id: CompetitionId) -> ServiceResult<Vec<Event>> {
        Ok(self.db.list_events(competition_id).await?)
    }

    pub async fn get_participant(&self, id: ParticipantId) -> ServiceResult<Participant> {
        Ok(self.db.get_participant(id).await?)
    }

    pub async fn get_event(&self, id: EventId) -> ServiceResult<Event> {
        Ok(self.db.get_event(id).await?)
    }

    // ---- Theming ----

    pub async fn resolve_theme(
        &self,
        competition_id: CompetitionId,
    ) -> ServiceResult<ScoreboardTheme> {
        let competition = self.db.get_competition(competition_id).await?;
        match competition.theme_id {
            Some(theme_id) => Ok(self.db.get_theme(theme_id).await?),
            None => Ok(ScoreboardTheme::old_school_grid()),
        }
    }

    pub async fn set_theme(
        &self,
        competition_id: CompetitionId,
        theme: ScoreboardTheme,
    ) -> ServiceResult<()> {
        if self.db.get_theme(theme.id).await.is_ok() {
            self.db.update_theme(&theme).await?;
        } else {
            self.db.create_theme(&theme).await?;
        }
        self.db
            .set_competition_theme(competition_id, Some(theme.id))
            .await?;
        Ok(())
    }

    // ---- Export / import ----

    pub async fn export_to(&self, path: impl AsRef<Path>) -> ServiceResult<()> {
        let bundle = self.db.export_all().await?;
        bundle.write_to_zip(path)?;
        Ok(())
    }

    pub async fn import_from(
        &self,
        path: impl AsRef<Path>,
        mode: db::export::ImportMode,
    ) -> ServiceResult<()> {
        let bundle = db::export::ExportBundle::read_from_zip(path)?;
        let asset_dir = db::export::default_asset_dir(&self.db_path);
        self.db.import_bundle(&bundle, &asset_dir, mode).await?;
        Ok(())
    }
}

fn generate_pairing_code() -> String {
    let n: u32 = rand::thread_rng().gen_range(0..1_000_000);
    format!("{n:06}")
}

fn generate_device_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
