-- Track which recipients have received sender keys for each distribution_id
-- This is needed for Signal's group messaging protocol where SenderKeyDistributionMessage
-- must be sent to recipients before they can decrypt group messages.
CREATE TABLE IF NOT EXISTS sender_key_shared (
  distribution_id TEXT NOT NULL,
  address TEXT NOT NULL,
  device_id INTEGER NOT NULL,
  identity TEXT NOT NULL CHECK (identity IN ('aci', 'pni')),
  PRIMARY KEY (distribution_id, address, device_id, identity)
);

CREATE INDEX idx_sender_key_shared_distribution ON sender_key_shared(distribution_id);
CREATE INDEX idx_sender_key_shared_address ON sender_key_shared(address, device_id, identity);