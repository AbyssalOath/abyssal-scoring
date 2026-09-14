use chrono::{DateTime, Utc};
use models::{CompetitionId, JudgeDevice, JudgeDeviceId, JudgeId, PairingCode};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_pairing_code(&self, p: &PairingCode) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO pairing_codes (code, judge_id, expires_at, consumed) VALUES (?, ?, ?, ?)",
        )
        .bind(&p.code)
        .bind(p.judge_id.0.to_string())
        .bind(p.expires_at.to_rfc3339())
        .bind(p.consumed)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_pairing_code(&self, code: &str) -> DbResult<PairingCode> {
        let row = sqlx::query("SELECT * FROM pairing_codes WHERE code = ?")
            .bind(code)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_pairing_code(&row)
    }

    pub async fn consume_pairing_code(&self, code: &str) -> DbResult<()> {
        sqlx::query("UPDATE pairing_codes SET consumed = 1 WHERE code = ?")
            .bind(code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_judge_device(&self, d: &JudgeDevice) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO judge_devices (id, judge_id, device_name, token_hash, paired_at, last_seen_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(d.id.0.to_string())
        .bind(d.judge_id.0.to_string())
        .bind(&d.device_name)
        .bind(&d.token_hash)
        .bind(d.paired_at.to_rfc3339())
        .bind(d.last_seen_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Authentication lookup: hash the presented bearer token and find
    /// the device it belongs to, if any.
    pub async fn find_judge_device_by_token_hash(
        &self,
        token_hash: &str,
    ) -> DbResult<Option<JudgeDevice>> {
        let row = sqlx::query("SELECT * FROM judge_devices WHERE token_hash = ?")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_judge_device).transpose()
    }

    pub async fn touch_judge_device(
        &self,
        id: JudgeDeviceId,
        seen_at: DateTime<Utc>,
    ) -> DbResult<()> {
        sqlx::query("UPDATE judge_devices SET last_seen_at = ? WHERE id = ?")
            .bind(seen_at.to_rfc3339())
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_judge_devices_for_competition(
        &self,
        competition_id: CompetitionId,
    ) -> DbResult<Vec<JudgeDevice>> {
        let rows = sqlx::query(
            "SELECT judge_devices.* FROM judge_devices
             JOIN judges ON judges.id = judge_devices.judge_id
             WHERE judges.competition_id = ?
             ORDER BY judge_devices.paired_at",
        )
        .bind(competition_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_judge_device).collect()
    }

    pub async fn revoke_judge_device(&self, id: JudgeDeviceId) -> DbResult<()> {
        sqlx::query("DELETE FROM judge_devices WHERE id = ?")
            .bind(id.0.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn row_to_pairing_code(row: &sqlx::sqlite::SqliteRow) -> DbResult<PairingCode> {
    let judge_str: String = row.try_get("judge_id")?;
    let expires_str: String = row.try_get("expires_at")?;
    Ok(PairingCode {
        code: row.try_get("code")?,
        judge_id: JudgeId(parse_uuid("pairing_codes.judge_id", &judge_str)?),
        expires_at: DateTime::parse_from_rfc3339(&expires_str)
            .map_err(|_| DbError::InvalidStoredValue {
                field: "pairing_codes.expires_at",
                value: expires_str.clone(),
            })?
            .with_timezone(&Utc),
        consumed: row.try_get("consumed")?,
    })
}

fn row_to_judge_device(row: &sqlx::sqlite::SqliteRow) -> DbResult<JudgeDevice> {
    let id_str: String = row.try_get("id")?;
    let judge_str: String = row.try_get("judge_id")?;
    let paired_str: String = row.try_get("paired_at")?;
    let last_seen_str: Option<String> = row.try_get("last_seen_at")?;
    Ok(JudgeDevice {
        id: JudgeDeviceId(parse_uuid("judge_devices.id", &id_str)?),
        judge_id: JudgeId(parse_uuid("judge_devices.judge_id", &judge_str)?),
        device_name: row.try_get("device_name")?,
        token_hash: row.try_get("token_hash")?,
        paired_at: DateTime::parse_from_rfc3339(&paired_str)
            .map_err(|_| DbError::InvalidStoredValue {
                field: "judge_devices.paired_at",
                value: paired_str.clone(),
            })?
            .with_timezone(&Utc),
        last_seen_at: last_seen_str
            .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .map_err(|_| DbError::InvalidStoredValue {
                field: "judge_devices.last_seen_at",
                value: "invalid timestamp".to_string(),
            })?,
    })
}
