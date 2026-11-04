CREATE TABLE sites (
    id TEXT PRIMARY KEY,
    -- lowercase, without www., events from www.<domain> count too
    domain TEXT NOT NULL,
    name TEXT NOT NULL,
    created_on TIMESTAMP NOT NULL,
    CONSTRAINT domain_unique UNIQUE (domain)
);

-- the sites a user who is not an admin may see
CREATE TABLE site_users (
    site_id TEXT NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (site_id, user_id)
);

CREATE INDEX idx_site_users_user_id ON site_users(user_id);
