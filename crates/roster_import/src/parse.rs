use std::path::Path;

use crate::mapping::guess_column_mapping;
use crate::types::{ParsedRosterPreview, RosterField, RosterImportError, RosterImportResult};

/// The roster's fixed banner cell -- not data, just a header artifact
/// every week's sheet carries. Detected so it's skipped rather than
/// mistaken for the header row.
const BANNER_NEEDLE: &str = "email inquir";

pub fn parse_file(path: impl AsRef<Path>) -> RosterImportResult<ParsedRosterPreview> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("csv") => parse_csv(path),
        Some("xlsx") | Some("xls") | Some("xlsb") | Some("ods") => parse_excel(path),
        Some("pdf") => parse_pdf(path),
        other => Err(RosterImportError::UnsupportedFormat(
            other.map(String::from),
        )),
    }
}

fn parse_csv(path: &Path) -> RosterImportResult<ParsedRosterPreview> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(path)?;

    let mut grid = Vec::new();
    for record in reader.records() {
        let record = record?;
        grid.push(record.iter().map(|f| f.to_string()).collect::<Vec<_>>());
    }

    build_preview(grid, Vec::new())
}

fn parse_excel(path: &Path) -> RosterImportResult<ParsedRosterPreview> {
    use calamine::{open_workbook_auto, Reader};

    let mut workbook =
        open_workbook_auto(path).map_err(|e| RosterImportError::Calamine(e.to_string()))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or(RosterImportError::Empty)?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| RosterImportError::Calamine(e.to_string()))?;

    let grid: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(|cell| cell.to_string()).collect())
        .collect();

    build_preview(grid, Vec::new())
}

fn parse_pdf(path: &Path) -> RosterImportResult<ParsedRosterPreview> {
    let text =
        pdf_extract::extract_text(path).map_err(|e| RosterImportError::Pdf(e.to_string()))?;

    let grid: Vec<Vec<String>> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(split_columns_by_whitespace_runs)
        .collect();

    let notes = vec![
        "Parsed from a PDF: column alignment is approximate. Verify every row before importing."
            .to_string(),
    ];
    build_preview(grid, notes)
}

/// PDF text extraction loses the original table structure, but a
/// roster laid out in columns still has runs of 2+ spaces between
/// cells -- this recovers columns from that spacing instead of
/// requiring a fixed layout.
fn split_columns_by_whitespace_runs(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut space_run = 0usize;

    for ch in line.chars() {
        if ch == ' ' {
            space_run += 1;
        } else {
            if space_run >= 2 {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    fields.push(trimmed.to_string());
                }
                current.clear();
            }
            space_run = 0;
            current.push(ch);
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        fields.push(trimmed.to_string());
    }
    fields
}

fn is_blank_row(row: &[String]) -> bool {
    row.iter().all(|cell| cell.trim().is_empty())
}

fn build_preview(
    mut grid: Vec<Vec<String>>,
    mut notes: Vec<String>,
) -> RosterImportResult<ParsedRosterPreview> {
    grid.retain(|row| !is_blank_row(row));
    if grid.is_empty() {
        return Err(RosterImportError::Empty);
    }

    // Skip the roster's fixed "email inquiries to..." banner row, if
    // present, to find the real header row underneath it.
    if grid[0]
        .iter()
        .any(|cell| cell.to_lowercase().contains(BANNER_NEEDLE))
    {
        grid.remove(0);
    }
    if grid.is_empty() {
        return Err(RosterImportError::Empty);
    }

    let headers = grid.remove(0);
    let width = headers.len();

    let rows: Vec<Vec<String>> = grid
        .into_iter()
        .map(|mut row| {
            row.resize(width, String::new());
            row.truncate(width);
            row
        })
        .collect();

    let column_mapping = guess_column_mapping(&headers);
    let row_warnings = rows
        .iter()
        .map(|row| warnings_for_row(&column_mapping, row))
        .collect();

    if !column_mapping.contains(&Some(RosterField::TaafAthleteNumber)) {
        notes.push(
            "No column was recognized as the TAAF athlete number -- re-imports won't be able to \
             match this file's rows against existing athletes without it."
                .to_string(),
        );
    }

    Ok(ParsedRosterPreview {
        headers,
        column_mapping,
        rows,
        row_warnings,
        notes,
    })
}

fn warnings_for_row(mapping: &[Option<RosterField>], row: &[String]) -> Vec<String> {
    let mut warnings = Vec::new();
    let find = |field: RosterField| {
        mapping
            .iter()
            .position(|m| *m == Some(field))
            .and_then(|i| row.get(i))
            .map(|s| s.trim())
    };

    if find(RosterField::FirstName).unwrap_or("").is_empty() {
        warnings.push("Missing first name".to_string());
    }
    if find(RosterField::LastName).unwrap_or("").is_empty() {
        warnings.push("Missing last name".to_string());
    }
    if find(RosterField::TaafAthleteNumber)
        .unwrap_or("")
        .is_empty()
    {
        warnings.push("Missing TAAF athlete #".to_string());
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_banner_row_and_maps_headers() {
        let grid = vec![
            vec!["email inquiries to: TAAFscoremaster@gmail.com".to_string()],
            vec![
                "First_name".to_string(),
                "Last_name".to_string(),
                "TAAF Athlete Competition #".to_string(),
            ],
            vec!["Jane".to_string(), "Doe".to_string(), "12345".to_string()],
            vec!["John".to_string(), "".to_string(), "".to_string()],
        ];
        let preview = build_preview(grid, Vec::new()).unwrap();
        assert_eq!(
            preview.headers,
            vec!["First_name", "Last_name", "TAAF Athlete Competition #"]
        );
        assert_eq!(preview.rows.len(), 2);
        assert!(preview.row_warnings[0].is_empty());
        assert!(preview.row_warnings[1].contains(&"Missing last name".to_string()));
        assert!(preview.row_warnings[1].contains(&"Missing TAAF athlete #".to_string()));
    }

    #[test]
    fn empty_input_is_an_error() {
        assert!(matches!(
            build_preview(vec![], Vec::new()),
            Err(RosterImportError::Empty)
        ));
    }

    #[test]
    fn splits_pdf_style_columns_on_wide_gaps() {
        let fields = split_columns_by_whitespace_runs("Jane   Doe        12345");
        assert_eq!(
            fields,
            vec!["Jane".to_string(), "Doe".to_string(), "12345".to_string()]
        );
    }
}
