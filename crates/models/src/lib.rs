//! Generic competition domain model, shared across every sport.
//! Sport-specific concepts (gymnastics routines, deductions,
//! apparatus, ...) live in their own submodule (see [`gymnastics`])
//! and hang off the generic types by id, rather than the generic
//! types knowing sport-specific fields exist.

pub mod competition;
pub mod event;
pub mod ids;
pub mod judge;
pub mod judge_device;
pub mod participant;
pub mod ranking;
pub mod result;
pub mod score;
pub mod scoreboard_state;
pub mod session;
pub mod team;
pub mod theme;

pub mod gymnastics;

pub use competition::{Competition, Sport};
pub use event::{Event, EventStatus};
pub use ids::*;
pub use judge::{Judge, JudgeRole};
pub use judge_device::{JudgeDevice, PairingCode};
pub use participant::Participant;
pub use ranking::RankingEntry;
pub use result::EventResult;
pub use score::Score;
pub use scoreboard_state::{DisplayMode, ScoreboardState};
pub use session::Session;
pub use team::Team;
pub use theme::{ScoreboardTheme, ThemePreset};
