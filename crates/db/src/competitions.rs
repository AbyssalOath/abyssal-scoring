use chrono::NaiveDate;
use models::{Competition, CompetitionId, Sport, ThemeId};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_competition(&self, c: &Competition) -> DbResult<()> {
        sqlx::query("INSERT INTO competitions (id, name, sport, date, location, theme_id) VALUES (?, ?, ?, ?, ?, ?)")
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

    pub async fn get_competition(&self, id: CompetitionId) -> DbResult<Competition> {
        let row = sqlx::query("SELECT * FROM competitions WHERE id = ?")
            .bind(id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_competition(&row)
    }

    pub async fn list_competitions(&self) -> DbResult<Vec<Competition>> {
        let rows = sqlx::query("SELECT * FROM competitions ORDER BY date DESC")
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_competition).collect()
    }

    pub async fn set_competition_theme(
        &self,
        id: CompetitionId,
        theme_id: Option<ThemeId>,
    ) -> DbResult<()> {
        sqlx::query("UPDATE competitions SET theme_id = ? WHERE id = ?")
            .bind(theme_id.map(|t| t.0.to_string()))
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_competition(&self, id: CompetitionId) -> DbResult<()> {
        sqlx::query("DELETE FROM competitions WHERE id = ?")
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn row_to_competition(row: &sqlx::sqlite::SqliteRow) -> DbResult<Competition> {
    let id_str: String = row.try_get("id")?;
    let sport_str: String = row.try_get("sport")?;
    let date_str: String = row.try_get("date")?;
    let theme_id_str: Option<String> = row.try_get("theme_id")?;
    Ok(Competition {
        id: CompetitionId(parse_uuid("competitions.id", &id_str)?),
        name: row.try_get("name")?,
        sport: Sport::from_db_str(&sport_str).ok_or_else(|| DbError::InvalidStoredValue {
            field: "competitions.sport",
            value: sport_str.clone(),
        })?,
        date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
            DbError::InvalidStoredValue {
                field: "competitions.date",
                value: date_str.clone(),
            }
        })?,
        location: row.try_get("location")?,
        theme_id: theme_id_str
            .map(|s| parse_uuid("competitions.theme_id", &s))
            .transpose()?
            .map(ThemeId),
    })
}
