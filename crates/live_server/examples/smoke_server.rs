//! Standalone smoke-test server: seeds one competition/judge/event/
//! participant into an in-memory database and serves the live-scoring
//! API on `127.0.0.1:7878`, so the whole API can be poked with plain
//! curl/websocat without building or launching the full Tauri app.
//!
//! Run: `cargo run -p live_server --example smoke_server`

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::NaiveDate;
use models::{JudgeRole, Sport};
use server::AppService;

#[tokio::main]
async fn main() {
    let service = Arc::new(
        AppService::open_in_memory()
            .await
            .expect("open in-memory db"),
    );

    let competition = service
        .create_competition(
            "Smoke Test Meet",
            Sport::Gymnastics,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        )
        .await
        .unwrap();
    let judge = service
        .add_judge(competition.id, "Smoke Test Judge", JudgeRole::Execution)
        .await
        .unwrap();
    let event = service.add_event(competition.id, "Beam", 1).await.unwrap();
    let participant = service
        .add_participant(competition.id, "Jane", "Doe")
        .await
        .unwrap();
    let pairing = service.create_pairing_code(judge.id).await.unwrap();

    let port = live_server::DEFAULT_PORT;
    println!("competition_id={}", competition.id);
    println!("event_id={}", event.id);
    println!("participant_id={}", participant.id);
    println!("pairing_code={}", pairing.code);
    println!();
    println!("curl -s http://127.0.0.1:{port}/api/v1/health");
    println!(
        "curl -s -X POST http://127.0.0.1:{port}/api/v1/pair -H 'content-type: application/json' -d '{{\"code\":\"{}\",\"deviceName\":\"curl\"}}'",
        pairing.code
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    live_server::serve(service, addr)
        .await
        .expect("server error");
}
