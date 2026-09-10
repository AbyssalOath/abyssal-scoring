use chrono::{DateTime, Utc};
use models::{
    EventId, EventResult, JudgeId, ParticipantId, RankingEntry, ResultId, Score, ScoreId, TeamId,
};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::{parse_json, parse_uuid};

impl Db {
    pub async fn create_result(&self, r: &EventResult) -> DbResult<()> {
        sqlx::query("INSERT INTO results (id, event_id, participant_id, team_id, final_value, place, finalized, detail) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
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

    pub async fn finalize_result(
        &self,
        id: ResultId,
        final_value: f64,
        detail: serde_json::Value,
    ) -> DbResult<()> {
        sqlx::query("UPDATE results SET final_value = ?, detail = ?, finalized = 1 WHERE id = ?")
            .bind(final_value)
            .bind(detail.to_string())
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_results_for_event(&self, event_id: EventId) -> DbResult<Vec<EventResult>> {
        let rows = sqlx::query("SELECT * FROM results WHERE event_id = ?")
            .bind(event_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_result).collect()
    }

    pub async fn add_score(&self, s: &Score) -> DbResult<()> {
        sqlx::query("INSERT INTO scores (id, result_id, judge_id, value, components, submitted_at) VALUES (?, ?, ?, ?, ?, ?)")
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

    pub async fn list_scores_for_result(&self, result_id: ResultId) -> DbResult<Vec<Score>> {
        let rows = sqlx::query("SELECT * FROM scores WHERE result_id = ?")
            .bind(result_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_score).collect()
    }

    /// Replaces the standings for one event. Called after a result is
    /// finalized; simple/cheap because rankings are just derived data.
    pub async fn set_rankings(&self, event_id: EventId, entries: &[RankingEntry]) -> DbResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM rankings WHERE event_id = ?")
            .bind(event_id.0.to_string())
            .execute(&mut *tx)
            .await?;
        for entry in entries {
            sqlx::query("INSERT INTO rankings (id, event_id, participant_id, team_id, rank, score) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(entry.id.0.to_string())
                .bind(entry.event_id.map(|e| e.0.to_string()))
                .bind(entry.participant_id.map(|p| p.0.to_string()))
                .bind(entry.team_id.map(|t| t.0.to_string()))
                .bind(entry.rank)
                .bind(entry.score)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_rankings_for_event(&self, event_id: EventId) -> DbResult<Vec<RankingEntry>> {
        let rows = sqlx::query("SELECT * FROM rankings WHERE event_id = ? ORDER BY rank")
            .bind(event_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_ranking).collect()
    }
}

fn row_to_result(row: &sqlx::sqlite::SqliteRow) -> DbResult<EventResult> {
    let id_str: String = row.try_get("id")?;
    let event_str: String = row.try_get("event_id")?;
    let participant_str: Option<String> = row.try_get("participant_id")?;
    let team_str: Option<String> = row.try_get("team_id")?;
    let detail_str: String = row.try_get("detail")?;
    Ok(EventResult {
        id: ResultId(parse_uuid("results.id", &id_str)?),
        event_id: EventId(parse_uuid("results.event_id", &event_str)?),
        participant_id: participant_str
            .map(|s| parse_uuid("results.participant_id", &s))
            .transpose()?
            .map(ParticipantId),
        team_id: team_str
            .map(|s| parse_uuid("results.team_id", &s))
            .transpose()?
            .map(TeamId),
        final_value: row.try_get("final_value")?,
        place: row.try_get("place")?,
        finalized: row.try_get("finalized")?,
        detail: parse_json("results.detail", &detail_str)?,
    })
}

fn row_to_score(row: &sqlx::sqlite::SqliteRow) -> DbResult<Score> {
    let id_str: String = row.try_get("id")?;
    let result_str: String = row.try_get("result_id")?;
    let judge_str: Option<String> = row.try_get("judge_id")?;
    let components_str: String = row.try_get("components")?;
    let submitted_str: String = row.try_get("submitted_at")?;
    Ok(Score {
        id: ScoreId(parse_uuid("scores.id", &id_str)?),
        result_id: ResultId(parse_uuid("scores.result_id", &result_str)?),
        judge_id: judge_str
            .map(|s| parse_uuid("scores.judge_id", &s))
            .transpose()?
            .map(JudgeId),
        value: row.try_get("value")?,
        components: parse_json("scores.components", &components_str)?,
        submitted_at: DateTime::parse_from_rfc3339(&submitted_str)
            .map_err(|_| DbError::InvalidStoredValue {
                field: "scores.submitted_at",
                value: submitted_str.clone(),
            })?
            .with_timezone(&Utc),
    })
}

fn row_to_ranking(row: &sqlx::sqlite::SqliteRow) -> DbResult<RankingEntry> {
    let id_str: String = row.try_get("id")?;
    let event_str: Option<String> = row.try_get("event_id")?;
    let participant_str: Option<String> = row.try_get("participant_id")?;
    let team_str: Option<String> = row.try_get("team_id")?;
    Ok(RankingEntry {
        id: models::RankingId(parse_uuid("rankings.id", &id_str)?),
        event_id: event_str
            .map(|s| parse_uuid("rankings.event_id", &s))
            .transpose()?
            .map(EventId),
        participant_id: participant_str
            .map(|s| parse_uuid("rankings.participant_id", &s))
            .transpose()?
            .map(ParticipantId),
        team_id: team_str
            .map(|s| parse_uuid("rankings.team_id", &s))
            .transpose()?
            .map(TeamId),
        rank: row.try_get("rank")?,
        score: row.try_get("score")?,
    })
}
