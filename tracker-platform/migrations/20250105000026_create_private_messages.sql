-- Create private_messages table
-- Private messaging system

CREATE TABLE private_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Participants
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Message content
    subject VARCHAR(300) NOT NULL,
    body TEXT NOT NULL,
    body_html TEXT, -- Rendered HTML

    -- Thread management
    thread_id UUID, -- For grouping related messages
    parent_id UUID REFERENCES private_messages(id) ON DELETE SET NULL,

    -- Status flags
    is_read BOOLEAN NOT NULL DEFAULT false,
    read_at TIMESTAMP WITH TIME ZONE,

    -- Deletion (each user can delete independently)
    deleted_by_sender BOOLEAN NOT NULL DEFAULT false,
    deleted_by_recipient BOOLEAN NOT NULL DEFAULT false,

    -- Sender information
    sender_ip INET,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_private_messages_sender_id ON private_messages(sender_id);
CREATE INDEX idx_private_messages_recipient_id ON private_messages(recipient_id);
CREATE INDEX idx_private_messages_thread_id ON private_messages(thread_id);
CREATE INDEX idx_private_messages_parent_id ON private_messages(parent_id);
CREATE INDEX idx_private_messages_is_read ON private_messages(is_read) WHERE is_read = false;
CREATE INDEX idx_private_messages_created_at ON private_messages(created_at DESC);

-- Composite index for inbox
CREATE INDEX idx_private_messages_inbox ON private_messages(recipient_id, created_at DESC)
WHERE deleted_by_recipient = false;

-- Composite index for sent messages
CREATE INDEX idx_private_messages_sent ON private_messages(sender_id, created_at DESC)
WHERE deleted_by_sender = false;

-- Composite index for unread messages
CREATE INDEX idx_private_messages_unread ON private_messages(recipient_id, is_read, created_at DESC)
WHERE is_read = false AND deleted_by_recipient = false;

-- Full-text search
CREATE INDEX idx_private_messages_search ON private_messages
USING gin(to_tsvector('english', subject || ' ' || body));

COMMENT ON TABLE private_messages IS 'Private messaging system between users';
COMMENT ON COLUMN private_messages.thread_id IS 'UUID grouping related messages in a conversation';
COMMENT ON COLUMN private_messages.deleted_by_sender IS 'Message deleted from sender''s sent folder';
COMMENT ON COLUMN private_messages.deleted_by_recipient IS 'Message deleted from recipient''s inbox';
