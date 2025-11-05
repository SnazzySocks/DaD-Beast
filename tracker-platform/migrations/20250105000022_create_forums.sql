-- Create forums table
-- Forum categories and sections

CREATE TABLE forums (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES forums(id) ON DELETE CASCADE,

    -- Forum details
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,

    -- Display
    position INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 0, -- Hierarchy depth
    icon VARCHAR(50),
    color VARCHAR(7),

    -- Permissions
    min_group_level_read INTEGER NOT NULL DEFAULT 0, -- Minimum group level to read
    min_group_level_post INTEGER NOT NULL DEFAULT 2, -- Minimum group level to post
    min_group_level_topic INTEGER NOT NULL DEFAULT 2, -- Minimum group level to create topics

    -- Settings
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_locked BOOLEAN NOT NULL DEFAULT false, -- No new posts
    auto_lock_topics_days INTEGER, -- Auto-lock topics after X days

    -- Statistics (denormalized)
    topics_count INTEGER NOT NULL DEFAULT 0,
    posts_count INTEGER NOT NULL DEFAULT 0,
    last_post_id UUID, -- Will be set after forum_posts table is created
    last_post_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_forums_parent_id ON forums(parent_id);
CREATE INDEX idx_forums_slug ON forums(slug);
CREATE INDEX idx_forums_position ON forums(position);
CREATE INDEX idx_forums_is_active ON forums(is_active) WHERE is_active = true;
CREATE INDEX idx_forums_last_post_at ON forums(last_post_at DESC);

-- Insert default forums
INSERT INTO forums (name, slug, description, position, min_group_level_read, min_group_level_post) VALUES
('Announcements', 'announcements', 'Official tracker announcements', 1, 0, 20),
('General', 'general', 'General discussion', 2, 0, 2),
('Support', 'support', 'Get help with the tracker', 3, 0, 2),
('Requests', 'requests', 'Request and discuss torrents', 4, 2, 2),
('Off Topic', 'off-topic', 'Anything goes', 5, 2, 2),
('VIP Lounge', 'vip-lounge', 'VIP users only', 6, 6, 6),
('Staff', 'staff', 'Staff discussions', 7, 10, 10);

COMMENT ON TABLE forums IS 'Forum categories and sections';
COMMENT ON COLUMN forums.min_group_level_read IS 'Minimum user group level required to read forum';
COMMENT ON COLUMN forums.min_group_level_post IS 'Minimum user group level required to post in forum';
COMMENT ON COLUMN forums.auto_lock_topics_days IS 'Automatically lock topics after this many days of inactivity';
