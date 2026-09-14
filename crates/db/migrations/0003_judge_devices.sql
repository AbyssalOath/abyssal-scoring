-- Judge identity/auth for networked live scoring: a judge pairs an
-- iPad by entering a short-lived code shown on the desktop, which
-- exchanges for a long-lived device token. Generic infrastructure
-- (not gymnastics-specific), so these aren't prefixed like
-- gymnastics_*.

CREATE TABLE pairing_codes (
    code TEXT PRIMARY KEY,
    judge_id TEXT NOT NULL REFERENCES judges(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,
    consumed INTEGER NOT NULL DEFAULT 0
);

-- token_hash, never the plaintext token, is what's persisted -- the
-- plaintext is handed to the device exactly once, at pairing time.
CREATE TABLE judge_devices (
    id TEXT PRIMARY KEY,
    judge_id TEXT NOT NULL REFERENCES judges(id) ON DELETE CASCADE,
    device_name TEXT NOT NULL,
    token_hash TEXT NOT NULL,
    paired_at TEXT NOT NULL,
    last_seen_at TEXT
);

CREATE INDEX idx_judge_devices_judge ON judge_devices(judge_id);
CREATE UNIQUE INDEX idx_judge_devices_token_hash ON judge_devices(token_hash);
