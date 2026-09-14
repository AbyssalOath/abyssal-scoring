use models::{CompetitionId, Participant, ParticipantId, TeamId};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_participant(&self, p: &Participant) -> DbResult<()> {
        sqlx::query("INSERT INTO participants (id, competition_id, first_name, last_name, team_id, bib_number, level_or_category, is_attending) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(p.id.0.to_string())
            .bind(p.competition_id.0.to_string())
            .bind(&p.first_name)
            .bind(&p.last_name)
            .bind(p.team_id.map(|t| t.0.to_string()))
            .bind(&p.bib_number)
            .bind(&p.level_or_category)
            .bind(p.is_attending)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_participant(&self, p: &Participant) -> DbResult<()> {
        sqlx::query(
            "UPDATE participants SET first_name = ?, last_name = ?, team_id = ?, bib_number = ?, level_or_category = ?, is_attending = ? WHERE id = ?",
        )
        .bind(&p.first_name)
        .bind(&p.last_name)
        .bind(p.team_id.map(|t| t.0.to_string()))
        .bind(&p.bib_number)
        .bind(&p.level_or_category)
        .bind(p.is_attending)
        .bind(p.id.0.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_participant(&self, id: ParticipantId) -> DbResult<Participant> {
        let row = sqlx::query("SELECT * FROM participants WHERE id = ?")
            .bind(id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_participant(&row)
    }

    pub async fn list_participants(
        &self,
        competition_id: CompetitionId,
    ) -> DbResult<Vec<Participant>> {
        let rows = sqlx::query(
            "SELECT * FROM participants WHERE competition_id = ? ORDER BY last_name, first_name",
        )
        .bind(competition_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_participant).collect()
    }
}

fn row_to_participant(row: &sqlx::sqlite::SqliteRow) -> DbResult<Participant> {
    let id_str: String = row.try_get("id")?;
    let comp_str: String = row.try_get("competition_id")?;
    let team_str: Option<String> = row.try_get("team_id")?;
    Ok(Participant {
        id: ParticipantId(parse_uuid("participants.id", &id_str)?),
        competition_id: CompetitionId(parse_uuid("participants.competition_id", &comp_str)?),
        first_name: row.try_get("first_name")?,
        last_name: row.try_get("last_name")?,
        team_id: team_str
            .map(|s| parse_uuid("participants.team_id", &s))
            .transpose()?
            .map(TeamId),
        bib_number: row.try_get("bib_number")?,
        level_or_category: row.try_get("level_or_category")?,
        is_attending: row.try_get("is_attending")?,
    })
}
