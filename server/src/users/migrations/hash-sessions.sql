-- only the sha256 of a session token is stored, like api tokens; the
-- sessions before can't be converted and end
DELETE FROM user_sessions;
ALTER TABLE user_sessions RENAME COLUMN token TO token_hash;
