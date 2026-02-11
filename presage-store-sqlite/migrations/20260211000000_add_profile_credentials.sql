-- Profile key credentials for GV2 group operations
-- These are ExpiringProfileKeyCredential serialized via zkgroup::serialize
-- Used to create ProfileKeyCredentialPresentations when creating/modifying groups
CREATE TABLE IF NOT EXISTS profile_credentials (
    uuid TEXT PRIMARY KEY NOT NULL,
    credential BLOB NOT NULL,
    expiration_time INTEGER NOT NULL  -- Unix timestamp in seconds
);

-- Index for cleanup of expired credentials
CREATE INDEX IF NOT EXISTS idx_profile_credentials_expiration 
ON profile_credentials(expiration_time);
