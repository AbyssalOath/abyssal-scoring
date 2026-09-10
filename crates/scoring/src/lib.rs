//! Deliberately NOT a `trait UniversalSportsScoringEngine`. Gymnastics
//! scoring is different enough from swimming/track that a forced
//! common interface would make both worse. Instead, each sport gets
//! its own module with its own input/output types; `server` calls
//! whichever module matches the competition's `Sport` and treats the
//! result the same way afterward (see `models::EventResult`).

pub mod error;
pub mod judging;

pub mod gymnastics;
pub mod swimming;

pub use error::ScoringError;
