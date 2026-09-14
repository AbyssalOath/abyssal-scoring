use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The TAAF rotation-sheet columns a roster row can map to. Kept
/// specific to what the sheet actually contains rather than generic
/// "column A/B/C" so both the auto-guessed mapping and any manual fix
/// the coach makes in the preview UI have real names to work with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RosterField {
    FirstName,
    LastName,
    /// The roster's "A&T" column -- even the coach isn't sure what it
    /// stands for, and it's usually blank in practice; stored verbatim
    /// rather than guessed at.
    AtCode,
    Gym,
    TaafAthleteNumber,
    TaafRegistrationNumber,
    /// "Next Meet Division" column.
    Division,
    /// "*PRELIMINARY* Qual State" column.
    QualStatus,
}

/// The result of parsing a roster file: a raw grid plus a best-guess
/// column mapping and any warnings, for a human to review and correct
/// before anything is imported. Parsing never writes to the database
/// on its own.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRosterPreview {
    pub headers: Vec<String>,
    /// Same length as `headers`; `None` where no field could be
    /// guessed (or the column isn't one of the known roster fields).
    pub column_mapping: Vec<Option<RosterField>>,
    /// Raw data rows, each the same length as `headers`.
    pub rows: Vec<Vec<String>>,
    /// Same length as `rows`; per-row issues found during parsing
    /// (e.g. a missing TAAF athlete number).
    pub row_warnings: Vec<Vec<String>>,
    /// File-level notes, e.g. flagging that a PDF source is
    /// approximate and every row should be checked by hand.
    pub notes: Vec<String>,
}

/// One normalized roster entry, as sent to `import_roster` after the
/// coach has reviewed/edited the parsed preview. This -- not a
/// re-parse of the source file -- is authoritative: edits made in the
/// review step take precedence over whatever was auto-detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterRow {
    pub first_name: String,
    pub last_name: String,
    pub at_code: Option<String>,
    pub gym: Option<String>,
    pub taaf_athlete_number: Option<String>,
    pub taaf_registration_number: Option<String>,
    pub division: Option<String>,
    pub qual_status: Option<String>,
    pub is_attending: bool,
}

#[derive(Debug, Error)]
pub enum RosterImportError {
    #[error("unsupported file extension: {0:?}")]
    UnsupportedFormat(Option<String>),
    #[error("the file has no rows to import")]
    Empty,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("spreadsheet error: {0}")]
    Calamine(String),
    #[error("pdf error: {0}")]
    Pdf(String),
}

pub type RosterImportResult<T> = Result<T, RosterImportError>;
