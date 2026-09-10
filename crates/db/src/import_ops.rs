//! `INSERT OR REPLACE` counterparts to the plain `create_*` functions,
//! used only by `import_bundle` so re-importing the same `.abyssal`
//! file twice (or importing into an instance that already has some of
//! this data) is safe instead of erroring on a primary-key conflict.

use models::gymnastics::Routine;
use models::{
    Competition, Event, EventResult, Judge, Participant, Score, ScoreboardTheme, Session, Team,
};

use crate::error::DbResult;
use crate::gymnastics::apparatus_str;
use crate::pool::Db;

impl Db {
    pub(crate) async fn import_theme_or_replace(&self, t: &ScoreboardTheme) -> DbResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO themes (id, name, preset, background_color, primary_color, accent_color, background_image_path, logo_path, font_family, show_grid_overlay)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(t.id.0.to_string())
        .bind(&t.name)
        .bind(t.preset.as_str())
        .bind(&t.background_color)
        .bind(&t.primary_color)
        .bind(&t.accent_color)
        .bind(&t.background_image_path)
        .bind(&t.logo_path)
        .bind(&t.font_family)
        .bind(t.show_grid_overlay)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(crate) async fn import_competition_or_replace(&self, c: &Competition) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO competitions (id, name, sport, date, location, theme_id) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(c.id.0.to_string())
            .bind(&c.name)
            .bind(c.sport.as_str())
            .bind(c.date.to_string())
            .bind(&c.location)
            .bind(c.theme_id.map(|t| t.0.to_string()))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_team_or_replace(&self, t: &Team) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO teams (id, competition_id, name, abbreviation, primary_color, secondary_color, logo_path) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(t.id.0.to_string())
            .bind(t.competition_id.0.to_string())
            .bind(&t.name)
            .bind(&t.abbreviation)
            .bind(&t.primary_color)
            .bind(&t.secondary_color)
            .bind(&t.logo_path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_participant_or_replace(&self, p: &Participant) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO participants (id, competition_id, first_name, last_name, team_id, bib_number, level_or_category) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(p.id.0.to_string())
            .bind(p.competition_id.0.to_string())
            .bind(&p.first_name)
            .bind(&p.last_name)
            .bind(p.team_id.map(|t| t.0.to_string()))
            .bind(&p.bib_number)
            .bind(&p.level_or_category)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_judge_or_replace(&self, j: &Judge) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO judges (id, competition_id, name, role, assignment) VALUES (?, ?, ?, ?, ?)")
            .bind(j.id.0.to_string())
            .bind(j.competition_id.0.to_string())
            .bind(&j.name)
            .bind(j.role.as_str())
            .bind(&j.assignment)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_session_or_replace(&self, s: &Session) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO sessions (id, competition_id, name, start_time, order_index) VALUES (?, ?, ?, ?, ?)")
            .bind(s.id.0.to_string())
            .bind(s.competition_id.0.to_string())
            .bind(&s.name)
            .bind(s.start_time.map(|t| t.to_rfc3339()))
            .bind(s.order_index)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_event_or_replace(&self, e: &Event) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO events (id, competition_id, session_id, name, order_index, status) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(e.id.0.to_string())
            .bind(e.competition_id.0.to_string())
            .bind(e.session_id.map(|s| s.0.to_string()))
            .bind(&e.name)
            .bind(e.order_index)
            .bind(e.status.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_result_or_replace(&self, r: &EventResult) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO results (id, event_id, participant_id, team_id, final_value, place, finalized, detail) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(r.id.0.to_string())
            .bind(r.event_id.0.to_string())
            .bind(r.participant_id.map(|p| p.0.to_string()))
            .bind(r.team_id.map(|t| t.0.to_string()))
            .bind(r.final_value)
            .bind(r.place)
            .bind(r.finalized)
            .bind(r.detail.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_score_or_replace(&self, s: &Score) -> DbResult<()> {
        sqlx::query("INSERT OR REPLACE INTO scores (id, result_id, judge_id, value, components, submitted_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(s.id.0.to_string())
            .bind(s.result_id.0.to_string())
            .bind(s.judge_id.map(|j| j.0.to_string()))
            .bind(s.value)
            .bind(s.components.to_string())
            .bind(s.submitted_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(crate) async fn import_gymnastics_routine_or_replace(&self, r: &Routine) -> DbResult<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO gymnastics_routines (result_id, apparatus, difficulty_score, execution_start_value, deductions, neutral_deductions)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(r.result_id.0.to_string())
        .bind(apparatus_str(r.apparatus))
        .bind(r.difficulty_score)
        .bind(r.execution_start_value)
        .bind(serde_json::to_string(&r.deductions)?)
        .bind(r.neutral_deductions)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
