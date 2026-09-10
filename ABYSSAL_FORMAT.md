# The `.abyssal` file format

A `.abyssal` file is a **standard zip archive** (Deflate compression)
containing a versioned snapshot of one Abyssal Scoring instance -
every competition, and everything hanging off each one. This is the
field-by-field reference; for the higher-level "why" (import modes,
when this gets written/read), see `ARCHITECTURE.md`'s "Export /
import" section.

Produced by `ExportBundle::write_to_zip` / read by
`ExportBundle::read_from_zip` in `crates/db/src/export.rs`.

## Archive layout

some-competition-2026-09-10.abyssal (zip)
├── metadata.json
├── data.json
└── assets/
├── team-logo.png
├── background.jpg
└── ...

- **`metadata.json`** - a small header, written on every export.
- **`data.json`** - the actual data (the `ExportBundle`, below).
- **`assets/`** - any logo/background image *files* referenced by an
  exported theme or team, copied in by content so the `.abyssal` file
  is self-contained and portable to a machine that doesn't have those
  original files. **Present only if at least one referenced image
  existed and was readable at export time** - no `assets/` entries at
  all is normal for a competition with no custom branding.

## `metadata.json`

```json
{
  "schema_version": 1,
  "exported_at": "2026-09-10T02:15:00.123456Z"
}
```

- `schema_version` - currently always `1`
  (`db::export::EXPORT_SCHEMA_VERSION`). Bump this constant whenever
  `data.json`'s shape changes in a backward-incompatible way.
- `exported_at` - UTC timestamp of the export, RFC 3339.

**Known gap:** `metadata.json` is written on every export, but
`read_from_zip` does not currently read or validate it - schema
compatibility isn't enforced on import yet. If you're building
tooling against this format, don't assume old exports will be
rejected; check `schema_version` yourself if it matters for your use
case.

## `data.json`

The full `ExportBundle`, one big JSON object with an array per entity
type:

```json
{
  "themes": [ ... ],
  "competitions": [ ... ],
  "teams": [ ... ],
  "participants": [ ... ],
  "judges": [ ... ],
  "sessions": [ ... ],
  "events": [ ... ],
  "results": [ ... ],
  "scores": [ ... ],
  "rankings": [ ... ],
  "gymnastics_routines": [ ... ]
}
```
Every array can be empty (`[]`) - a brand new competition with
nothing added yet still exports validly. Order within each array is
not meaningful; relationships are entirely by id.

### A note on ids

Every id (`CompetitionId`, `EventId`, `ParticipantId`, `TeamId`,
`JudgeId`, `SessionId`, `ResultId`, `ScoreId`, `RankingId`,
`ThemeId`) is a Rust newtype wrapping a UUID, but serializes
**transparently as a plain UUID string** - not as `{"0": "..."}` or
similar:

```json
"id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
```

Any tool reading this format can treat every `*_id` field as a plain
UUID v4 string.

### `themes[]` - `ScoreboardTheme`

```json
{
  "id": "0f5e...",
  "name": "Old School Grid",
  "preset": "old_school_grid",
  "background_color": "#0a0a0a",
  "primary_color": "#ff1a1a",
  "accent_color": "#ffb000",
  "background_image_path": null,
  "logo_path": "/home/donnie/Pictures/team-logo.png",
  "font_family": "\"Seven Segment\", monospace",
  "show_grid_overlay": true
}
```

`preset` is one of: `"old_school_grid"`, `"modern"`, `"minimal"`,
`"custom"` - purely descriptive metadata, doesn't affect rendering by
itself (the actual colors/font/etc. do).

`background_image_path`/`logo_path` are the **original absolute
paths** from the machine that exported the file - see "Assets and
path rewriting" below for what happens to these on import.

### `competitions[]` - `Competition`

```json
{
  "id": "c001...",
  "name": "Fall Invitational",
  "sport": "gymnastics",
  "date": "2026-09-09",
  "location": null,
  "theme_id": "0f5e..."
}
```

`sport` is one of: `"gymnastics"`, `"swimming"`, `"track_and_field"`.
`date` is a plain `YYYY-MM-DD` string (no time component).
`theme_id` is `null` if the competition never had a custom theme set
(the app falls back to the old-school-grid default at display time in
that case - there's no "default theme" entity in the export).

### `teams[]` - `Team`

```json
{
  "id": "t001...",
  "competition_id": "c001...",
  "name": "Riverside Gymnastics",
  "abbreviation": null,
  "primary_color": "#c8102e",
  "secondary_color": "#111111",
  "logo_path": "/home/donnie/Pictures/riverside-logo.png"
}
```

### `participants[]` - `Participant`

```json
{
  "id": "p001...",
  "competition_id": "c001...",
  "first_name": "Jane",
  "last_name": "Doe",
  "team_id": "t001...",
  "bib_number": null,
  "level_or_category": null
}
```

### `judges[]` - `Judge`

```json
{
  "id": "j001...",
  "competition_id": "c001...",
  "name": "Pat Smith",
  "role": "execution",
  "assignment": null
}
```

**`role` encoding - the one genuine gotcha in this format.** The six
fixed roles (`head`, `difficulty`, `execution`, `line`, `referee`,
`timer`) serialize as a plain string, exactly like `sport` and
`preset` above. But `JudgeRole` also has an `Other(String)` variant
(reachable in code, though not currently exposed anywhere in the
UI), and *that* serializes as an object instead of a string:

```json
"role": { "other": "Chief of Course" }
```

So `role` is `string | { "other": string }` - a consumer needs to
handle both shapes. (This is standard serde behavior for a Rust enum
with a mix of unit and data-carrying variants - mentioned here
because it's easy to assume every field in this format is a plain
string given how many of them are.)

Separately, worth knowing if you're ever comparing this file against
the live SQLite database directly: the **database column** for
`role` uses a different encoding entirely (a custom
`"other:<label>"` prefix scheme, not JSON) - the export/import path
always goes through the real `JudgeRole` enum and its normal
`Serialize`/`Deserialize`, so this only matters if you're inspecting
`abyssal.sqlite` by hand.

### `sessions[]` - `Session`

```json
{
  "id": "s001...",
  "competition_id": "c001...",
  "name": "Session 1 - Morning",
  "start_time": "2026-09-09T13:00:00Z",
  "order_index": 0
}
```

`start_time` is `null` or an RFC 3339 UTC timestamp.

### `events[]` - `Event`

```json
{
  "id": "e001...",
  "competition_id": "c001...",
  "session_id": null,
  "name": "Women's Floor Exercise",
  "order_index": 1,
  "status": "completed"
}
```

`status` is one of: `"scheduled"`, `"in_progress"`, `"completed"`.

### `results[]` - `EventResult`

```json
{
  "id": "r001...",
  "event_id": "e001...",
  "participant_id": "p001...",
  "team_id": null,
  "final_value": 14.85,
  "place": null,
  "finalized": true,
  "detail": { }
}
```

`final_value` is the single number used for ranking - points for
gymnastics, seconds for swimming/track (see
`server::ranking::higher_is_better` in `ARCHITECTURE.md` for which
direction wins per sport). `place` is currently always `null` in
practice - standings live in `rankings[]` instead (see below), this
field is reserved but unused by the app today.

`detail` is a free-form JSON object, entirely sport-specific.
Current known shapes:

**Gymnastics:**
```json
{
  "difficulty": 5.0,
  "execution": 9.85,
  "judge_execution_scores": [9.9, 9.8],
  "neutral_deductions": 0.0,
  "final_score": 14.85
}
```
(`judge_execution_scores` is one entry per judge who was checked on
the panel - with zero judges checked, this is a single-element array
from the anonymous fallback scorer.)

**Swimming / track:**
```json
{
  "raw_seconds": 52.31,
  "penalty_seconds": 0.5,
  "final_seconds": 52.81
}
```

A third sport would add a third shape here - nothing parses `detail`
generically, so there's no schema to update elsewhere when that
happens.

### `scores[]` - `Score`

```json
{
  "id": "sc001...",
  "result_id": "r001...",
  "judge_id": "j001...",
  "value": 9.9,
  "components": null,
  "submitted_at": "2026-09-09T14:32:10Z"
}
```

One row per judge's individual number that fed into a result (for
gymnastics: one `Score` per judge on the panel, `value` = that
judge's own execution score; for swimming: exactly one `Score` with
`judge_id: null`, `value` = the final time). `components` is reserved
for a future per-judge breakdown and is currently always `null` -
nothing populates it yet.

### `rankings[]` - `RankingEntry`

```json
{
  "id": "rk001...",
  "event_id": "e001...",
  "participant_id": "p001...",
  "team_id": null,
  "rank": 1,
  "score": 14.85
}
```

Recomputed after every score submission (`AppService::recompute_rankings`)
This is derived data, not a source of truth. Standard competition
ranking: ties share a rank, the next distinct value skips ahead by
the number of tied entries (1, 2, 2, 4).

### `gymnastics_routines[]` - `Routine`

```json
{
  "result_id": "r001...",
  "apparatus": "floor",
  "difficulty_score": 5.0,
  "execution_start_value": 10.0,
  "deductions": [
    { "reason": "panel deduction total", "value": 0.1 },
    { "reason": "panel deduction total", "value": 0.2 }
  ],
  "neutral_deductions": 0.0
}
```

Only present for gymnastics results - swimming/track results have no
corresponding entry anywhere in this file (see `ARCHITECTURE.md` for
why swimming doesn't get its own table at all). Keyed by
`result_id`, not its own id.

`apparatus` is one of: `"vault"`, `"uneven_bars"`, `"balance_beam"`,
`"floor"`, `"pommel_horse"`, `"still_rings"`, `"parallel_bars"`,
`"horizontal_bar"`.

`deductions` is the **flattened combination of every judge's**
deduction list on the panel - the per-judge split lives in
`scores[]` (each judge's resulting execution score), not here; this
array just records every individual deduction line that was entered,
regardless of which judge entered it.

## Assets and path rewriting

`background_image_path`/`logo_path` fields in `themes[]`/`teams[]`
hold the **original absolute path** from the exporting machine (e.g.
`/home/donnie/Pictures/logo.png`) - these are almost certainly wrong
on a different computer, which is exactly what `assets/` is for.

On export, every referenced path that pointed to a real, readable
file gets copied into the zip under `assets/<basename>` - just the
filename, not the full path. On import, those files get extracted
into an `assets/` folder next to the target instance's database
(`db::export::default_asset_dir`), but **the path fields in
`data.json` are not currently rewritten to point at the new
location** - the importing app resolves this at *display* time
instead (the frontend's `convertFileSrc` / Tauri's asset protocol),
not by mutating the JSON.

**Known limitation:** because assets are keyed only by filename, two
different images that happen to share a filename (e.g. two teams
both using a file literally named `logo.png`) will collide in the
zip - the second one silently overwrites the first in `assets/`.
Fine for a single team's branding; worth knowing if you're exporting
a competition with many teams that all used generically-named image
files.

## Minimal complete example

A competition with one team, one participant, one judge, one event,
and one finalized gymnastics result - everything else in this file
would be `[]`:

```json
{
  "themes": [],
  "competitions": [
    { "id": "c1", "name": "Test Meet", "sport": "gymnastics", "date": "2026-09-10", "location": null, "theme_id": null }
  ],
  "teams": [
    { "id": "t1", "competition_id": "c1", "name": "Riverside", "abbreviation": null, "primary_color": "#c8102e", "secondary_color": null, "logo_path": null }
  ],
  "participants": [
    { "id": "p1", "competition_id": "c1", "first_name": "Jane", "last_name": "Doe", "team_id": "t1", "bib_number": null, "level_or_category": null }
  ],
  "judges": [
    { "id": "j1", "competition_id": "c1", "name": "Pat Smith", "role": "execution", "assignment": null }
  ],
  "sessions": [],
  "events": [
    { "id": "e1", "competition_id": "c1", "session_id": null, "name": "Floor", "order_index": 1, "status": "completed" }
  ],
  "results": [
    { "id": "r1", "event_id": "e1", "participant_id": "p1", "team_id": null, "final_value": 14.85, "place": null, "finalized": true,
      "detail": { "difficulty": 5.0, "execution": 9.85, "judge_execution_scores": [9.85], "neutral_deductions": 0.0, "final_score": 14.85 } }
  ],
  "scores": [
    { "id": "sc1", "result_id": "r1", "judge_id": "j1", "value": 9.85, "components": null, "submitted_at": "2026-09-10T02:00:00Z" }
  ],
  "rankings": [
    { "id": "rk1", "event_id": "e1", "participant_id": "p1", "team_id": null, "rank": 1, "score": 14.85 }
  ],
  "gymnastics_routines": [
    { "result_id": "r1", "apparatus": "floor", "difficulty_score": 5.0, "execution_start_value": 10.0,
      "deductions": [{ "reason": "panel deduction total", "value": 0.15 }], "neutral_deductions": 0.0 }
  ]
}
```

## Versioning this format

If you change any `#[derive(Serialize, Deserialize)]` struct in
`crates/db/src/export.rs`'s dependency chain (i.e. any type
reachable from `ExportBundle`) in a way that isn't purely additive:

1. Bump `EXPORT_SCHEMA_VERSION` in `crates/db/src/export.rs`.
2. Update this document.
3. Consider whether old exports should still import - right now
   nothing checks `schema_version` on the way in (see the "Known
   gap" note above), so a breaking change here will currently fail
   with a generic JSON deserialization error rather than a clear
   "this file is from an older/newer version" message. Improving
   that is a reasonable follow-up if the format ever actually
   changes.
