CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payload_type VARCHAR(50) NOT NULL,
    payload_data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE message_destinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id),
    messenger_type VARCHAR(20) NOT NULL,
    chat_id VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    retry_count INTEGER DEFAULT 0,
    last_attempt TIMESTAMP WITH TIME ZONE,
    sent_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_message_destinations_status ON message_destinations(status);
CREATE INDEX idx_message_destinations_messenger_type ON message_destinations(messenger_type);
CREATE INDEX idx_message_destinations_retry ON message_destinations(retry_count, status);
CREATE INDEX idx_message_destinations_message_id ON message_destinations(message_id);