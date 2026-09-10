use models::{CompetitionId, Event, EventId, EventStatus, SessionId};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_event(&self, e: &Event) -> DbResult<()> {
        sqlx::query("INSERT INTO events (id, competition_id, session_id, name, order_index, status) VALUES (?, ?, ?, ?, ?, ?)")
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

    pub async fn get_event(&self, id: EventId) -> DbResult<Event> {
        let row = sqlx::query("SELECT * FROM events WHERE id = ?")
            .bind(id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_event(&row)
    }

    pub async fn list_events(&self, competition_id: CompetitionId) -> DbResult<Vec<Event>> {
        let rows =
            sqlx::query("SELECT * FROM events WHERE competition_id = ? ORDER BY order_index")
                .bind(competition_id.0.to_string())
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(row_to_event).collect()
    }

    pub async fn set_event_status(&self, id: EventId, status: EventStatus) -> DbResult<()> {
        sqlx::query("UPDATE events SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn row_to_event(row: &sqlx::sqlite::SqliteRow) -> DbResult<Event> {
    let id_str: String = row.try_get("id")?;
    let comp_str: String = row.try_get("competition_id")?;
    let session_str: Option<String> = row.try_get("session_id")?;
    let status_str: String = row.try_get("status")?;
    Ok(Event {
        id: EventId(parse_uuid("events.id", &id_str)?),
        competition_id: CompetitionId(parse_uuid("events.competition_id", &comp_str)?),
        session_id: session_str
            .map(|s| parse_uuid("events.session_id", &s))
            .transpose()?
            .map(SessionId),
        name: row.try_get("name")?,
        order_index: row.try_get("order_index")?,
        status: EventStatus::from_db_str(&status_str).ok_or_else(|| {
            DbError::InvalidStoredValue {
                field: "events.status",
                value: status_str.clone(),
            }
        })?,
    })
}
