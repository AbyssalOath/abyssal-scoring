//! `.abyssal` export/import: a portable, versioned snapshot of one
//! instance's data (competitions down through scores/rankings, plus
//! themes and any referenced logo/background images), independent of
//! SQLite, a particular schema version, or a particular server. Point
//! is: Instance A exports -> `.abyssal` file -> Instance B imports,
//! and neither has to be running the same database engine.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use models::gymnastics::{GymnastDetail, Routine};
use models::{
    Competition, CompetitionId, Event, EventId, EventResult, Judge, JudgeId, Participant,
    ParticipantId, RankingEntry, RankingId, ResultId, Score, ScoreId, ScoreboardTheme, Session,
    SessionId, Team, TeamId, ThemeId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DbResult;
use crate::pool::Db;

/// Bump this whenever the shape of `ExportBundle` changes in a way
/// that isn't backward compatible; `import` can then branch on it.
pub const EXPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub schema_version: u32,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportBundle {
    pub themes: Vec<ScoreboardTheme>,
    pub competitions: Vec<Competition>,
    pub teams: Vec<Team>,
    pub participants: Vec<Participant>,
    pub judges: Vec<Judge>,
    pub sessions: Vec<Session>,
    pub events: Vec<Event>,
    pub results: Vec<EventResult>,
    pub scores: Vec<Score>,
    pub rankings: Vec<RankingEntry>,
    pub gymnastics_routines: Vec<Routine>,
    #[serde(default)]
    pub gymnastics_details: Vec<GymnastDetail>,
    /// original absolute/app-relative path -> bytes, for every logo /
    /// background image referenced by a theme or team.
    #[serde(skip)]
    pub assets: Vec<(String, Vec<u8>)>,
}

impl Db {
    /// Dumps every competition in this instance into one bundle.
    pub async fn export_all(&self) -> DbResult<ExportBundle> {
        let mut bundle = ExportBundle {
            themes: self.list_themes().await?,
            competitions: self.list_competitions().await?,
            ..Default::default()
        };

        for competition in &bundle.competitions {
            bundle.teams.extend(self.list_teams(competition.id).await?);
            bundle
                .participants
                .extend(self.list_participants(competition.id).await?);
            bundle.gymnastics_details.extend(
                self.list_gymnast_details_for_competition(competition.id)
                    .await?,
            );
            bundle
                .judges
                .extend(self.list_judges(competition.id).await?);
            bundle
                .sessions
                .extend(self.list_sessions(competition.id).await?);
            let events = self.list_events(competition.id).await?;
            for event in &events {
                let results = self.list_results_for_event(event.id).await?;
                for result in &results {
                    bundle
                        .scores
                        .extend(self.list_scores_for_result(result.id).await?);
                    if let Ok(routine) = self.get_gymnastics_routine(result.id).await {
                        bundle.gymnastics_routines.push(routine);
                    }
                }
                bundle.results.extend(results);
                bundle
                    .rankings
                    .extend(self.list_rankings_for_event(event.id).await?);
            }
            bundle.events.extend(events);
        }

        for path in bundle.referenced_asset_paths() {
            if let Ok(bytes) = std::fs::read(&path) {
                bundle.assets.push((path, bytes));
            }
        }

        Ok(bundle)
    }

    /// Writes `bundle` to `path` as a `.abyssal` zip. `Replace`: rows
    /// with matching ids are overwritten (safe to re-run, right for
    /// restoring/moving an instance). `DuplicateAsNew`: every id in
    /// the bundle is regenerated first, so this import can never
    /// collide with -- or overwrite -- anything already present.
    pub async fn import_bundle(
        &self,
        bundle: &ExportBundle,
        asset_dir: &Path,
        mode: ImportMode,
    ) -> DbResult<()> {
        let remapped;
        let bundle = match mode {
            ImportMode::Replace => bundle,
            ImportMode::DuplicateAsNew => {
                remapped = {
                    let mut b = bundle.clone();
                    b.remap_ids();
                    b
                };
                &remapped
            }
        };

        std::fs::create_dir_all(asset_dir)?;
        for (original_path, bytes) in &bundle.assets {
            let file_name = Path::new(original_path).file_name().map(|f| f.to_owned());
            if let Some(file_name) = file_name {
                std::fs::write(asset_dir.join(&file_name), bytes)?;
            }
        }

        for theme in &bundle.themes {
            self.import_theme_or_replace(theme).await?;
        }
        for competition in &bundle.competitions {
            self.import_competition_or_replace(competition).await?;
        }
        for team in &bundle.teams {
            self.import_team_or_replace(team).await?;
        }
        for participant in &bundle.participants {
            self.import_participant_or_replace(participant).await?;
        }
        for detail in &bundle.gymnastics_details {
            self.import_gymnast_detail_or_replace(detail).await?;
        }
        for judge in &bundle.judges {
            self.import_judge_or_replace(judge).await?;
        }
        for session in &bundle.sessions {
            self.import_session_or_replace(session).await?;
        }
        for event in &bundle.events {
            self.import_event_or_replace(event).await?;
        }
        for result in &bundle.results {
            self.import_result_or_replace(result).await?;
        }
        for routine in &bundle.gymnastics_routines {
            self.import_gymnastics_routine_or_replace(routine).await?;
        }
        for score in &bundle.scores {
            self.import_score_or_replace(score).await?;
        }
        for event in &bundle.events {
            let entries: Vec<_> = bundle
                .rankings
                .iter()
                .filter(|r| r.event_id == Some(event.id))
                .cloned()
                .collect();
            if !entries.is_empty() {
                self.set_rankings(event.id, &entries).await?;
            }
        }
        Ok(())
    }
}

impl ExportBundle {
    /// Assigns fresh ids to every entity in this bundle and rewrites
    /// every cross-reference to match, so importing it can never
    /// collide with -- or silently overwrite -- anything already in
    /// the target instance.
    pub fn remap_ids(&mut self) {
        fn remap(map: &mut HashMap<Uuid, Uuid>, id: Uuid) -> Uuid {
            *map.entry(id).or_insert_with(Uuid::new_v4)
        }

        let mut map: HashMap<Uuid, Uuid> = HashMap::new();

        for theme in &mut self.themes {
            theme.id = ThemeId(remap(&mut map, theme.id.0));
        }
        for competition in &mut self.competitions {
            competition.id = CompetitionId(remap(&mut map, competition.id.0));
            competition.theme_id = competition.theme_id.map(|t| ThemeId(remap(&mut map, t.0)));
        }
        for team in &mut self.teams {
            team.id = TeamId(remap(&mut map, team.id.0));
            team.competition_id = CompetitionId(remap(&mut map, team.competition_id.0));
        }
        for participant in &mut self.participants {
            participant.id = ParticipantId(remap(&mut map, participant.id.0));
            participant.competition_id =
                CompetitionId(remap(&mut map, participant.competition_id.0));
            participant.team_id = participant.team_id.map(|t| TeamId(remap(&mut map, t.0)));
        }
        for detail in &mut self.gymnastics_details {
            detail.participant_id = ParticipantId(remap(&mut map, detail.participant_id.0));
            detail.competition_id = CompetitionId(remap(&mut map, detail.competition_id.0));
        }
        for judge in &mut self.judges {
            judge.id = JudgeId(remap(&mut map, judge.id.0));
            judge.competition_id = CompetitionId(remap(&mut map, judge.competition_id.0));
        }
        for session in &mut self.sessions {
            session.id = SessionId(remap(&mut map, session.id.0));
            session.competition_id = CompetitionId(remap(&mut map, session.competition_id.0));
        }
        for event in &mut self.events {
            event.id = EventId(remap(&mut map, event.id.0));
            event.competition_id = CompetitionId(remap(&mut map, event.competition_id.0));
            event.session_id = event.session_id.map(|s| SessionId(remap(&mut map, s.0)));
        }
        for result in &mut self.results {
            result.id = ResultId(remap(&mut map, result.id.0));
            result.event_id = EventId(remap(&mut map, result.event_id.0));
            result.participant_id = result
                .participant_id
                .map(|p| ParticipantId(remap(&mut map, p.0)));
            result.team_id = result.team_id.map(|t| TeamId(remap(&mut map, t.0)));
        }
        for routine in &mut self.gymnastics_routines {
            routine.result_id = ResultId(remap(&mut map, routine.result_id.0));
        }
        for score in &mut self.scores {
            score.id = ScoreId(remap(&mut map, score.id.0));
            score.result_id = ResultId(remap(&mut map, score.result_id.0));
            score.judge_id = score.judge_id.map(|j| JudgeId(remap(&mut map, j.0)));
        }
        for ranking in &mut self.rankings {
            ranking.id = RankingId(remap(&mut map, ranking.id.0));
            ranking.event_id = ranking.event_id.map(|e| EventId(remap(&mut map, e.0)));
            ranking.participant_id = ranking
                .participant_id
                .map(|p| ParticipantId(remap(&mut map, p.0)));
            ranking.team_id = ranking.team_id.map(|t| TeamId(remap(&mut map, t.0)));
        }
    }

    fn referenced_asset_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        for theme in &self.themes {
            paths.extend(theme.background_image_path.clone());
            paths.extend(theme.logo_path.clone());
        }
        for team in &self.teams {
            paths.extend(team.logo_path.clone());
        }
        paths
    }

    pub fn write_to_zip(&self, path: impl AsRef<Path>) -> DbResult<()> {
        let file = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let metadata = ExportMetadata {
            schema_version: EXPORT_SCHEMA_VERSION,
            exported_at: chrono::Utc::now(),
        };
        zip.start_file("metadata.json", options)?;
        zip.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;

        zip.start_file("data.json", options)?;
        zip.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;

        for (original_path, bytes) in &self.assets {
            if let Some(file_name) = Path::new(original_path)
                .file_name()
                .and_then(|f| f.to_str())
            {
                zip.start_file(format!("assets/{}", file_name), options)?;
                zip.write_all(bytes)?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    pub fn read_from_zip(path: impl AsRef<Path>) -> DbResult<Self> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut data_str = String::new();
        archive
            .by_name("data.json")?
            .read_to_string(&mut data_str)?;
        let mut bundle: ExportBundle = serde_json::from_str(&data_str)?;

        let asset_names: Vec<String> = (0..archive.len())
            .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
            .filter(|n| n.starts_with("assets/"))
            .collect();
        for name in asset_names {
            let mut bytes = Vec::new();
            archive.by_name(&name)?.read_to_end(&mut bytes)?;
            bundle.assets.push((name, bytes));
        }

        Ok(bundle)
    }
}

pub fn default_asset_dir(db_path: impl AsRef<Path>) -> PathBuf {
    db_path
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("assets")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// Existing rows with matching ids are overwritten. Right for
    /// restoring or moving an instance onto a new computer.
    Replace,
    /// Every id in the bundle is regenerated before importing, so it
    /// always lands as new data alongside anything already present --
    /// never overwrites, never collides.
    DuplicateAsNew,
}
