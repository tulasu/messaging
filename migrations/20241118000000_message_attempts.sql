CREATE TABLE IF NOT EXISTS message_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES message_history (id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    status_reason TEXT,
    requested_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS message_attempts_message_id_idx
    ON message_attempts (message_id, created_at DESC);

CREATE INDEX IF NOT EXISTS message_attempts_created_at_idx
    ON message_attempts (created_at DESC);

