CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    admin BOOLEAN NOT NULL,
    created_on TIMESTAMP NOT NULL,
    CONSTRAINT email_unique UNIQUE (email)
);

CREATE TABLE user_sessions (
    token TEXT PRIMARY KEY,
    timeout BIGINT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_on TIMESTAMP NOT NULL
);

CREATE INDEX idx_sessions_user_id ON user_sessions(user_id);

CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    -- sha256 of the token in hex, the token itself is only shown once
    token_hash TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_on TIMESTAMP NOT NULL,
    last_used TIMESTAMP NULL,
    CONSTRAINT token_hash_unique UNIQUE (token_hash)
);
