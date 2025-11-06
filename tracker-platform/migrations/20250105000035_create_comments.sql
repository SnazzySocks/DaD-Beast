-- Create comments table
-- Comments on torrents, requests, and other entities

CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Commentable entity (polymorphic)
    commentable_type VARCHAR(50) NOT NULL, -- torrent, request, collection, etc.
    commentable_id UUID NOT NULL,

    -- Author
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Comment content
    body TEXT NOT NULL,
    body_html TEXT, -- Rendered HTML

    -- Threading support
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,

    -- Editing
    edit_count INTEGER NOT NULL DEFAULT 0,
    last_edited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    last_edited_at TIMESTAMP WITH TIME ZONE,

    -- Moderation
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,

    -- Author info
    author_ip INET,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_comments_commentable ON comments(commentable_type, commentable_id);
CREATE INDEX idx_comments_author_id ON comments(author_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);
CREATE INDEX idx_comments_created_at ON comments(created_at DESC);
CREATE INDEX idx_comments_is_deleted ON comments(is_deleted) WHERE is_deleted = true;

-- Composite index for entity comments
CREATE INDEX idx_comments_entity_created ON comments(commentable_type, commentable_id, created_at);

-- Composite index for user's comments
CREATE INDEX idx_comments_author_created ON comments(author_id, created_at DESC);

-- Full-text search
CREATE INDEX idx_comments_search ON comments USING gin(to_tsvector('english', body));

COMMENT ON TABLE comments IS 'Comments on torrents, requests, collections, and other entities';
COMMENT ON COLUMN comments.commentable_type IS 'Type of entity being commented on (torrent, request, etc.)';
COMMENT ON COLUMN comments.commentable_id IS 'UUID of the commented entity';
COMMENT ON COLUMN comments.parent_id IS 'Parent comment ID for threaded discussions';
