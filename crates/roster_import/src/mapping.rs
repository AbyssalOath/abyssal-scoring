use crate::types::RosterField;

/// Lowercases and strips everything but letters/digits, so header text
/// like "2026 TAAF Registration #" or "A&T" reduces to something a
/// substring match can key off reliably regardless of punctuation,
/// spacing, or the exact wording a given week's roster uses.
fn normalize(header: &str) -> String {
    header
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Best-guess mapping from a header row to the known roster fields.
/// Order matters: more specific checks run before looser ones so, for
/// example, "TAAF Athlete Competition #" doesn't fall through to a
/// generic "athlete" match meant for something else.
pub fn guess_column_mapping(headers: &[String]) -> Vec<Option<RosterField>> {
    headers.iter().map(|h| guess_one(&normalize(h))).collect()
}

fn guess_one(norm: &str) -> Option<RosterField> {
    if norm.contains("first") {
        return Some(RosterField::FirstName);
    }
    if norm.contains("last") {
        return Some(RosterField::LastName);
    }
    if norm.contains("athlete") && norm.contains("competition") {
        return Some(RosterField::TaafAthleteNumber);
    }
    if norm.contains("registration") {
        return Some(RosterField::TaafRegistrationNumber);
    }
    if norm.contains("division") {
        return Some(RosterField::Division);
    }
    if norm.contains("qual") {
        return Some(RosterField::QualStatus);
    }
    if norm.contains("gym") {
        return Some(RosterField::Gym);
    }
    // "A&T" normalizes to exactly "at" -- checked before the generic
    // "athlete" fallback below, and as an exact match rather than
    // `contains`, so it doesn't swallow headers like "Athlete" or
    // "State" that happen to contain those letters in sequence.
    if norm == "at" {
        return Some(RosterField::AtCode);
    }
    // ProScore exports this same number under a plainer header, just
    // "Athlete #" with no "Competition" in it -- fall back to a bare
    // "athlete" match so those files map correctly too.
    if norm.contains("athlete") {
        return Some(RosterField::TaafAthleteNumber);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_taaf_headers() {
        let headers = vec![
            "First_name".to_string(),
            "Last_name".to_string(),
            "A&T".to_string(),
            "Gym".to_string(),
            "TAAF Athlete Competition #".to_string(),
            "2026 TAAF Registration #".to_string(),
            "Next Meet Division 4/10/2026".to_string(),
            "*PRELIMINARY* Qual State".to_string(),
        ];
        let mapping = guess_column_mapping(&headers);
        assert_eq!(
            mapping,
            vec![
                Some(RosterField::FirstName),
                Some(RosterField::LastName),
                Some(RosterField::AtCode),
                Some(RosterField::Gym),
                Some(RosterField::TaafAthleteNumber),
                Some(RosterField::TaafRegistrationNumber),
                Some(RosterField::Division),
                Some(RosterField::QualStatus),
            ]
        );
    }

    #[test]
    fn unrecognized_header_maps_to_none() {
        assert_eq!(guess_one(&normalize("Notes")), None);
    }

    #[test]
    fn maps_proscores_plain_athlete_header() {
        // ProScore calls this same number just "Athlete #", with no
        // "Competition" in the header the TAAF roster uses.
        assert_eq!(
            guess_one(&normalize("Athlete #")),
            Some(RosterField::TaafAthleteNumber)
        );
    }
}
