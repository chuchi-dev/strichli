-- a token reads the stats, one that can write also adds and changes
-- sites. The earlier tokens keep what their user could do.
ALTER TABLE api_tokens ADD COLUMN can_write BOOLEAN NOT NULL DEFAULT FALSE;
UPDATE api_tokens SET can_write = users.admin
FROM users WHERE users.id = api_tokens.user_id;
ALTER TABLE api_tokens ALTER COLUMN can_write DROP DEFAULT;
