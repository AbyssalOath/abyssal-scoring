#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::net::SocketAddr;
use std::sync::Arc;

use server::AppService;
use tauri::Manager;

/// WebKitGTK's DMA-BUF render path fails to allocate a GBM buffer on
/// a number of Linux GPU setups (NVIDIA proprietary drivers
/// especially), leaving the webview blank with no error dialog. This
/// has to be set before the webview is created, so it goes at the
/// very top of `main()`.
#[cfg(target_os = "linux")]
fn disable_dmabuf_renderer_if_needed() {
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

#[cfg(not(target_os = "linux"))]
fn disable_dmabuf_renderer_if_needed() {}

pub struct AppState {
    pub service: Arc<AppService>,
}

fn main() {
    disable_dmabuf_renderer_if_needed();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("no app data dir available for this platform");
            std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");
            let db_path = data_dir.join("abyssal.sqlite");

            // AppService::open runs migrations, so do it once up front
            // and block startup on it -- a local SQLite file opens in
            // milliseconds, this isn't worth making the splash screen
            // async for.
            let service = tauri::async_runtime::block_on(AppService::open(&db_path))
                .expect("failed to open local database");
            let service = Arc::new(service);

            // Embedded HTTP + WebSocket API for judges' iPads (and,
            // read-only, parent/spectator devices) on the local
            // network. Reuses the tokio runtime Tauri already owns --
            // no separate runtime to bootstrap. Binds on all
            // interfaces (not just loopback) since other devices on
            // the LAN need to reach it.
            let live_service = service.clone();
            tauri::async_runtime::spawn(async move {
                let addr = SocketAddr::from(([0, 0, 0, 0], live_server::DEFAULT_PORT));
                if let Err(e) = live_server::serve(live_service, addr).await {
                    eprintln!("live-scoring server failed to start: {e}");
                }
            });

            app.manage(AppState { service });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_competitions,
            commands::create_competition,
            commands::add_team,
            commands::set_team_colors,
            commands::add_participant,
            commands::parse_roster_file,
            commands::import_roster,
            commands::list_roster_with_gymnast_detail,
            commands::add_judge,
            commands::create_pairing_code,
            commands::list_judge_devices,
            commands::revoke_judge_device,
            commands::get_live_server_info,
            commands::add_session,
            commands::add_event,
            commands::list_teams,
            commands::list_participants,
            commands::list_judges,
            commands::list_events,
            commands::get_scoreboard_state,
            commands::set_current_participant,
            commands::show_result_on_scoreboard,
            commands::clear_scoreboard,
            commands::submit_gymnastics_panel_score,
            commands::submit_swimming_result,
            commands::get_participant,
            commands::get_event,
            commands::resolve_theme,
            commands::set_theme,
            commands::export_data,
            commands::import_data,
            commands::open_scoreboard_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Abyssal Scoring");
}
