-- Create chat_rooms table
-- Chat room definitions for real-time chat

CREATE TABLE chat_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Room details
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    topic VARCHAR(300), -- Current topic

    -- Room type
    room_type VARCHAR(50) NOT NULL DEFAULT 'public', -- public, private, staff, etc.

    -- Permissions
    min_group_level INTEGER NOT NULL DEFAULT 0,
    allowed_user_ids UUID[], -- For private rooms
    banned_user_ids UUID[], -- Users banned from room
    moderator_user_ids UUID[], -- Room moderators

    -- Settings
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_public BOOLEAN NOT NULL DEFAULT true,
    max_users INTEGER, -- Max concurrent users
    message_retention_days INTEGER NOT NULL DEFAULT 30,
    slow_mode_seconds INTEGER, -- Minimum seconds between messages

    -- Statistics
    current_users_count INTEGER NOT NULL DEFAULT 0,
    total_messages_count INTEGER NOT NULL DEFAULT 0,

    -- Display
    icon VARCHAR(50),
    color VARCHAR(7),
    position INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_chat_rooms_slug ON chat_rooms(slug);
CREATE INDEX idx_chat_rooms_room_type ON chat_rooms(room_type);
CREATE INDEX idx_chat_rooms_is_active ON chat_rooms(is_active) WHERE is_active = true;
CREATE INDEX idx_chat_rooms_is_public ON chat_rooms(is_public) WHERE is_public = true;
CREATE INDEX idx_chat_rooms_position ON chat_rooms(position);

-- GIN indexes for arrays
CREATE INDEX idx_chat_rooms_allowed_users ON chat_rooms USING gin(allowed_user_ids);
CREATE INDEX idx_chat_rooms_banned_users ON chat_rooms USING gin(banned_user_ids);
CREATE INDEX idx_chat_rooms_moderators ON chat_rooms USING gin(moderator_user_ids);

-- Insert default chat rooms
INSERT INTO chat_rooms (name, slug, description, room_type, min_group_level, position) VALUES
('General', 'general', 'General chat', 'public', 0, 1),
('Support', 'support', 'Get help and support', 'public', 0, 2),
('Trading', 'trading', 'Trade invites and discuss releases', 'public', 2, 3),
('VIP', 'vip', 'VIP members only', 'private', 6, 4),
('Staff', 'staff', 'Staff chat', 'staff', 10, 5);

COMMENT ON TABLE chat_rooms IS 'Chat room definitions for real-time messaging';
COMMENT ON COLUMN chat_rooms.allowed_user_ids IS 'Array of user IDs allowed in private rooms';
COMMENT ON COLUMN chat_rooms.banned_user_ids IS 'Array of user IDs banned from room';
COMMENT ON COLUMN chat_rooms.slow_mode_seconds IS 'Minimum seconds between messages per user';
