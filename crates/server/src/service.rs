use std::path::{Path, PathBuf};

use chrono::Utc;
use db::Db;
use models::gymnastics::{Apparatus, Deduction, Routine};
use models::{
    Competition, CompetitionId, DisplayMode, Event, EventId, EventResult, Judge, JudgeId,
    JudgeRole, Participant, ParticipantId, RankingEntry, Score, ScoreboardState, ScoreboardTheme,
    Session, Sport, Team, TeamId,
};
use serde::{Deserialize, Serialize};

use crate::error::ServiceResult;
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

impl AppService {
    pub async fn open(db_path: impl AsRef<Path>) -> ServiceResult<Self> {
        let db = Db::open(&db_path).await?;
        Ok(Self {
            db,
            db_path: db_path.as_ref().to_path_buf(),
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
