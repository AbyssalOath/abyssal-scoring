use chrono::{DateTime, Utc};
use models::{CompetitionId, DisplayMode, EventId, ParticipantId, ScoreboardState};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::{parse_json, parse_uuid};

impl Db {
    /// Fetches the live state, creating an idle row the first time a
    /// competition is asked about (so callers never have to special-
    /// case "never set yet" vs. "explicitly cleared").
    pub async fn get_or_init_scoreboard_state(
        &self,
        competition_id: CompetitionId,
    ) -> DbResult<ScoreboardState> {
        let row = sqlx::query("SELECT * FROM scoreboard_state WHERE competition_id = ?")
            .bind(competition_id.0.to_string())
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(row) => row_to_state(&row),
            None => {
                let state = ScoreboardState::idle(competition_id);
                self.upsert_scoreboard_state(&state).await?;
                Ok(state)
            }
        }
    }

    pub async fn upsert_scoreboard_state(&self, state: &ScoreboardState) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO scoreboard_state (competition_id, current_event_id, current_participant_id, display_mode, live_value, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(competition_id) DO UPDATE SET
                current_event_id = excluded.current_event_id,
                current_participant_id = excluded.current_participant_id,
                display_mode = excluded.display_mode,
                live_value = excluded.live_value,
                updated_at = excluded.updated_at",
        )
        .bind(state.competition_id.0.to_string())
        .bind(state.current_event_id.map(|e| e.0.to_string()))
        .bind(state.current_participant_id.map(|p| p.0.to_string()))
        .bind(state.display_mode.as_str())
        .bind(state.live_value.to_string())
        .bind(state.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// "Clear scoreboard for the next event." Only touches
    /// `scoreboard_state` -- everything in `events`, `participants`,
    /// `judges`, `results`, `scores` is left exactly as it was.
    pub async fn clear_scoreboard(
        &self,
        competition_id: CompetitionId,
    ) -> DbResult<ScoreboardState> {
        let mut state = self.get_or_init_scoreboard_state(competition_id).await?;
        state.clear();
        self.upsert_scoreboard_state(&state).await?;
        Ok(state)
    }
}

fn row_to_state(row: &sqlx::sqlite::SqliteRow) -> DbResult<ScoreboardState> {
    let comp_str: String = row.try_get("competition_id")?;
    let event_str: Option<String> = row.try_get("current_event_id")?;
    let participant_str: Option<String> = row.try_get("current_participant_id")?;
    let mode_str: String = row.try_get("display_mode")?;
    let live_str: String = row.try_get("live_value")?;
    let updated_str: String = row.try_get("updated_at")?;
    Ok(ScoreboardState {
        competition_id: CompetitionId(parse_uuid("scoreboard_state.competition_id", &comp_str)?),
        current_event_id: event_str
            .map(|s| parse_uuid("scoreboard_state.current_event_id", &s))
            .transpose()?
            .map(EventId),
        current_participant_id: participant_str
            .map(|s| parse_uuid("scoreboard_state.current_participant_id", &s))
            .transpose()?
            .map(ParticipantId),
        display_mode: DisplayMode::from_db_str(&mode_str).ok_or_else(|| {
            DbError::InvalidStoredValue {
                field: "scoreboard_state.display_mode",
                value: mode_str.clone(),
            }
        })?,
        live_value: parse_json("scoreboard_state.live_value", &live_str)?,
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
            .map_err(|_| DbError::InvalidStoredValue {
                field: "scoreboard_state.updated_at",
                value: updated_str.clone(),
            })?
            .with_timezone(&Utc),
    })
}
