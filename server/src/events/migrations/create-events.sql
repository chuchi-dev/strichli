-- a pageview (key is null) or a custom event, never anything that
-- identifies a visitor: no ip, no user agent, no id
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    site_id TEXT NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    ts TIMESTAMPTZ NOT NULL DEFAULT now(),
    path TEXT NOT NULL,
    -- the host of an external referrer
    referrer TEXT NULL,
    -- the first pageview of a visit: not from the site itself, no reload
    is_unique BOOLEAN NOT NULL,
    key TEXT NULL,
    data JSONB NULL
);

CREATE INDEX idx_events_site_ts ON events (site_id, ts);
CREATE INDEX idx_events_site_key_ts ON events (site_id, key, ts)
    WHERE key IS NOT NULL;
