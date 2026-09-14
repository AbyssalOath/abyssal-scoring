# Live Scoring API

Hand-maintained reference for the HTTP + WebSocket API `crates/live_server`
exposes, and that `ios/AbyssalLive` (the judge/spectator iPad app) is
built against. There's no shared-schema codegen between Rust and
Swift, so **this file is the contract** - if you change a route or a
JSON shape in `crates/live_server`, update this doc and the Swift
models in the same change.

See also: `ARCHITECTURE.md`'s "Networked live judging" section for
the design rationale (why REST for scoring, why polling instead of a
broadcast channel, why it's LAN-only).

## Base URL

`http://<host>:7878/api/v1` - `<host>` is whatever LAN IP/hostname the
desktop's "Live Judging" tab shows. Plain HTTP, no TLS (LAN-only,
local-first, no certificate to manage). Port is fixed
(`live_server::DEFAULT_PORT`), not configurable today.

## Authentication

Two tiers:
- **Public** - no header needed. Anything a TV scoreboard already
  shows (schedule, current state) or that's needed just to start
  pairing.
- **Device-authenticated** - `Authorization: Bearer <token>`, where
  `<token>` came from `POST /pair`. Invalid/missing/revoked token ->
  `401` with `{"error": "..."}`.

There is no per-request competition-level authorization beyond
this - a paired device can read/write for any competition on this
server, matching a single coach/one-meet-at-a-time deployment.

## Error shape

Any non-2xx response body is `{"error": "<message>"}`. Status codes:
- `400` - bad pairing code (wrong, already used, or expired), or a
  scoring-rule violation (e.g. negative difficulty)
- `401` - missing/invalid/revoked bearer token
- `404` - referenced id not found
- `500` - something broke server-side

## Endpoints

### `GET /health`
Public. Returns the plain text `ok`. Use to confirm the address/port
are reachable before attempting to pair.

### `GET /competitions`
Public. Returns every `Competition` on this server:
```json
[{"id": "uuid", "name": "Fall Invitational", "sport": "gymnastics", "date": "2026-09-09", "location": null, "theme_id": null}]
```
`sport` is one of `"gymnastics" | "swimming" | "track_and_field"`.
This is what a spectator app's competition picker calls; a judge app
doesn't need it (pairing already scopes you to one judge/competition
implicitly via the code).

### `POST /pair`
Public. Body:
```json
{"code": "123456", "deviceName": "Sarah's iPad"}
```
`code` is the 6-digit code the desktop's "Live Judging" tab generated
for a specific judge (10-minute expiry, single use). Response:
```json
{"token": "<40-char alphanumeric>", "judge": {"id": "uuid", "competition_id": "uuid", "name": "Sarah Lee", "role": "execution", "assignment": null}}
```
`role` is one of `"head" | "difficulty" | "execution" | "line" |
"referee" | "timer"`, or `"other:<label>"` for a custom role. Store
`token` securely (Keychain) - it's the only time it's ever returned.
`400` if the code is wrong, already consumed, or expired.

### `GET /competitions/:competition_id/events`
Public - a spectator app needs event names to label the scoreboard
(`current_event_id` alone isn't human-readable), and this is the same
schedule already visible on the venue's rotation sheet. Returns every
`Event` for that competition:
```json
[{"id": "uuid", "competition_id": "uuid", "session_id": null, "name": "Beam", "order_index": 1, "status": "scheduled"}]
```
`status` is one of `"scheduled" | "in_progress" | "completed"`.

### `GET /competitions/:competition_id/participants`
Public, same reasoning as `list_events` - a spectator app needs
participant names, and these are already announced publicly at the
meet. Returns every `Participant`:
```json
[{"id": "uuid", "competition_id": "uuid", "first_name": "Jane", "last_name": "Doe", "team_id": null, "bib_number": null, "level_or_category": "Level 8", "is_attending": true}]
```

### `POST /scores/gymnastics-panel`
Auth required. Body:
```json
{
  "eventId": "uuid",
  "participantId": "uuid",
  "apparatus": "balance_beam",
  "difficultyScore": 5.0,
  "judgeScores": [{"judgeId": null, "deductions": [{"reason": "form", "value": 0.3}]}],
  "neutralDeductions": 0.0
}
```
`apparatus` is one of `"vault" | "uneven_bars" | "balance_beam" |
"floor" | "pommel_horse" | "still_rings" | "parallel_bars" |
"horizontal_bar"`. `judgeScores` is normally exactly one entry for a
single judge submitting from their own device - **`judgeId` can be
omitted/`null`in that case; the server fills it in from the
authenticated device automatically.** Pass it explicitly only for a
multi-judge submission (e.g. a head judge's device relaying a whole
panel's totals).

This computes the score (same math the desktop uses,
`scoring::gymnastics::artistic`), persists it, recomputes rankings,
**and immediately puts it up on the live scoreboard** - there is no
separate "now show it" call. Response is the resulting `EventResult`:
```json
{"id": "uuid", "event_id": "uuid", "participant_id": "uuid", "team_id": null, "final_value": 14.7, "place": null, "finalized": true, "detail": {"difficulty": 5.0, "execution": 9.7, "final_score": 14.7, "judge_execution_scores": [9.7], "neutral_deductions": 0.0}}
```

### `POST /scores/swimming`
Auth required. Body:
```json
{"eventId": "uuid", "participantId": "uuid", "elapsedSeconds": 62.4, "timePenaltySeconds": 0.0}
```
Same "computes + persists + shows on scoreboard immediately" behavior
as the gymnastics endpoint. Response is an `EventResult` with
`detail: {"raw_seconds", "penalty_seconds", "final_seconds"}`.

### `GET /scoreboard/:competition_id`
Public. Returns the current `ScoreboardState`:
```json
{"competition_id": "uuid", "current_event_id": "uuid", "current_participant_id": "uuid", "display_mode": "showing_result", "live_value": {"...": "..."}, "updated_at": "2026-09-09T12:00:00Z"}
```
`display_mode` is one of `"idle" | "in_progress" | "showing_result" |
"showing_ranking"`. Poll this if you'd rather not hold a WebSocket
open; the WS endpoint below is the same data, pushed.

### `GET /ws/scoreboard/:competition_id` (WebSocket upgrade)
Public. On connect, and again every time the state actually changes
(polled server-side every 500ms, matching the desktop scoreboard
display's own refresh rate - not a shared broadcast, so it doesn't
matter whether the change came from a judge's device or the desktop
control panel), sends:
```json
{"type": "scoreboard_state", "payload": { /* same shape as GET /scoreboard/:id */ }}
```
No client-to-server messages are expected; the server just closes the
loop if you disconnect. Reconnect with backoff if it drops.

## What a judge app's flow looks like end to end

1. Ask the coach for the server address (shown on the desktop's "Live
   Judging" tab) and generate a pairing code for you there.
2. `POST /pair` with that code -> store the token.
3. `GET /competitions/:id/events` and `/participants` to build the
   picker - `:id` is `judge.competition_id` from the `/pair` response,
   no separate lookup needed.
4. Judge picks an apparatus/event/participant, enters deductions,
   submits via `POST /scores/...`.
5. Optionally watch `GET /ws/scoreboard/:id` to show a "now on the
   board" banner.

## Out of scope (see ARCHITECTURE.md)

No internet-wide access - this API only ever binds to the local
network. A parent off the venue's Wi-Fi needs a different, not-yet-built
hosted relay; don't design the app to assume one exists.
