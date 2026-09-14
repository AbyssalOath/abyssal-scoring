//! CRUD for `gymnastics_details` -- kept separate from `gymnastics.rs`,
//! which only handles `gymnastics_routines` (a per-result table; this
//! one is per-participant).

use models::gymnastics::GymnastDetail;
use models::{CompetitionId, ParticipantId};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_gymnast_detail(&self, g: &GymnastDetail) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO gymnastics_details (participant_id, competition_id, level, club, taaf_athlete_number, taaf_registration_number, qual_status, at_code)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(g.participant_id.0.to_string())
        .bind(g.competition_id.0.to_string())
        .bind(&g.level)
        .bind(&g.club)
        .bind(&g.taaf_athlete_number)
        .bind(&g.taaf_registration_number)
        .bind(&g.qual_status)
        .bind(&g.at_code)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_gymnast_detail(&self, g: &GymnastDetail) -> DbResult<()> {
        sqlx::query(
            "UPDATE gymnastics_details SET level = ?, club = ?, taaf_athlete_number = ?, taaf_registration_number = ?, qual_status = ?, at_code = ? WHERE participant_id = ?",
        )
        .bind(&g.level)
        .bind(&g.club)
        .bind(&g.taaf_athlete_number)
        .bind(&g.taaf_registration_number)
        .bind(&g.qual_status)
        .bind(&g.at_code)
        .bind(g.participant_id.0.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_gymnast_detail(
        &self,
        participant_id: ParticipantId,
    ) -> DbResult<GymnastDetail> {
        let row = sqlx::query("SELECT * FROM gymnastics_details WHERE participant_id = ?")
            .bind(participant_id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_gymnast_detail(&row)
    }

    /// The upsert key for weekly roster re-imports: find the existing
    /// participant this TAAF athlete number belongs to, if any.
    pub async fn find_gymnast_detail_by_taaf_number(
        &self,
        competition_id: CompetitionId,
        taaf_athlete_number: &str,
    ) -> DbResult<Option<GymnastDetail>> {
        let row = sqlx::query(
            "SELECT * FROM gymnastics_details WHERE competition_id = ? AND taaf_athlete_number = ?",
        )
        .bind(competition_id.0.to_string())
        .bind(taaf_athlete_number)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(row_to_gymnast_detail).transpose()
    }

    pub async fn list_gymnast_details_for_competition(
        &self,
        competition_id: CompetitionId,
    ) -> DbResult<Vec<GymnastDetail>> {
        let rows = sqlx::query("SELECT * FROM gymnastics_details WHERE competition_id = ?")
            .bind(competition_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_gymnast_detail).collect()
    }
}

fn row_to_gymnast_detail(row: &sqlx::sqlite::SqliteRow) -> DbResult<GymnastDetail> {
    let participant_str: String = row.try_get("participant_id")?;
    let competition_str: String = row.try_get("competition_id")?;
    Ok(GymnastDetail {
        participant_id: ParticipantId(parse_uuid(
            "gymnastics_details.participant_id",
            &participant_str,
        )?),
        competition_id: CompetitionId(parse_uuid(
            "gymnastics_details.competition_id",
            &competition_str,
        )?),
        level: row.try_get("level")?,
        club: row.try_get("club")?,
        taaf_athlete_number: row.try_get("taaf_athlete_number")?,
        taaf_registration_number: row.try_get("taaf_registration_number")?,
        qual_status: row.try_get("qual_status")?,
        at_code: row.try_get("at_code")?,
    })
}
