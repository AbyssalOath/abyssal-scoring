use models::{RankingEntry, RankingId};

/// Whether a bigger `final_value` is better for this sport. Gymnastics
/// scores are point totals (higher wins); swimming/track results are
/// times in seconds (lower wins). Each sport module in `scoring`
/// could eventually own this instead, but it's a one-line fact per
/// sport, not an algorithm, so it lives here for now.
pub fn higher_is_better(sport: models::Sport) -> bool {
    match sport {
        models::Sport::Gymnastics => true,
        models::Sport::Swimming => false,
        models::Sport::TrackAndField => false,
    }
}

/// Standard competition ranking (ties share a rank; the next distinct
/// value skips ahead by the number of tied entries), e.g. 1, 2, 2, 4.
pub fn rank_results(
    event_id: models::EventId,
    mut entries: Vec<(Option<models::ParticipantId>, Option<models::TeamId>, f64)>,
    higher_is_better: bool,
) -> Vec<RankingEntry> {
    entries.sort_by(|a, b| {
        if higher_is_better {
            b.2.partial_cmp(&a.2).unwrap()
        } else {
            a.2.partial_cmp(&b.2).unwrap()
        }
    });

    let mut out = Vec::with_capacity(entries.len());
    let mut rank = 0i32;
    let mut previous_value: Option<f64> = None;
    for (index, (participant_id, team_id, value)) in entries.into_iter().enumerate() {
        if previous_value != Some(value) {
            rank = index as i32 + 1;
        }
        previous_value = Some(value);
        out.push(RankingEntry {
            id: RankingId::new(),
            event_id: Some(event_id),
            participant_id,
            team_id,
            rank,
            score: value,
        });
    }
    out
}
