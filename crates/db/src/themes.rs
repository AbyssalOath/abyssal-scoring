use models::{ScoreboardTheme, ThemeId, ThemePreset};
use sqlx::Row;

use crate::error::{DbError, DbResult};
use crate::pool::Db;
use crate::util::parse_uuid;

impl Db {
    pub async fn create_theme(&self, theme: &ScoreboardTheme) -> DbResult<()> {
        sqlx::query(
            "INSERT INTO themes (id, name, preset, background_color, primary_color, accent_color, background_image_path, logo_path, font_family, show_grid_overlay)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(theme.id.0.to_string())
        .bind(&theme.name)
        .bind(theme.preset.as_str())
        .bind(&theme.background_color)
        .bind(&theme.primary_color)
        .bind(&theme.accent_color)
        .bind(&theme.background_image_path)
        .bind(&theme.logo_path)
        .bind(&theme.font_family)
        .bind(theme.show_grid_overlay)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_theme(&self, id: ThemeId) -> DbResult<ScoreboardTheme> {
        let row = sqlx::query("SELECT * FROM themes WHERE id = ?")
            .bind(id.0.to_string())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DbError::NotFound)?;
        row_to_theme(&row)
    }

    pub async fn list_themes(&self) -> DbResult<Vec<ScoreboardTheme>> {
        let rows = sqlx::query("SELECT * FROM themes ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_theme).collect()
    }

    pub async fn update_theme(&self, theme: &ScoreboardTheme) -> DbResult<()> {
        sqlx::query(
            "UPDATE themes SET name = ?, preset = ?, background_color = ?, primary_color = ?, accent_color = ?, background_image_path = ?, logo_path = ?, font_family = ?, show_grid_overlay = ? WHERE id = ?",
        )
        .bind(&theme.name)
        .bind(theme.preset.as_str())
        .bind(&theme.background_color)
        .bind(&theme.primary_color)
        .bind(&theme.accent_color)
        .bind(&theme.background_image_path)
        .bind(&theme.logo_path)
        .bind(&theme.font_family)
        .bind(theme.show_grid_overlay)
        .bind(theme.id.0.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn row_to_theme(row: &sqlx::sqlite::SqliteRow) -> DbResult<ScoreboardTheme> {
    let id_str: String = row.try_get("id")?;
    let preset_str: String = row.try_get("preset")?;
    Ok(ScoreboardTheme {
        id: ThemeId(parse_uuid("themes.id", &id_str)?),
        name: row.try_get("name")?,
        preset: ThemePreset::from_db_str(&preset_str).ok_or_else(|| {
            DbError::InvalidStoredValue {
                field: "themes.preset",
                value: preset_str.clone(),
            }
        })?,
        background_color: row.try_get("background_color")?,
        primary_color: row.try_get("primary_color")?,
        accent_color: row.try_get("accent_color")?,
        background_image_path: row.try_get("background_image_path")?,
        logo_path: row.try_get("logo_path")?,
        font_family: row.try_get("font_family")?,
        show_grid_overlay: row.try_get("show_grid_overlay")?,
    })
}
