-- Generic infrastructure tables. Nothing below assumes gymnastics
-- except the tables explicitly prefixed `gymnastics_`.

CREATE TABLE themes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    preset TEXT NOT NULL,
    background_color TEXT NOT NULL,
    primary_color TEXT NOT NULL,
    accent_color TEXT NOT NULL,
    background_image_path TEXT,
    logo_path TEXT,
    font_family TEXT NOT NULL,
    show_grid_overlay INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE competitions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sport TEXT NOT NULL,
    date TEXT NOT NULL,
    location TEXT,
    theme_id TEXT REFERENCES themes(id) ON DELETE SET NULL
);

CREATE TABLE teams (
    id TEXT PRIMARY KEY,
    competition_id TEXT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    abbreviation TEXT,
    primary_color TEXT,
    secondary_color TEXT,
    logo_path TEXT
);

CREATE TABLE participants (
    id TEXT PRIMARY KEY,
    competition_id TEXT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    team_id TEXT REFERENCES teams(id) ON DELETE SET NULL,
    bib_number TEXT,
    level_or_category TEXT
);

CREATE TABLE judges (
    id TEXT PRIMARY KEY,
    competition_id TEXT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    assignment TEXT
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    competition_id TEXT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_time TEXT,
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    competition_id TEXT NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'scheduled'
);

CREATE TABLE results (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    participant_id TEXT REFERENCES participants(id) ON DELETE CASCADE,
    team_id TEXT REFERENCES teams(id) ON DELETE CASCADE,
    final_value REAL,
    place INTEGER,
    finalized INTEGER NOT NULL DEFAULT 0,
    detail TEXT NOT NULL DEFAULT 'null'
);

CREATE TABLE scores (
    id TEXT PRIMARY KEY,
    result_id TEXT NOT NULL REFERENCES results(id) ON DELETE CASCADE,
    judge_id TEXT REFERENCES judges(id) ON DELETE SET NULL,
    value REAL NOT NULL,
    components TEXT NOT NULL DEFAULT 'null',
    submitted_at TEXT NOT NULL
);

CREATE TABLE rankings (
    id TEXT PRIMARY KEY,
    event_id TEXT REFERENCES events(id) ON DELETE CASCADE,
    participant_id TEXT REFERENCES participants(id) ON DELETE CASCADE,
    team_id TEXT REFERENCES teams(id) ON DELETE CASCADE,
    rank INTEGER NOT NULL,
    score REAL NOT NULL
);

-- Singleton-per-competition "what's on the display right now" pointer.
-- This is the only table "Clear scoreboard" writes to.
CREATE TABLE scoreboard_state (
    competition_id TEXT PRIMARY KEY REFERENCES competitions(id) ON DELETE CASCADE,
    current_event_id TEXT REFERENCES events(id) ON DELETE SET NULL,
    current_participant_id TEXT REFERENCES participants(id) ON DELETE SET NULL,
    display_mode TEXT NOT NULL DEFAULT 'idle',
    live_value TEXT NOT NULL DEFAULT 'null',
    updated_at TEXT NOT NULL
);

-- Gymnastics-specific tables. A second sport (e.g. swimming) would add
-- its own `swimming_*` tables here without touching anything above.
CREATE TABLE gymnastics_routines (
    result_id TEXT PRIMARY KEY REFERENCES results(id) ON DELETE CASCADE,
    apparatus TEXT NOT NULL,
    difficulty_score REAL NOT NULL DEFAULT 0,
    execution_start_value REAL NOT NULL DEFAULT 10.0,
    deductions TEXT NOT NULL DEFAULT '[]',
    neutral_deductions REAL NOT NULL DEFAULT 0
);

CREATE TABLE gymnastics_details (
    participant_id TEXT PRIMARY KEY REFERENCES participants(id) ON DELETE CASCADE,
    level TEXT,
    club TEXT
);

CREATE INDEX idx_events_competition ON events(competition_id);
CREATE INDEX idx_participants_competition ON participants(competition_id);
CREATE INDEX idx_results_event ON results(event_id);
CREATE INDEX idx_results_participant ON results(participant_id);
CREATE INDEX idx_scores_result ON scores(result_id);
CREATE INDEX idx_rankings_event ON rankings(event_id);
