-- Messaging Schema for Creto Enablement Layer
-- Secure E2E encrypted messaging following Signal protocol patterns

-- Agent key bundles
CREATE TABLE IF NOT EXISTS key_bundles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL UNIQUE,
    identity_key_id UUID NOT NULL,
    identity_public_key BYTEA NOT NULL,
    signed_prekey_id INTEGER NOT NULL,
    signed_prekey_public BYTEA NOT NULL,
    signed_prekey_signature BYTEA NOT NULL,
    signed_prekey_timestamp BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_key_bundles_agent ON key_bundles(agent_id);

-- One-time pre-keys
CREATE TABLE IF NOT EXISTS prekeys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    prekey_id INTEGER NOT NULL,
    public_key BYTEA NOT NULL,
    consumed BOOLEAN NOT NULL DEFAULT false,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_id, prekey_id)
);

CREATE INDEX idx_prekeys_agent ON prekeys(agent_id, consumed);

-- Messaging sessions
CREATE TABLE IF NOT EXISTS messaging_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    local_agent_id UUID NOT NULL,
    remote_agent_id UUID NOT NULL,
    state VARCHAR(20) NOT NULL DEFAULT 'establishing',  -- establishing, active, suspended, closed, failed
    -- Ratchet state (encrypted)
    ratchet_state_encrypted BYTEA,
    ratchet_state_nonce BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(local_agent_id, remote_agent_id)
);

CREATE INDEX idx_sessions_local ON messaging_sessions(local_agent_id);
CREATE INDEX idx_sessions_remote ON messaging_sessions(remote_agent_id);
CREATE INDEX idx_sessions_state ON messaging_sessions(state);

-- Message envelopes (for store-and-forward)
CREATE TABLE IF NOT EXISTS message_envelopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID NOT NULL,
    recipient_id UUID NOT NULL,
    envelope_version SMALLINT NOT NULL DEFAULT 1,
    -- Header (clear text for routing)
    content_type VARCHAR(20) NOT NULL,
    reply_to UUID,
    -- Ratchet header
    dh_public BYTEA NOT NULL,
    prev_chain_length INTEGER NOT NULL,
    message_number INTEGER NOT NULL,
    -- Encrypted payload
    ciphertext BYTEA NOT NULL,
    mac BYTEA NOT NULL,
    -- Delivery tracking
    delivered BOOLEAN NOT NULL DEFAULT false,
    delivered_at TIMESTAMPTZ,
    acknowledged BOOLEAN NOT NULL DEFAULT false,
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_envelopes_recipient ON message_envelopes(recipient_id, delivered);
CREATE INDEX idx_envelopes_sender ON message_envelopes(sender_id);
CREATE INDEX idx_envelopes_expires ON message_envelopes(expires_at) WHERE NOT delivered;

-- Delivery receipts
CREATE TABLE IF NOT EXISTS delivery_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    envelope_id UUID NOT NULL REFERENCES message_envelopes(id) ON DELETE CASCADE,
    receipt_type VARCHAR(20) NOT NULL,  -- delivered, read, failed
    signature BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_receipts_envelope ON delivery_receipts(envelope_id);

-- Channel configurations
CREATE TABLE IF NOT EXISTS messaging_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    channel_type VARCHAR(20) NOT NULL,  -- direct, queue, pubsub, store_forward, webhook
    name VARCHAR(100) NOT NULL,
    url VARCHAR(500),
    config JSONB NOT NULL DEFAULT '{}',
    retry_policy JSONB NOT NULL DEFAULT '{"max_attempts": 3, "initial_backoff_ms": 100}',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messaging_channels_org ON messaging_channels(organization_id);

-- Skipped message keys (for out-of-order decryption)
CREATE TABLE IF NOT EXISTS skipped_message_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES messaging_sessions(id) ON DELETE CASCADE,
    dh_public BYTEA NOT NULL,
    message_number INTEGER NOT NULL,
    message_key_encrypted BYTEA NOT NULL,  -- Encrypted with session key
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE(session_id, dh_public, message_number)
);

CREATE INDEX idx_skipped_keys_session ON skipped_message_keys(session_id);
CREATE INDEX idx_skipped_keys_expires ON skipped_message_keys(expires_at);

-- Group messaging (future extension)
CREATE TABLE IF NOT EXISTS messaging_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    creator_agent_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES messaging_groups(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'member',  -- admin, member
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(group_id, agent_id)
);

CREATE INDEX idx_group_members_agent ON group_members(agent_id);
