-- Create chat_messages table
-- Real-time chat messages

CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES chat_rooms(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Message content
    message TEXT NOT NULL,
    message_html TEXT, -- Rendered HTML with emojis, mentions, etc.

    -- Message type
    message_type VARCHAR(50) NOT NULL DEFAULT 'message', -- message, system, action, etc.

    -- Mentions and references
    mentioned_user_ids UUID[], -- Users mentioned in message

    -- Moderation
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,

    -- User info snapshot
    user_ip INET,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_chat_messages_room_id ON chat_messages(room_id);
CREATE INDEX idx_chat_messages_user_id ON chat_messages(user_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at DESC);
CREATE INDEX idx_chat_messages_is_deleted ON chat_messages(is_deleted) WHERE is_deleted = true;

-- Composite index for room messages
CREATE INDEX idx_chat_messages_room_created ON chat_messages(room_id, created_at DESC);

-- GIN index for mentions
CREATE INDEX idx_chat_messages_mentions ON chat_messages USING gin(mentioned_user_ids);

-- Index for cleanup of old messages
CREATE INDEX idx_chat_messages_cleanup ON chat_messages(room_id, created_at)
WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '30 days';

COMMENT ON TABLE chat_messages IS 'Real-time chat messages in chat rooms';
COMMENT ON COLUMN chat_messages.message_type IS 'Type: message, system, action (e.g., user joined)';
COMMENT ON COLUMN chat_messages.mentioned_user_ids IS 'Array of user IDs mentioned with @ in message';

-- Note: Old messages should be cleaned up based on chat_rooms.message_retention_days
