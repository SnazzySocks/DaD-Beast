-- Create forum_topics table
-- Discussion threads in forums

CREATE TABLE forum_topics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forum_id UUID NOT NULL REFERENCES forums(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Topic details
    title VARCHAR(300) NOT NULL,
    slug VARCHAR(300) NOT NULL,

    -- Status flags
    is_locked BOOLEAN NOT NULL DEFAULT false,
    is_sticky BOOLEAN NOT NULL DEFAULT false,
    is_poll BOOLEAN NOT NULL DEFAULT false,

    -- Statistics (denormalized)
    posts_count INTEGER NOT NULL DEFAULT 0,
    views_count INTEGER NOT NULL DEFAULT 0,
    subscribers_count INTEGER NOT NULL DEFAULT 0,

    -- Last activity
    last_post_id UUID, -- Will be set after forum_posts table is created
    last_post_at TIMESTAMP WITH TIME ZONE,
    last_post_user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_forum_topics_forum_id ON forum_topics(forum_id);
CREATE INDEX idx_forum_topics_author_id ON forum_topics(author_id);
CREATE INDEX idx_forum_topics_slug ON forum_topics(slug);
CREATE INDEX idx_forum_topics_is_sticky ON forum_topics(is_sticky) WHERE is_sticky = true;
CREATE INDEX idx_forum_topics_last_post_at ON forum_topics(last_post_at DESC);
CREATE INDEX idx_forum_topics_created_at ON forum_topics(created_at DESC);

-- Composite index for forum listing
CREATE INDEX idx_forum_topics_forum_sticky_updated ON forum_topics(forum_id, is_sticky DESC, last_post_at DESC);

-- Unique slug per forum
CREATE UNIQUE INDEX idx_forum_topics_unique_slug ON forum_topics(forum_id, slug);

COMMENT ON TABLE forum_topics IS 'Discussion topics/threads in forums';
COMMENT ON COLUMN forum_topics.is_sticky IS 'Pinned to top of forum';
COMMENT ON COLUMN forum_topics.is_poll IS 'Topic includes a poll';
