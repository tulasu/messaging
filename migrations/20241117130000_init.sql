CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS messenger_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    messenger TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS messenger_tokens_user_messenger_idx
    ON messenger_tokens (user_id, messenger);

CREATE TABLE IF NOT EXISTS message_history (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    messenger TEXT NOT NULL,
    recipient TEXT NOT NULL,
    body TEXT NOT NULL,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL,
    status_reason TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    requested_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS message_history_user_idx
    ON message_history (user_id, created_at DESC);

