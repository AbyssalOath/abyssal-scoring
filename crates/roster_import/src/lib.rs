//! Parses a TAAF roster spreadsheet/PDF into a reviewable preview
//! (headers + best-guess column mapping + raw rows + warnings). This
//! crate never touches a database or decides what to import -- it
//! only turns a file into something a human can review and correct,
//! following the same "pure function in, structured data out" shape
//! as `scoring`.

mod mapping;
mod parse;
mod types;

pub use mapping::guess_column_mapping;
pub use parse::parse_file;
pub use types::{
    ParsedRosterPreview, RosterField, RosterImportError, RosterImportResult, RosterRow,
};
