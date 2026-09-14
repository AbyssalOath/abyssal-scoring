# Architecture

This is the deeper technical reference. If you just want to run the
app, see the README; if you want to contribute, see
`CONTRIBUTING.md`. This document is for understanding *why* things
are shaped the way they are, and how to extend them correctly.

## The one rule everything else follows

> Make the infrastructure generic; make the scoring rules specific.

Concretely: `competitions`, `events`, `participants`, `teams`,
`judges`, `sessions`, `results`, `scores`, and `rankings` - the
tables/types that exist in `models` and `db` at top level - must
never contain a `match sport { ... }` or a gymnastics/swimming-shaped
field. If a change to `db` or `server` needs one outside of
dispatching to a sport-specific module, that change belongs
somewhere else.

This is what makes "add a second sport" a contained change instead of
a rewrite. Swimming proves it: it shipped with **zero** new tables,
zero new `models` types, and one new method on `AppService`. See
["Adding a new sport"](#adding-a-new-sport-a-worked-example) below.

## Layering

models → scoring → db → server → src-tauri / frontend

- **`models`** - plain data. No I/O, no async, no business logic.
  Every entity has a strongly-typed UUID (`CompetitionId`,
  `EventId`, etc. - see `models::ids`) so it's a compile error to
  pass a `TeamId` where a `ParticipantId` is expected.
- **`scoring`** - pure functions. Given validated input, returns a
  computed result or a `ScoringError`. No knowledge of SQLite, Tauri,
  or even that a database exists. Fully unit tested; this is the
  crate where correctness matters most and is cheapest to verify.
- **`db`** - SQLite persistence via `sqlx`, plus the `.abyssal`
  export/import format. Knows nothing about scoring math.
- **`server`** - `AppService`, the orchestration layer. Wires `db` +
  `scoring` together: hands `scoring` some numbers, gets a result
  back, persists it via `db`. This is what every UI (the current
  Tauri app, or a hypothetical future tablet/web client) calls
  through.
- **`src-tauri` / `frontend`** - the actual desktop app. Commands in
  `src-tauri/src/commands.rs` are thin - each one locks the
  `AppService` mutex, calls exactly one method, maps errors to
  strings. No business logic lives here.

## Data model

                competitions
                     │
                     ├──────────────┐
                     ▼              ▼
                   events         teams
                     │              │
          ┌──────────┴──────────┐   │
          ▼                     ▼   │
    participants  ◄─────────────┘   │
          │                          │
          └──────────┬───────────────┘
                     ▼
                  results
                     │
              ┌──────┴──────┐
              ▼             ▼
           scores       rankings
              │
              ▼
     gymnastics_routines
    (only for gymnastics results;
     swimming results have no
     sport-specific table at all)

Plus: `judges` (per competition), `sessions` (optional grouping of
events), `themes` (referenced by competitions), and
`scoreboard_state` (one row per competition - see below).

Every table lives in `crates/db/migrations/0001_init.sql`. There's
been exactly one migration so far; when the schema needs to change,
add a new numbered migration file rather than editing the first one.

## The scoreboard state machine

`scoreboard_state` is a **singleton row per competition** - the only
thing that represents "what's currently on the display." It is
deliberately the *only* thing `ScoreboardState.clear()` /
`clear_scoreboard` touches.

DisplayMode:
Idle → nothing live (e.g. a title/sponsor screen)
InProgress → a participant is announced, no score yet
(set_current_participant)
ShowingResult → a finished score/time is on screen
(show_result_on_scoreboard)
ShowingRanking → (reserved; not wired into the UI yet)


"Stays where they left off, or clear for the next event" maps
directly onto this: relaunching the app just re-reads this one row
(`resume_scoreboard`); "Clear" resets it to `Idle` and nothing else.
Every other table - `events`, `participants`, `judges`, `results`,
`scores` - is untouched by either action. This split was the reason
`scoreboard_state` is its own table instead of a field bolted onto
`competitions`: it needed to be trivially resettable without any risk
of an accidental `DELETE` or `UPDATE` touching real history.

## Scoring: gymnastics

`scoring::gymnastics::artistic` implements the Code of Points shape:
**final = Difficulty (D) + Execution (E) − Neutral Deductions (ND)**.

Two entry points:
- `calculate(&RoutineInput) -> RoutineScore` - single deduction list,
  one number in, one number out. This was the original, single-judge
  implementation.
- `calculate_from_panel(&JudgePanelInput) -> PanelRoutineScore` - the
  one the UI actually calls. Takes `Vec<Vec<Deduction>>` (one
  deduction list per judge), converts each to a raw execution score
  (`start_value − sum(deductions)`), then combines them via
  `scoring::judging::average_drop_high_low` (drops the single highest
  and lowest score, averages the rest - falls back to a plain average
  with 2 or fewer judges). The combined execution score then goes
  through the same D + E − ND formula.

`scoring::judging` is intentionally separate from `scoring::gymnastics`
- "how do N judges' raw numbers become one number" is a genuinely
sport-agnostic technique (also used in diving, for example), unlike
the D/E/ND formula itself, which is gymnastics-specific.

## Scoring: swimming / track

`scoring::swimming::time_trial` is close to the simplest possible
scoring module: elapsed time plus an optional penalty, added
together, lower is better. No difficulty, no execution, no
deductions. This is intentional - it's the architecture's proof
point that a sport doesn't have to be forced into gymnastics' shape.

`AppService::submit_swimming_result` writes `EventResult.detail` as
plain JSON (`{"raw_seconds", "penalty_seconds", "final_seconds"}`)
with **no dedicated table** - unlike gymnastics, which gets
`gymnastics_routines` because it has enough structured, queryable
data (apparatus, deduction list) to be worth its own table. Swimming
doesn't need that yet; if it later needs, say, a per-lap split-time
breakdown, that's when a `swimming_*` table would get added, exactly
the way `gymnastics_routines` was.

## Ranking direction is per-sport, not hardcoded

`server::ranking::higher_is_better(sport: Sport) -> bool` is the one
place that knows gymnastics scores rank highest-first and
swimming/track times rank lowest-first. `rank_results` implements
standard competition ranking (ties share a rank; the next distinct
value skips ahead by the number of tied entries - 1, 2, 2, 4) against
whichever direction it's told to use. Adding a sport with the
opposite convention from both existing ones is a one-line addition to
that `match`.

## The theme system

`ScoreboardTheme` (background/primary/accent colors, optional
background image, optional logo, font family, grid-overlay toggle) is
attached to a `Competition` by id, with a hardcoded fallback
(`ScoreboardTheme::old_school_grid()`) when none is set.

The scoreboard display window (`frontend/scoreboard.js`) polls
`resolve_theme` and writes the result straight into CSS custom
properties (`--abyssal-bg`, `--abyssal-primary`, etc. - see
`frontend/theme.css`). This means every visual element reads the
theme through CSS, not through JS branching - adding a new themeable
property is "add a CSS variable + add it to the poll/apply function,"
not "rewrite the rendering."

Background images and logos are picked via native file dialogs
(`tauri-plugin-dialog`, filtered to image formats) and referenced by
absolute path; `convertFileSrc` turns that into something the webview
can actually load in an `<img>`/`background-image`.

## Export / import (`.abyssal`)

Full field-by-field format reference: **[ABYSSAL_FORMAT.md](ABYSSAL_FORMAT.md)**.

A `.abyssal` file is a zip containing:
- `metadata.json` - `{schema_version, exported_at}`
- `data.json` - the full `ExportBundle` (every competition, plus
  everything hanging off it: teams, participants, judges, sessions,
  events, results, scores, rankings, gymnastics routines, themes)
- `assets/<filename>` - any logo/background image files referenced
  by an exported theme or team, copied in by content so the export
  is self-contained

`Db::export_all()` walks every competition in the instance and
assembles one `ExportBundle`; `ExportBundle::write_to_zip` /
`read_from_zip` handle the zip format itself.

**Import modes** (`db::export::ImportMode`):
- `Replace` - `INSERT OR REPLACE` on every table, keyed by the ids in
  the file. Correct for "restore this instance" or "move it to a new
  computer": re-importing the same file twice is a no-op, and
  importing a newer export of the same competition overwrites the
  older data as expected.
- `DuplicateAsNew` - before importing, `ExportBundle::remap_ids()`
  walks every entity and every cross-reference (foreign key) and
  assigns fresh UUIDs throughout, using one `HashMap<Uuid, Uuid>` so
  every reference to a given original id resolves to the same new
  id. The result is imported as entirely new data that cannot
  collide with - or silently overwrite - anything already in the
  target instance. Correct for "merge this into an instance that
  already has other, unrelated data."

This isn't yet a true three-way merge (e.g. "these two instances both
edited the same competition since a common export, reconcile the
diff") - `DuplicateAsNew` sidesteps that problem entirely by never
overwriting, at the cost of not detecting "this is actually the same
competition." That's a reasonable trade-off for now; revisit if
someone actually needs true merge semantics.

## Networked live judging (`live_server`)

`crates/live_server` is a separate crate from `server` on purpose:
axum/tokio-networking dependencies don't belong in the orchestration
layer, and keeping it separate means the whole HTTP/WS API is unit
testable (`tower::ServiceExt::oneshot`, or a real socket + a real WS
client - see `crates/live_server/tests/api.rs`) without booting Tauri
at all. `src-tauri/src/main.rs`'s `.setup()` spawns
`live_server::serve(..)` via `tauri::async_runtime::spawn`, reusing the
tokio runtime Tauri already owns rather than starting a second one.

**`AppState` is `Arc<AppService>`, not `Arc<Mutex<AppService>>`.**
Every `AppService` method takes `&self` and the underlying sqlx pool
already handles its own concurrency; the mutex that used to wrap it in
`src-tauri` was an app-wide serialization point imposed by the Tauri
shell, not something `AppService` needed. It had to go before adding
judge traffic that would otherwise queue behind whatever the desktop
control panel happened to be doing.

**Judge identity/auth** didn't exist before this (`Judge` has no
password/token/device concept of its own). Two new generic tables
(judge identity isn't sport-specific):
- `pairing_codes` - a short-lived (10 minute), desktop-generated code
  a judge enters on their device once.
- `judge_devices` - the long-lived credential that code exchanges for.
  Only a SHA-256 hash of the token is ever stored; the plaintext is
  handed back exactly once, at pairing time
  (`AppService::redeem_pairing_code`), the same way a password reset
  link works.

**REST, not WebSocket, for judges submitting scores** - simpler and
retryable on a flaky Wi-Fi connection, and it's the same
`AppService::submit_gymnastics_panel_score` /
`submit_swimming_result` the desktop control panel calls, so scoring
math and ranking recompute can't drift between the two clients. A
score submission over this API also calls
`show_result_on_scoreboard` immediately afterward, matching the
desktop's "Submit score" button doing both in one action rather than
requiring a separate "now show it" step.

**WebSocket is poll-based, not a shared broadcast channel.** Each
`/api/v1/ws/scoreboard/:id` connection independently polls
`AppService::get_scoreboard_state` (same 500ms interval
`frontend/scoreboard.js` already uses) and only sends a message when
`updated_at` changes. This was a deliberate simplification over
threading a `tokio::sync::broadcast::Sender` through `AppService`: it
needs zero changes to `server` or `commands.rs`, and it picks up a
scoreboard change regardless of whether it came from a judge's iPad
(via this crate) or the desktop control panel (which never talks to
`live_server` at all) - there's exactly one source of truth
(`scoreboard_state`), and every client just watches it. The endpoint
is public/unauthenticated, same as `GET /api/v1/scoreboard/:id` - it's
the same thing already shown on the TV, not sensitive.

**LAN-only, by design.** There's no port-forwarding, relay, or public
TLS anywhere in this crate. True internet-wide remote viewing (parents
off the venue's Wi-Fi) would need a hosted relay component - a
materially different, non-local-first architecture from the rest of
this app - and is a deliberately separate, not-yet-decided piece of
future work, not an oversight here.

**Native client: `ios/AbyssalLive`, a separate Xcode project outside
this Cargo workspace.** `crates/live_server` is independently tested
(`crates/live_server/tests/api.rs`); the Swift app consuming it is
not, and was written without access to a Mac/Xcode - see
`ios/README.md` for its actual status and known gaps before assuming
it just works. `docs/LIVE_API.md` is the hand-maintained wire contract
both sides are built against; keep it in sync with any route/JSON
shape change here.

## Adding a new sport (a worked example)

Swimming is the real, completed example of this. To add a sport that
looks roughly like swimming (a single computed number, no need for a
dedicated table):

1. New module: `crates/scoring/src/<sport>/`. Pure functions, own
   `Input`/`Result` structs, own unit tests. Reuse
   `scoring::judging` if a judge panel applies; otherwise it's just a
   calculation.
2. If (and only if) the error cases don't fit the existing
   `ScoringError` variants, add new ones - don't repurpose an
   existing variant with a misleading message (see `NegativeElapsedTime`/
   `NegativePenalty`, added specifically for swimming rather than
   reusing gymnastics' `NegativeDifficulty`).
3. Add the sport to `models::Sport` and to `server::ranking::higher_is_better`.
4. One new method on `AppService`: build the sport's `Input`, call
   `scoring::<sport>::calculate`, write the result into
   `EventResult.detail` as JSON, call `recompute_rankings`. Add a
   dedicated table (like `gymnastics_routines`) only if the sport has
   enough structured data to be worth querying directly - try without
   one first.
5. One new Tauri command in `src-tauri/src/commands.rs`, registered
   in `main.rs`.
6. Frontend: a form, shown/hidden based on the active competition's
   sport (see `updateScoringVisibility()` in `app.js` for the
   pattern), calling the new command.

If a sport needs its own participant-level fields (like gymnastics'
optional `GymnastDetail` - level, club), add a
`models::<sport>::<Thing>Detail` struct keyed by `ParticipantId`,
plus a matching `<sport>_details` table, following
`models::gymnastics::GymnastDetail` /
`db.gymnastics_details` as the template. Most sports probably won't
need this - swimming didn't.

## Known technical debt

- `DuplicateAsNew` import doesn't detect "this is actually a
  re-import of the same competition" - see the export/import section
  above.
- No third sport with a genuinely different data shape has been
  added yet (gymnastics = subjective panel score, swimming = lower
  time wins; something rubric/checklist-based, or something
  distance/height-based, would be a better stress test than a third
  time-based sport).
- Growing automated coverage, but still partial - `scoring` has real
  unit tests, `db` and `live_server` now have integration tests
  against real (in-memory/socket) backends, but the Tauri command
  layer itself is still only covered by manual testing (see
  `WHAT_TO_TEST.md`).
- `DisplayMode::ShowingRanking` exists in the type but nothing in the
  UI sets it yet - standings are computed and stored
  (`recompute_rankings`) but there's no "show the leaderboard on the
  big screen" button.
- Judge device tokens (`judge_devices.token_hash`) never expire on
  their own - a device stays paired until someone revokes it from the
  "Live Judging" tab. Fine for a single meet's judge panel; would want
  an explicit expiry if devices were reused across many
  organizations/events.
- `AppService::create_pairing_code` doesn't retry on a code collision
  (two valid, unexpired 6-digit codes landing on the same value) -
  deliberately, since it's astronomically unlikely with the handful of
  judges pairing at any one meet, but worth revisiting if that
  assumption ever stops holding.
