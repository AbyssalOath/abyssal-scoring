//! Orchestration layer: wires `models` + `scoring` + `db` together
//! behind one `AppService`. This is what the Tauri commands (and any
//! future UI -- desktop, tablet, web) call. It never contains
//! scoring rules itself -- see `scoring` -- and never contains SQL --
//! see `db`.

pub mod error;
pub mod ranking;
pub mod service;

pub use error::{ServiceError, ServiceResult};
pub use service::{AppService, JudgeDeductions};
