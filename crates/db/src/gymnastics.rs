use models::gymnastics::{Apparatus, Deduction, Routine};
use models::ResultId;
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_gymnastics_routine(&self, r: &Routine) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO gymnastics_routines (result_id, apparatus, difficulty_score, execution_start_value, deductions, neutral_deductions)
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

    pub async fn get_gymnastics_routine(&self, result_id: ResultId) -> DbResult<Routine> {
        let row = sqlx::query("SELECT * FROM gymnastics_routines WHERE result_id = ?")
            .bind(result_id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        let result_id_str: String = row.try_get("result_id")?;
        let apparatus_s: String = row.try_get("apparatus")?;
        let deductions_s: String = row.try_get("deductions")?;
        Ok(Routine {
            result_id: ResultId(parse_uuid("gymnastics_routines.result_id", &result_id_str)?),
            apparatus: apparatus_from_str(&apparatus_s)?,
            difficulty_score: row.try_get("difficulty_score")?,
            execution_start_value: row.try_get("execution_start_value")?,
            deductions: serde_json::from_str::<Vec<Deduction>>(&deductions_s)?,
            neutral_deductions: row.try_get("neutral_deductions")?,
        })
    }
}

pub(crate) fn apparatus_str(a: Apparatus) -> &'static str {
    match a {
        Apparatus::Vault => "vault",
        Apparatus::UnevenBars => "uneven_bars",
        Apparatus::BalanceBeam => "balance_beam",
        Apparatus::Floor => "floor",
        Apparatus::PommelHorse => "pommel_horse",
        Apparatus::StillRings => "still_rings",
        Apparatus::ParallelBars => "parallel_bars",
        Apparatus::HorizontalBar => "horizontal_bar",
    }
}

fn apparatus_from_str(s: &str) -> DbResult<Apparatus> {
    Ok(match s {
        "vault" => Apparatus::Vault,
        "uneven_bars" => Apparatus::UnevenBars,
        "balance_beam" => Apparatus::BalanceBeam,
        "floor" => Apparatus::Floor,
        "pommel_horse" => Apparatus::PommelHorse,
        "still_rings" => Apparatus::StillRings,
        "parallel_bars" => Apparatus::ParallelBars,
        "horizontal_bar" => Apparatus::HorizontalBar,
        other => {
            return Err(DbError::InvalidStoredValue {
                field: "gymnastics_routines.apparatus",
                value: other.to_string(),
            })
        }
    })
}
