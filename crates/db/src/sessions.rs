use chrono::{DateTime, Utc};
use models::{CompetitionId, Session, SessionId};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_session(&self, s: &Session) -> DbResult<()> {
        sqlx::query("INSERT INTO sessions (id, competition_id, name, start_time, order_index) VALUES (?, ?, ?, ?, ?)")
            .bind(s.id.0.to_string())
            .bind(s.competition_id.0.to_string())
            .bind(&s.name)
            .bind(s.start_time.map(|t| t.to_rfc3339()))
            .bind(s.order_index)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_sessions(&self, competition_id: CompetitionId) -> DbResult<Vec<Session>> {
        let rows =
            sqlx::query("SELECT * FROM sessions WHERE competition_id = ? ORDER BY order_index")
                .bind(competition_id.0.to_string())
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(row_to_session).collect()
    }
}

fn row_to_session(row: &sqlx::sqlite::SqliteRow) -> DbResult<Session> {
    let id_str: String = row.try_get("id")?;
    let comp_str: String = row.try_get("competition_id")?;
    let start_str: Option<String> = row.try_get("start_time")?;
    Ok(Session {
        id: SessionId(parse_uuid("sessions.id", &id_str)?),
        competition_id: CompetitionId(parse_uuid("sessions.competition_id", &comp_str)?),
        name: row.try_get("name")?,
        start_time: start_str
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Utc)))
            .transpose()
            .map_err(|_| DbError::InvalidStoredValue {
                field: "sessions.start_time",
                value: "".to_string(),
            })?,
        order_index: row.try_get("order_index")?,
    })
}
