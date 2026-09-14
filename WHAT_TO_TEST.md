# Pre-release manual test checklist

This app doesn't have integration tests yet - everything below has
only been exercised by hand, on Linux (Arch). Before tagging a
release, work through this list, especially the Windows/macOS
sections, which haven't been touched at all yet.

Use a **throwaway app-data directory** for this pass if you can
(rename/move your real `abyssal.sqlite` first) so test data doesn't
mix with anything real. See the README for where that lives per OS.

## Fresh install / first run

- [ ] App launches with no existing database and doesn't error
- [ ] Control panel window renders correctly (not blank - this was a
      real bug on Linux/NVIDIA; see README's "Linux note")
- [ ] Window/taskbar icon shows correctly (not the placeholder
      solid-color square, if you've since run `cargo tauri icon`)

## Competitions

- [ ] Create a **gymnastics** competition - appears in the list,
      becomes selectable
- [ ] Create a **swimming** competition
- [ ] Create a **track & field** competition
- [ ] Creating with an empty name is rejected with a visible message
- [ ] Creating with a malformed date (not `YYYY-MM-DD`) is rejected
- [ ] Switching between competitions updates the Live Scoring tab's
      form correctly (gymnastics fields for gymnastics, time fields
      for swimming/track)
- [ ] Quit and relaunch the app - the previously active competition
      is restored automatically, roster/schedule reload correctly

## Roster

- [ ] Add a team, set primary/secondary color via the color pickers
      - swatches visibly update immediately (this was broken under
      WebKitGTK; confirm the fix holds)
- [ ] Add a team logo via "Choose image…" - native file dialog opens,
      filtered to image types, filename shows after picking
- [ ] Add a participant (first + last name)
- [ ] Adding a participant with an empty name is rejected
- [ ] Add several judges with different roles
- [ ] Adding a judge with an empty name is rejected

## Roster import (TAAF roster sheet)

- [ ] Import a real TAAF roster `.xlsx` - the banner row ("email
      inquiries to...") is skipped, headers are recognized and
      auto-mapped correctly (First/Last name, A&T, Gym, TAAF Athlete
      Competition #, Registration #, Division, Qual Status)
- [ ] Import the same `.csv` export of that roster - same result
- [ ] Import a `.pdf` version of the roster - a note about the parse
      being approximate is shown, and every row is manually verified
      before confirming (this format is inherently lossy - don't trust
      it blind)
- [ ] Manually remap a column via its dropdown and edit a cell value
      before confirming - the edited values, not the auto-detected
      ones, are what gets imported
- [ ] Uncheck "Attending" on a row, confirm the import summary and
      that the rotation sheet excludes that athlete afterward
- [ ] Re-import the same file unchanged - summary shows `0 created,
      N updated` (no duplicate participants created)
- [ ] Re-import an updated roster (some athletes removed/added,
      others' division changed) - existing athletes update in place by
      TAAF athlete #, new ones are created, removed-from-roster
      athletes are left alone (still attending unless unchecked by
      hand)
- [ ] A row missing a TAAF athlete # still imports, with a warning
      that a future re-import can't match it automatically
- [ ] Confirm `gymnastics_details` (TAAF #s, division, qual status)
      round-trips through a full `.abyssal` export/import
- [x] "A&T" column meaning confirmed with the coach: usually blank,
      nobody's sure what it stands for - stored verbatim, no further
      action needed unless that changes
- [ ] Import a ProScore-exported athlete list ("Athlete #" header
      instead of "TAAF Athlete Competition #") - still auto-maps to
      the TAAF athlete number column

## Rotation sheet

- [ ] With attendance/roster data imported, open Rotation Sheet, group
      by Division - athletes appear grouped and sorted by last name
- [ ] Switch grouping to Gym - regroups correctly
- [ ] Non-attending athletes never appear
- [ ] Print (or print-preview) - only the rotation sheet content shows,
      not the nav/header/other panels

## Schedule

- [ ] Add multiple events with different order values
- [ ] Events appear in the Live Scoring event dropdown

## Live scoring - gymnastics

- [ ] Put a participant "up" on the scoreboard (no score yet) -
      confirm the scoreboard display window shows their name/event
      with no score
- [ ] Submit a score with **zero** judges checked (fallback
      deduction-total field) - final score appears on the scoreboard
      display
- [ ] Submit a score with **one** judge checked
- [ ] Submit a score with **three or more** judges checked, using
      clearly different deduction totals - confirm the high and low
      are dropped (check `EventResult.detail.judge_execution_scores`
      if you want to verify the math, or just sanity-check the final
      number)
- [ ] Try a negative difficulty / negative deduction total - rejected
      client-side with an alert, never reaches the backend
- [ ] Standings/rankings update after each submission (higher score
      = better rank for gymnastics)

## Live scoring - swimming/track

- [ ] Submit an elapsed time with no penalty
- [ ] Submit an elapsed time with a penalty added
- [ ] Try a negative elapsed time or negative penalty - rejected
      client-side
- [ ] Rankings favor the **lower** time (opposite direction from
      gymnastics - confirm this is actually true, not just assumed)

## Scoreboard display

- [ ] Open the display window (non-fullscreen) - separate window from
      control panel
- [ ] Open fullscreen - confirm it actually fills the screen /
      second monitor if you have one
- [ ] Default theme ("Old School Grid") shows black background, red
      glowing text, grid overlay
- [ ] Switch to "Modern" preset, confirm the display updates
- [ ] Set a custom background image and/or logo, confirm both render
      on the display window
- [ ] "Clear scoreboard for next event" - display returns to idle;
      confirm the roster/schedule/results are **unchanged** afterward
      (this is the whole point of that button - verify it, don't just
      trust it)

## Live judging (networked, backend/API only - no iPad app yet)

- [x] Automated: `cargo test -p live_server` covers pairing, auth
      rejection/acceptance, score submission + scoreboard push, and a
      real WebSocket connection receiving pushes on state change.
- [x] Manual, via `cargo run -p live_server --example smoke_server` +
      curl/websocat against a real socket: health check, pair, 401
      without a token, 401 with a bad token, authenticated event
      listing, gymnastics score submission (single-judge auto-fill of
      `judgeId` from the authenticated device), scoreboard read.
- [ ] From the real desktop app: open "Live Judging", confirm the LAN
      address shown is actually reachable from another device on the
      same network (not just `localhost`)
- [ ] Generate a pairing code for a judge, redeem it from a second
      machine/device via curl on the same network within the 10-minute
      window - confirm it works, and that trying the same code again
      fails
- [ ] Let a pairing code sit unused past 10 minutes - confirm it's
      rejected as expired
- [ ] Revoke a paired device from "Live Judging" - confirm its token
      is rejected on the next request
- [ ] Submit a score via the live API while the desktop's scoreboard
      display window is open - confirm it updates (currently via the
      display's existing 500ms poll, not yet wired to this API's push)
- [ ] First run: confirm the OS firewall prompt (Windows Defender /
      macOS Application Firewall) appears and allowing it lets other
      devices actually connect
- [ ] Restart the app - previously paired devices still authenticate
      (tokens are persisted in SQLite, not held only in memory)

## Export / import

- [ ] Export a competition with a decent amount of data (a few
      participants, judges, a couple of scores) - native save dialog
      opens, filename is pre-filled as `<competition>-<date>.abyssal`,
      filtered to `.abyssal`
- [ ] Import that file back in **Replace** mode into the same
      instance - no duplicate rows, data matches
- [ ] Import that file in **Import as new copies** mode - a second,
      independent copy of the competition appears (different id),
      original is untouched
- [ ] Export from one machine/install, import into a genuinely
      separate install (a different app-data directory, or a second
      machine if you have one) - this is the actual "move to another
      computer" use case, worth testing for real at least once
- [ ] If a theme had a logo/background image, confirm the image
      travels with the export and re-renders correctly after import

## Cross-platform (the big untested gap)

### Windows
- [ ] `cargo tauri dev` runs at all (WebView2 present/prompts to
      install if missing)
- [ ] Native file dialogs (export/import/image pickers) look and
      behave correctly
- [ ] Date text field, color pickers, all form inputs behave the same
      as Linux
- [ ] Fullscreen scoreboard window actually fullscreens correctly
- [ ] App-data directory resolves somewhere sensible
      (`%APPDATA%`-ish) and persists across relaunches

### macOS
- [ ] `cargo tauri dev` runs (Xcode command line tools present)
- [ ] Native file dialogs look correct (macOS's are visually distinct
      from GTK's/Windows')
- [ ] Fullscreen behaves correctly (macOS fullscreen has its own
      window-management quirks vs. Windows/Linux)
- [ ] App-data directory resolves correctly and persists

## Regression watch-list (things that broke once already)

- [ ] Scoreboard actually updates on score submission (not just "sets
      current participant" with no number)
- [ ] Date field doesn't get stuck / require Tab to escape
- [ ] Color swatches aren't blank after picking a color
- [ ] Window isn't blank on launch (Linux GPU issue)
- [ ] `cargo clippy --workspace` and `cargo fmt --all -- --check` are
      both clean before every push
