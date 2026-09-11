# Changelog

All notable changes to this project are documented here. Format
loosely follows [Keep a Changelog](https://keepachangelog.com/), and
this project intends to follow [Semantic Versioning](https://semver.org/)
once it has a first tagged release.

## [Unreleased]

Nothing yet.

## [0.1.0] - 2026-09-10

First tagged release. Everything below is pre-`v0.1.0` history, from
initial scaffolding through getting the project ready for a public
GitHub push.

### Security
- Bumped `sqlx` 0.7.4 → 0.8.6 (`crates/db/Cargo.toml`), fixing
  [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363.html)
  ("Binary Protocol Misinterpretation caused by Truncating or
  Overflowing Casts", GHSA-xmrp-424f-vfpx) - a length-prefix overflow
  in sqlx's wire protocol encoding, patched in 0.8.1+. The advisory's
  demonstrated exploit is Postgres-specific and this project only
  uses the `sqlite` feature, but there was no reason not to take the
  fix. No code changes were needed - verified with `cargo
  check`/`test --workspace` plus a throwaway smoke test that opens an
  in-memory db and runs the real migrations against 0.8.6 before this
  was committed.
- **Known, currently unfixable**: Dependabot also flags
  `glib` 0.18.5 for
  [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
  (unsound `Iterator`/`DoubleEndedIterator` impls on
  `glib::VariantStrIter`, patched in glib 0.20+). `glib` isn't a
  direct dependency - it comes in transitively via Tauri's Linux GTK
  stack (`tauri` → `tauri-runtime-wry` → `tao`/`gtk`/`webkit2gtk`),
  and `tauri-runtime-wry 2.11.4` (the version behind the latest
  published `tauri 2.11.5`) pins `tao` to `^0.35.0`, which caps `gtk`
  at 0.18.x and therefore `glib` at 0.18.x too - confirmed by trying
  `cargo update -p tao --precise 0.37.0`, which cargo refuses for
  exactly this reason. Fixing this requires an upstream Tauri release
  that bumps its gtk-rs stack; nothing in this repo's Cargo.toml can
  reach a patched `glib` today. Revisit by re-running `cargo update
  -p tao --dry-run` after future Tauri upgrades.

### Added
- GitHub Actions CI workflow (`.github/workflows/ci.yml`): `cargo
  check`/`test`/`clippy`/`fmt`, matrixed across Linux/macOS/Windows,
  on every push and pull request.
- GitHub Actions release workflow (`.github/workflows/release.yml`):
  pushing a `v*` tag builds installers for Windows (`.msi`/NSIS),
  macOS (`.dmg`, Intel + Apple Silicon as separate builds), and Linux
  (`.AppImage`) and attaches them to a draft GitHub Release.
- Initial workspace: `models`, `scoring`, `db`, `server` crates plus
  a Tauri desktop shell and a plain HTML/CSS/JS frontend, built
  around a sport-agnostic domain model (Competition, Event,
  Participant, Team, Judge, Session, EventResult, Score,
  RankingEntry, ScoreboardState, ScoreboardTheme).
- Gymnastics scoring (`scoring::gymnastics::artistic`): D + E − ND,
  fully unit tested.
- SQLite persistence via `sqlx`, one file per instance, no server
  required.
- `.abyssal` export/import format: a zip of versioned JSON plus any
  referenced logo/background image assets.
- Scoreboard display window with a themeable, LED-scoreboard-style
  "Old School Grid" default (black background, red glowing digits,
  grid overlay), plus a "Modern" preset and full custom theming
  (colors, background image, logo, grid on/off).
- "Clear scoreboard for next event" - resets only the live display
  pointer; competitor/judge/event/result history is untouched.
- `CONTRIBUTING.md`, `SECURITY.md`, `ARCHITECTURE.md`, and
  `ABYSSAL_FORMAT.md`.
- **Multi-judge execution panels**: any number of judges can each
  submit their own deduction total for a routine; 2+ judges
  automatically drops the high and low score and averages the rest
  (`scoring::judging::average_drop_high_low`). Falls back to a
  single anonymous scorer when no judges are checked.
- **Swimming/track scoring** (`scoring::swimming::time_trial`):
  elapsed time + optional time penalty, ranked lowest-time-wins.
  Added without touching `models`, `db`, or any existing table -
  proof that the generic-infrastructure/sport-specific-scoring split
  holds up for a second sport.
- **Import modes**: choice of "Replace" (restore/move an instance,
  matching ids overwritten) or "Import as new copies" (every id in
  the bundle is regenerated before import, so it can never collide
  with or overwrite existing data).
- Native OS file dialogs (via `tauri-plugin-dialog`) for export
  (suggests `<competition>-<date>.abyssal`, filtered save dialog),
  import (filtered open dialog), and logo/background image pickers
  (filtered to common image formats) - replacing manual path text
  entry everywhere.
- Client-side validation on every form (required names, `YYYY-MM-DD`
  date format, non-negative scores/times/deductions) plus a
  `safeInvoke` wrapper on the frontend so any backend error surfaces
  as a visible alert instead of an unhandled promise rejection.

### Changed
- `bundle.targets` in `tauri.conf.json` narrowed from `"all"` to
  `["msi", "nsis", "dmg", "appimage"]` - drops `.deb`/`.rpm` in favor
  of shipping exactly one Windows format pair, one macOS format, and
  one distro-generic Linux format (AppImage) per release.
- **Migrated Tauri v1 → v2**, after Arch Linux (and the AUR) dropped
  the `webkit2gtk-4.0` package v1 depended on. v2 targets
  `webkit2gtk-4.1`, which current distros actually ship. Touched
  `tauri.conf.json` (rewritten for the v2 schema), window/webview
  APIs in `main.rs`/`commands.rs`, and the frontend's
  `window.__TAURI__` namespace.
- Dropped the original Docker/MariaDB server-mode scaffolding in
  favor of the local-first SQLite + Tauri design - a networked
  server deployment was the original plan before requirements
  shifted to "one executable, local database, no setup."
- Date input switched from a native `<input type="date">` to a plain
  text field with `YYYY-MM-DD` validation, after the native picker's
  popup proved unreliable to dismiss under WebKitGTK on Linux.
- Renamed the ad hoc `from_str` helpers on `Sport`, `EventStatus`,
  `DisplayMode`, and `ThemePreset` to `from_db_str` (clippy:
  `should_implement_trait` - they were shadowing
  `std::str::FromStr::from_str` in a way that could confuse callers).
- Removed the single-judge `submit_gymnastics_score` method/command
  once the panel-based version replaced it in the UI (clippy:
  `too_many_arguments`, plus it was simply dead code).
- `Cargo.lock` is now committed (previously gitignored) - correct
  default for an application, as opposed to a library.

### Fixed
- Release workflow (`release.yml`) failed on `ubuntu-22.04` with
  `apt-get` exit code 100 - the Linux dependency list installed both
  `libappindicator3-dev` and `libayatana-appindicator3-dev`, which
  provide conflicting versions of the same headers
  (`libayatana-appindicator3-dev : Conflicts: libappindicator3-dev`).
  Trimmed the install list in both `ci.yml` and `release.yml` back
  down to Tauri's own recommended minimal set (`libwebkit2gtk-4.1-dev
  libappindicator3-dev librsvg2-dev patchelf`); the others weren't
  needed anyway.
- CI workflow (`ci.yml`) never ran at all - it triggered on
  `branches: [main]`, but this repo's default branch is `master`.
  Fixed the trigger; caught via `gh run list --workflow=ci.yml`
  returning a 404 for "workflow not found on the default branch".
- Blank/invisible control-panel window on Linux with certain GPU
  drivers (`Failed to create GBM buffer` in the WebKitGTK log) -
  worked around by disabling WebKitGTK's DMA-BUF renderer on Linux
  at startup.
- Scoreboard display never actually showed a submitted score - the
  control panel was calling `set_current_participant` (which
  announces who's up next with *no* score) instead of a "show this
  finished result" action. Added `show_result_on_scoreboard` as a
  distinct command/service method and wired score submission to use
  it.
- `sqlx::migrate!`/`MigrateError` failed to resolve - the `migrate`
  Cargo feature wasn't enabled on the `sqlx` dependency.
- Team/theme color swatches (`<input type="color">`) stayed visually
  blank after picking a color, a known WebKitGTK-on-Linux rendering
  bug - worked around by overriding the native swatch styling and
  syncing the input's own background color via JS.
- A handful of unused-import clippy/rustc warnings (`DbError` in
  `judges.rs`/`teams.rs`, `ServiceError` in `service.rs`, a doc-list
  indentation warning in `artistic.rs`, a
  `field_reassign_with_default` warning in `export.rs`).
- License file was misspelled `LISCENSE`, which stops GitHub from
  auto-detecting the project's license - renamed to `LICENSE`.
- Stray em dash and a stale "icons may still be placeholders" note in
  `README.md` (icons had already been generated for real).
