use serde::{Deserialize, Serialize};

use crate::ids::ThemeId;

/// Named starting points for a theme. `Custom` means every field on
/// `ScoreboardTheme` is user-chosen; the others are just convenient
/// defaults the UI can offer and then let the user tweak further.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    /// Black grid background, red segmented digits -- classic gym
    /// scoreboard look.
    OldSchoolGrid,
    Modern,
    Minimal,
    Custom,
}

/// Drives the look of the scoreboard display window. Applied as CSS
/// custom properties client-side, so re-skinning never touches layout
/// markup. Can be attached to a `Competition` (event-wide) or to a
/// `Team` via its own colors/logo for a split-screen look.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreboardTheme {
    pub id: ThemeId,
    pub name: String,
    pub preset: ThemePreset,
    /// Hex colors, e.g. "#0a0a0a".
    pub background_color: String,
    pub primary_color: String,
    pub accent_color: String,
    /// Absolute or app-data-relative path to a background image.
    pub background_image_path: Option<String>,
    /// Team/competition logo, drawn over or beside the background.
    pub logo_path: Option<String>,
    /// CSS font-family value. The old-school preset ships a
    /// seven-segment-style webfont; anything else falls back to a
    /// bundled sans-serif.
    pub font_family: String,
    /// Draws the faint grid-line overlay associated with physical
    /// LED scoreboards. Independent of preset so it can be toggled
    /// even on a "Custom" theme.
    pub show_grid_overlay: bool,
}

impl ScoreboardTheme {
    pub fn old_school_grid() -> Self {
        Self {
            id: ThemeId::new(),
            name: "Old School Grid".to_string(),
            preset: ThemePreset::OldSchoolGrid,
            background_color: "#0a0a0a".to_string(),
            primary_color: "#ff1a1a".to_string(),
            accent_color: "#ffb000".to_string(),
            background_image_path: None,
            logo_path: None,
            font_family: "\"Seven Segment\", monospace".to_string(),
            show_grid_overlay: true,
        }
    }

    pub fn modern() -> Self {
        Self {
            id: ThemeId::new(),
            name: "Modern".to_string(),
            preset: ThemePreset::Modern,
            background_color: "#101820".to_string(),
            primary_color: "#ffffff".to_string(),
            accent_color: "#3ea6ff".to_string(),
            background_image_path: None,
            logo_path: None,
            font_family: "\"Inter\", sans-serif".to_string(),
            show_grid_overlay: false,
        }
    }

    /// Builds a theme from a team's brand colors, keeping everything
    /// else about the current theme (grid style, font) the same --
    /// e.g. for a split-screen dual-team scoreboard.
    pub fn from_team_colors(
        base: &ScoreboardTheme,
        primary: &str,
        accent: &str,
        logo_path: Option<String>,
    ) -> Self {
        Self {
            id: ThemeId::new(),
            name: format!("{} (team colors)", base.name),
            preset: ThemePreset::Custom,
            background_color: base.background_color.clone(),
            primary_color: primary.to_string(),
            accent_color: accent.to_string(),
            background_image_path: base.background_image_path.clone(),
            logo_path,
            font_family: base.font_family.clone(),
            show_grid_overlay: base.show_grid_overlay,
        }
    }
}

impl Default for ScoreboardTheme {
    fn default() -> Self {
        Self::old_school_grid()
    }
}

impl ThemePreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemePreset::OldSchoolGrid => "old_school_grid",
            ThemePreset::Modern => "modern",
            ThemePreset::Minimal => "minimal",
            ThemePreset::Custom => "custom",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "old_school_grid" => Some(ThemePreset::OldSchoolGrid),
            "modern" => Some(ThemePreset::Modern),
            "minimal" => Some(ThemePreset::Minimal),
            "custom" => Some(ThemePreset::Custom),
            _ => None,
        }
    }
}
