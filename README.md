# Abyssal Scoring

A local-first, cross-platform (Windows/macOS/Linux) scoreboard app,
built as a Tauri desktop app around a generic competition data model
so it isn't locked to gymnastics. Ships with gymnastics and
swimming/track scoring today; adding another sport is a contained
change, not a rewrite - see [ARCHITECTURE.md](ARCHITECTURE.md) for
why, and for a worked example of adding one.

## Documentation map

- **README.md** (this file) - what it is, how to run it, current status
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - how the layers fit together, the data model, how to add a sport
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - dev setup, PR conventions
- **[WHAT_TO_TEST.md](WHAT_TO_TEST.md)** - pre-release manual test checklist
- **[ABYSSAL_FORMAT.md](ABYSSAL_FORMAT.md)** - the `.abyssal` export file format, field by field
- **[CHANGELOG.md](CHANGELOG.md)** - what's changed, in order
- **[SECURITY.md](SECURITY.md)** - vulnerability reporting, data-at-rest notes

## Architecture at a glance

```text
crates/
├── models/    Generic domain types: Competition, Event, Participant,
│              Team, Judge, Session, EventResult, Score, RankingEntry,
│              ScoreboardState, ScoreboardTheme — plus a gymnastics
│              submodule (Apparatus, Deduction, Routine) for the one
│              sport that needs its own dedicated fields so far.
│
├── scoring/   Sport scoring algorithms, one module per sport,
│              deliberately NOT behind one shared trait — see the
│              module doc comment in crates/scoring/src/lib.rs.
│              ├── gymnastics::artistic — D + E − ND, multi-judge
│              │   panel support (drop-high-low)
│              └── swimming::time_trial — elapsed time + penalties
│                  (also covers track)
│
├── db/        SQLite persistence (via sqlx) + the .abyssal
│              export/import format, with a choice of import modes.
│              One file per competition instance, no server required.
│
├── roster_import/ Parses a roster spreadsheet (.xlsx/.xls/.csv) or
│              PDF into a reviewable preview - column mapping guesses
│              + per-row warnings, never writes to the database itself.
│
├── server/    AppService: the orchestration layer everything above
│              is wired together behind. This is what every UI (the
│              desktop app, and live_server below) calls.
│
└── live_server/ Embedded HTTP + WebSocket API (axum) so judges'
               iPads can pair, submit scores, and see the live
               scoreboard pushed over the local network - LAN-only,
               no relay/internet component. See ARCHITECTURE.md.

src-tauri/     The Tauri v2 desktop app shell: control-panel window +
               a dynamically opened scoreboard display window, native
               file dialogs for export/import/logo pickers, and IPC
               commands in src/commands.rs delegating to AppService.

frontend/      Plain HTML/CSS/JS, no build step. index.html is the
               control panel; scoreboard.html is what you'd put on a
               second monitor / projector.
```

The guiding rule from the original planning document still holds:

**make the infrastructure generic, make the scoring rules specific.**

`server` and `db` don't know what a "deduction" or a "lap" is; only each sport's own module in `scoring` (and `models::gymnastics`, for the one sport that needs it) does.

Full detail is in [ARCHITECTURE.md](https://github.com/AbyssalOath/abyssal-scoring/blob/master/ARCHITECTURE.md).

## Requirements to build

- A current stable Rust toolchain (`rustup` recommended) - Tauri
  itself requires a recent compiler, so whatever you have for that
  will cover this workspace too.
- Node.js, if you later want to move `frontend/` onto a bundler
  (not required right now - it's plain static files).
- The [Tauri CLI](https://tauri.app): `cargo install tauri-cli --version "^2"`
- Platform build tools Tauri v2 needs (WebView2 on Windows, Xcode CLT
  on macOS, `webkit2gtk-4.1` + friends on Linux - see Tauri's
  ["Prerequisites"](https://v2.tauri.app/start/prerequisites/) page
  for the exact package list per OS). On Arch specifically, the
  official `webkit2gtk-4.1` package (not the old `webkit2gtk`/4.0,
  which Arch and the AUR have both dropped) is what this project is
  built against.

## Running it

```bash
cd abyssal-scoring
cargo tauri dev
```

This opens the control panel window. Use "Open scoreboard display
window" (in the Scoreboard & Theme tab) to pop the actual scoreboard
open - drag that window to a second monitor/projector and
full-screen it from there, or use the "Open fullscreen" button to
launch it full-screen directly.

The SQLite database lives in the OS's standard app-data directory
(wherever Tauri's `app.path().app_data_dir()` resolves to per
platform), as `abyssal.sqlite`. Nothing else needs to run - no
Docker, no separate server process to start by hand.

The app does listen on the local network (port 7878 by default) for
the "Live Judging" feature - judges' devices submitting scores and
seeing the live scoreboard. This is the same process, not a separate
one to manage, but expect a first-run OS firewall prompt (Windows
Defender / macOS Application Firewall) to allow it. See "Live Judging
(Beta)" in the control panel and [ARCHITECTURE.md](ARCHITECTURE.md)
for how it's authenticated.

**Linux note:** on some GPU setups (NVIDIA especially) WebKitGTK's
accelerated rendering fails silently and leaves the window blank.
`main.rs` unconditionally sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` on
Linux to route around this - see the comment above
`disable_dmabuf_renderer_if_needed()` if you ever need to revisit it.

## What actually works right now

- Create a competition (gymnastics, swimming, or track & field), add
  teams (with colors + logo), participants, and judges; build a
  schedule of events.
- Put a participant "up" on the live scoreboard.
- **Gymnastics scoring**: difficulty + an execution panel of any
  number of judges (each entering their own deduction total - with
  2+ judges checked, the high and low are automatically dropped and
  the rest averaged) + neutral deductions → computed via
  `scoring::gymnastics::artistic`, persisted, and standings for that
  event recomputed automatically. With zero judges checked, falls
  back to a single deduction-total field.
- **Swimming/track scoring**: elapsed time + optional time penalty →
  `scoring::swimming::time_trial`, ranked lowest-time-wins
  automatically (vs. highest-score-wins for gymnastics - the ranking
  direction is per-sport, not hardcoded).
- "Clear scoreboard for next event" resets only the live display
  pointer (`scoreboard_state` table) - competitors, judges, events,
  results and scores are all left alone. Reopening the app re-reads
  that same pointer and restores the previously active competition,
  so it picks up where it left off unless you explicitly clear it.
- **Roster import**: import a TAAF-style roster (.xlsx/.xls/.csv or
  PDF) - auto-guesses the column mapping, lets you fix it and mark who
  isn't attending before confirming, then upserts by TAAF athlete #
  so a re-sent weekly roster updates existing athletes instead of
  duplicating them. A "Rotation Sheet" tab shows attending athletes
  grouped by division or gym, with a print-friendly view.
- Theme editor: pick a preset or dial in your own background/primary/
  accent colors, background image, and logo - both picked via native
  file dialogs, filtered to image files. Default is the "Old School
  Grid" preset - black background, red glowing digits, faint grid
  overlay - matching a classic gym scoreboard.
- Export the whole instance to a portable `.abyssal` file (native
  save dialog, filtered to `.abyssal`, filename suggested from the
  competition name + date) - a zip of versioned JSON plus any
  referenced logo/background images.
- Import an `.abyssal` file (native open dialog, filtered to
  `.abyssal`) with a choice of **Replace** (restore/move an instance -
  matching ids are overwritten) or **Import as new copies** (every id
  in the file is regenerated first, so it's added alongside existing
  data and can never collide with or overwrite it).
- Basic client-side validation on every form (required names, date
  format, non-negative scores/times/deductions) plus a catch-all that
  surfaces any backend error via an alert instead of failing silently.
- **Live Judging (Beta)**: pair a device to a judge with a short-lived
  code, then submit gymnastics/swimming scores over the local
  network's REST API exactly as the desktop UI would - same scoring
  math, same "goes straight to the live scoreboard" behavior. No
  native iPad app exists yet (see below); this is the backend/API
  layer, directly testable with curl/websocat today.

## What's stubbed or missing (next steps)

- **The native iPad app (`ios/AbyssalLive`) exists but is unbuilt and
  untested.** It's a real SwiftUI app - judge pairing/scoring and a
  spectator scoreboard view, against the same `live_server` API a
  `curl` request already exercises - but it was written without access
  to a Mac/Xcode/Swift toolchain, so treat it as a scaffold to open,
  build, and shake out in Xcode rather than verified working code (the
  Rust side has real automated tests; the Swift side doesn't yet). See
  [`ios/README.md`](ios/README.md). Neither role supports viewing from
  off the venue's network - that would need a hosted relay component,
  a deliberately separate, not-yet-decided piece of work.
- **Windows/macOS builds are untested by hand.** Only Linux (Arch) has
  been run through end-to-end manual testing so far. The release
  workflow below builds all three platforms in CI, but nobody has
  clicked through a Windows or macOS install yet - see
  [WHAT_TO_TEST.md](WHAT_TO_TEST.md) before calling this releasable
  on those platforms.
- A third sport (something non-time-based and non-D/E-scored, e.g.
  something judged on a fixed-point rubric) hasn't been added yet -
  gymnastics and swimming cover "subjective panel score" and "lower
  time wins" respectively, but a third data shape would stress-test
  the architecture further.
- No automated integration tests - see "Known technical debt" in
  [ARCHITECTURE.md](ARCHITECTURE.md).

## Releases

`.github/workflows/ci.yml` runs `cargo check`/`test`/`clippy`/`fmt` on
every push and PR, matrixed across Linux/macOS/Windows.

`.github/workflows/release.yml` builds and drafts a GitHub Release
whenever a tag matching `v*` (e.g. `v0.1.0`) is pushed - one job per
platform/architecture, each running `cargo tauri build` and attaching
the resulting installer to a **draft** release:

- **Windows** - `.msi` and `-setup.exe` (NSIS)
- **macOS** - `.dmg`, built separately for Apple Silicon
  (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`) - no
  universal binary, so both are attached
- **Linux** - `.AppImage` only (not `.deb`/`.rpm`) - the one
  genuinely distro-generic format; built on `ubuntu-22.04` (rather
  than `-latest`) specifically to keep the glibc requirement low
  enough to run on older distros too

`src-tauri/tauri.conf.json`'s `bundle.targets` controls this list -
edit it (and the matching `releaseBody` in `release.yml`) if you want
`.deb`/`.rpm` back.

## License

[GNU AGPLv3](LICENSE).
