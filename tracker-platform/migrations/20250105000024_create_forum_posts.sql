-- Create forum_posts table
-- Individual posts in forum topics

CREATE TABLE forum_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic_id UUID NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Post content
    body TEXT NOT NULL,
    body_html TEXT, -- Rendered HTML version

    -- Editing
    edit_count INTEGER NOT NULL DEFAULT 0,
    last_edited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    last_edited_at TIMESTAMP WITH TIME ZONE,
    edit_reason VARCHAR(500),

    -- Moderation
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    deletion_reason TEXT,

    -- Author information snapshot
    author_ip INET, -- For moderation

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_forum_posts_topic_id ON forum_posts(topic_id);
CREATE INDEX idx_forum_posts_author_id ON forum_posts(author_id);
CREATE INDEX idx_forum_posts_created_at ON forum_posts(created_at DESC);
CREATE INDEX idx_forum_posts_is_deleted ON forum_posts(is_deleted) WHERE is_deleted = true;

-- Composite index for topic pagination
CREATE INDEX idx_forum_posts_topic_created ON forum_posts(topic_id, created_at);

-- Index for user's posts
CREATE INDEX idx_forum_posts_author_created ON forum_posts(author_id, created_at DESC);

-- Full-text search index
CREATE INDEX idx_forum_posts_search ON forum_posts USING gin(to_tsvector('english', body));

COMMENT ON TABLE forum_posts IS 'Individual posts/replies in forum topics';
COMMENT ON COLUMN forum_posts.body_html IS 'Pre-rendered HTML for performance';
COMMENT ON COLUMN forum_posts.edit_count IS 'Number of times post has been edited';
COMMENT ON COLUMN forum_posts.author_ip IS 'IP address for moderation purposes';

-- Now update forums table to add foreign key
ALTER TABLE forums ADD CONSTRAINT fk_forums_last_post
FOREIGN KEY (last_post_id) REFERENCES forum_posts(id) ON DELETE SET NULL;

-- Update forum_topics table to add foreign key
ALTER TABLE forum_topics ADD CONSTRAINT fk_forum_topics_last_post
FOREIGN KEY (last_post_id) REFERENCES forum_posts(id) ON DELETE SET NULL;
