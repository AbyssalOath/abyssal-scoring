//! Exercises the roster-import schema/CRUD end to end against a real
//! (in-memory) SQLite connection -- `sqlx::query`'s column names are
//! only checked at runtime, so this is what would actually catch a
//! typo'd column or a migration that doesn't apply cleanly.

use chrono::NaiveDate;
use db::export::ExportBundle;
use db::Db;
use models::gymnastics::GymnastDetail;
use models::{Competition, Participant, Sport};

async fn seed_competition(db: &Db) -> Competition {
    let competition = Competition::new(
        "Test Meet",
        Sport::Gymnastics,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    );
    db.create_competition(&competition).await.unwrap();
    competition
}

#[tokio::test]
async fn upserts_by_taaf_athlete_number() {
    let db = Db::open_in_memory().await.unwrap();
    let competition = seed_competition(&db).await;

    let mut participant = Participant::new(competition.id, "Jane", "Doe");
    db.create_participant(&participant).await.unwrap();

    let mut detail = GymnastDetail::new(participant.id, competition.id);
    detail.taaf_athlete_number = Some("12345".to_string());
    detail.club = Some("Acme Gymnastics".to_string());
    db.create_gymnast_detail(&detail).await.unwrap();

    let found = db
        .find_gymnast_detail_by_taaf_number(competition.id, "12345")
        .await
        .unwrap()
        .expect("should find the athlete by TAAF number");
    assert_eq!(found.club.as_deref(), Some("Acme Gymnastics"));

    // Simulates a weekly re-import: same TAAF #, updated fields.
    participant.last_name = "Smith".to_string();
    participant.is_attending = false;
    db.update_participant(&participant).await.unwrap();
    let reloaded = db.get_participant(participant.id).await.unwrap();
    assert_eq!(reloaded.last_name, "Smith");
    assert!(!reloaded.is_attending);

    detail.club = Some("New Gym".to_string());
    db.update_gymnast_detail(&detail).await.unwrap();
    let reloaded_detail = db.get_gymnast_detail(participant.id).await.unwrap();
    assert_eq!(reloaded_detail.club.as_deref(), Some("New Gym"));

    let all = db
        .list_gymnast_details_for_competition(competition.id)
        .await
        .unwrap();
    assert_eq!(all.len(), 1);

    let none = db
        .find_gymnast_detail_by_taaf_number(competition.id, "no-such-number")
        .await
        .unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn export_bundle_round_trips_gymnastics_details_and_stays_backward_compatible() {
    let db = Db::open_in_memory().await.unwrap();
    let competition = seed_competition(&db).await;
    let participant = Participant::new(competition.id, "Jane", "Doe");
    db.create_participant(&participant).await.unwrap();
    let mut detail = GymnastDetail::new(participant.id, competition.id);
    detail.taaf_athlete_number = Some("999".to_string());
    db.create_gymnast_detail(&detail).await.unwrap();

    let bundle = db.export_all().await.unwrap();
    assert_eq!(bundle.gymnastics_details.len(), 1);
    assert_eq!(
        bundle.gymnastics_details[0].taaf_athlete_number.as_deref(),
        Some("999")
    );

    // An older `.abyssal` export, made before this field existed,
    // must still deserialize (via `#[serde(default)]`).
    let mut value = serde_json::to_value(&bundle).unwrap();
    value.as_object_mut().unwrap().remove("gymnastics_details");
    let restored: ExportBundle = serde_json::from_value(value).unwrap();
    assert!(restored.gymnastics_details.is_empty());
}
