# Contributing to Abyssal Scoring

Thanks for taking a look. This is a young project, so the fastest way
to help right now is picking something off the TODO list in the
README rather than proposing large new features.

## Setup

You'll need:
- A current stable Rust toolchain (`rustup` - Tauri needs a recent
  compiler).
- The Tauri CLI: `cargo install tauri-cli --version "^2"`
- Platform prerequisites for Tauri v2 - see
  https://v2.tauri.app/start/prerequisites/. On Linux this means
  `webkit2gtk-4.1` (not the old 4.0 package, which most distros have
  dropped), plus gtk3/appindicator/librsvg dev headers.

## Running it

```bash
cargo tauri dev
```

opens the control panel. Use the Scoreboard & Theme tab to open the
actual scoreboard display window (drag it to a second monitor for a
real setup).

## Before you open a PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

CI runs all three (plus a full build) on Linux, macOS, and Windows -
please make sure they pass locally first.

## Project layout / the one architectural rule that matters

crates/models/ generic domain types (Competition, Event,
Participant, ...) + sport-specific submodules
crates/scoring/ scoring algorithms, one module per sport, no shared
"universal" trait
crates/db/ SQLite persistence + the .abyssal export/import
format
crates/server/ AppService: orchestrates db + scoring, is what the
UI actually calls
src-tauri/ the Tauri app shell (windows, IPC commands)
frontend/ plain HTML/CSS/JS control panel + scoreboard display

The rule everything above follows: **the generic infrastructure
(competitions, events, participants, results) must not know anything
sport-specific; only each sport's own module in `models`/`scoring`
does.** If a change to `db` or `server` needs a `match sport { ... }`
outside of dispatching to a sport module, that's usually a sign the
change belongs somewhere else. This is what makes adding a second
sport (swimming, track) a contained change instead of a rewrite -
please keep it that way.

## Commit / PR conventions

- Keep PRs scoped to one thing - a bugfix, one new feature, one
  refactor. Easier to review, easier to revert if something's wrong.
- Reference the TODO item or issue you're addressing in the PR
  description.
- No hard requirement on commit message format yet, but a clear
  one-line summary is appreciated.

## Reporting bugs

Open an issue with: what you did, what you expected, what happened
instead, and your OS + `cargo tauri info` output if it's a build or
runtime issue rather than a logic bug.
