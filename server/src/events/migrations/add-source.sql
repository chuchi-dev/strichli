-- where an imported event comes from (e.g. simpleanalytics), null for
-- the ones strichli counted itself; a new import replaces the old one
ALTER TABLE events ADD COLUMN source TEXT NULL;

CREATE INDEX idx_events_site_source ON events (site_id, source)
    WHERE source IS NOT NULL;
