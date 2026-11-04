-- one-time links to set a password, for new users and resets
CREATE TABLE password_links (
    -- sha256 of the token in hex
    token_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    timeout BIGINT NOT NULL
);

CREATE INDEX idx_password_links_user_id ON password_links(user_id);
