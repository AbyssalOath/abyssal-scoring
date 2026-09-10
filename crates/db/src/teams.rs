use models::{CompetitionId, Team, TeamId};
use sqlx::Row;

use crate::error::DbResult;
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_team(&self, t: &Team) -> DbResult<()> {
        sqlx::query("INSERT INTO teams (id, competition_id, name, abbreviation, primary_color, secondary_color, logo_path) VALUES (?, ?, ?, ?, ?, ?, ?)")
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

    pub async fn list_teams(&self, competition_id: CompetitionId) -> DbResult<Vec<Team>> {
        let rows = sqlx::query("SELECT * FROM teams WHERE competition_id = ? ORDER BY name")
            .bind(competition_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_team).collect()
    }

    pub async fn update_team_branding(
        &self,
        id: TeamId,
        primary_color: Option<String>,
        secondary_color: Option<String>,
        logo_path: Option<String>,
    ) -> DbResult<()> {
        sqlx::query(
            "UPDATE teams SET primary_color = ?, secondary_color = ?, logo_path = ? WHERE id = ?",
        )
        .bind(primary_color)
        .bind(secondary_color)
        .bind(logo_path)
        .bind(id.0.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_team(row: &sqlx::sqlite::SqliteRow) -> DbResult<Team> {
    let id_str: String = row.try_get("id")?;
    let comp_str: String = row.try_get("competition_id")?;
    Ok(Team {
        id: TeamId(parse_uuid("teams.id", &id_str)?),
        competition_id: CompetitionId(parse_uuid("teams.competition_id", &comp_str)?),
        name: row.try_get("name")?,
        abbreviation: row.try_get("abbreviation")?,
        primary_color: row.try_get("primary_color")?,
        secondary_color: row.try_get("secondary_color")?,
        logo_path: row.try_get("logo_path")?,
    })
}
