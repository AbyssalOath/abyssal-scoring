-- Roster import support: a generic "attending this meet" flag on
-- participants, plus the TAAF-specific identifiers/status fields on
-- gymnastics_details (which up to now had a table but no code using
-- it). See ARCHITECTURE.md's generic/sport-specific split -- an
-- attendance flag is useful for any sport's roster, so it lives on
-- `participants`; the rest is TAAF/gymnastics-specific.

ALTER TABLE participants ADD COLUMN is_attending INTEGER NOT NULL DEFAULT 1;

-- Denormalized from participants.competition_id so the uniqueness
-- constraint below (which must be scoped per-competition) can live on
-- this table -- SQLite unique indexes can't reach across a join.
ALTER TABLE gymnastics_details ADD COLUMN competition_id TEXT REFERENCES competitions(id) ON DELETE CASCADE;
ALTER TABLE gymnastics_details ADD COLUMN taaf_athlete_number TEXT;
ALTER TABLE gymnastics_details ADD COLUMN taaf_registration_number TEXT;
ALTER TABLE gymnastics_details ADD COLUMN qual_status TEXT;
-- The roster's "A&T" column -- meaning unconfirmed; stored verbatim.
ALTER TABLE gymnastics_details ADD COLUMN at_code TEXT;

-- Upsert key for weekly roster re-imports: importing the same TAAF
-- athlete number for the same competition again updates that row
-- instead of creating a duplicate. Partial index so multiple athletes
-- with no TAAF number yet don't collide with each other.
CREATE UNIQUE INDEX idx_gymnastics_details_taaf_number
    ON gymnastics_details(competition_id, taaf_athlete_number)
    WHERE taaf_athlete_number IS NOT NULL;
