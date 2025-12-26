-- Add checkpoints table for durable oversight request state
-- This supports crash recovery and resumption of requests after restart

CREATE TABLE IF NOT EXISTS checkpoints (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES oversight_requests(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    checkpoint_data JSONB NOT NULL,
    version INTEGER NOT NULL,
    reason TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient lookup by request
CREATE INDEX idx_checkpoints_request_id ON checkpoints(request_id);

-- Index for efficient lookup of latest checkpoint
CREATE INDEX idx_checkpoints_request_timestamp ON checkpoints(request_id, timestamp DESC);

-- Index for cleanup queries
CREATE INDEX idx_checkpoints_timestamp ON checkpoints(timestamp);

-- Comments for documentation
COMMENT ON TABLE checkpoints IS 'Persistent checkpoints for oversight request state recovery';
COMMENT ON COLUMN checkpoints.checkpoint_data IS 'Full serialized checkpoint including state machine and context';
COMMENT ON COLUMN checkpoints.version IS 'Schema version for migration compatibility';
COMMENT ON COLUMN checkpoints.reason IS 'Optional reason for checkpoint (e.g., before_approval, periodic)';
