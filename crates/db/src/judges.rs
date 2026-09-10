use models::{CompetitionId, Judge, JudgeId, JudgeRole};
use sqlx::Row;

use crate::error::DbResult;
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_judge(&self, j: &Judge) -> DbResult<()> {
        sqlx::query("INSERT INTO judges (id, competition_id, name, role, assignment) VALUES (?, ?, ?, ?, ?)")
            .bind(j.id.0.to_string())
            .bind(j.competition_id.0.to_string())
            .bind(&j.name)
            .bind(j.role.as_str())
            .bind(&j.assignment)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_judges(&self, competition_id: CompetitionId) -> DbResult<Vec<Judge>> {
        let rows = sqlx::query("SELECT * FROM judges WHERE competition_id = ? ORDER BY name")
            .bind(competition_id.0.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_judge).collect()
    }
}

fn row_to_judge(row: &sqlx::sqlite::SqliteRow) -> DbResult<Judge> {
    let id_str: String = row.try_get("id")?;
    let comp_str: String = row.try_get("competition_id")?;
    let role_str: String = row.try_get("role")?;
    Ok(Judge {
        id: JudgeId(parse_uuid("judges.id", &id_str)?),
        competition_id: CompetitionId(parse_uuid("judges.competition_id", &comp_str)?),
        name: row.try_get("name")?,
        role: JudgeRole::from_stored(&role_str),
        assignment: row.try_get("assignment")?,
    })
}
