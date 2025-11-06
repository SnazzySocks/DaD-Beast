-- Create torrent_tags table
-- Tags with voting system for torrents

CREATE TABLE torrent_tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,

    -- Tag metadata
    tag_type VARCHAR(50) DEFAULT 'user', -- user, system, genre, quality, etc.
    color VARCHAR(7), -- Hex color
    icon VARCHAR(50),

    -- Statistics
    usage_count INTEGER NOT NULL DEFAULT 0,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    requires_approval BOOLEAN NOT NULL DEFAULT false,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create torrent_tag_assignments table for many-to-many relationship
CREATE TABLE torrent_tag_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES torrent_tags(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Voting system
    upvotes INTEGER NOT NULL DEFAULT 0,
    downvotes INTEGER NOT NULL DEFAULT 0,
    score INTEGER NOT NULL DEFAULT 0, -- upvotes - downvotes

    -- Status
    is_approved BOOLEAN NOT NULL DEFAULT true,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create torrent_tag_votes table for tracking individual votes
CREATE TABLE torrent_tag_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES torrent_tag_assignments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vote_type VARCHAR(10) NOT NULL CHECK (vote_type IN ('upvote', 'downvote')),

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for torrent_tags
CREATE INDEX idx_torrent_tags_slug ON torrent_tags(slug);
CREATE INDEX idx_torrent_tags_tag_type ON torrent_tags(tag_type);
CREATE INDEX idx_torrent_tags_usage_count ON torrent_tags(usage_count DESC);
CREATE INDEX idx_torrent_tags_is_active ON torrent_tags(is_active) WHERE is_active = true;

-- Create indexes for torrent_tag_assignments
CREATE INDEX idx_torrent_tag_assignments_torrent_id ON torrent_tag_assignments(torrent_id);
CREATE INDEX idx_torrent_tag_assignments_tag_id ON torrent_tag_assignments(tag_id);
CREATE INDEX idx_torrent_tag_assignments_user_id ON torrent_tag_assignments(user_id);
CREATE INDEX idx_torrent_tag_assignments_score ON torrent_tag_assignments(score DESC);

-- Unique constraint: prevent duplicate tag assignments
CREATE UNIQUE INDEX idx_torrent_tag_assignments_unique ON torrent_tag_assignments(torrent_id, tag_id);

-- Create indexes for torrent_tag_votes
CREATE INDEX idx_torrent_tag_votes_assignment_id ON torrent_tag_votes(assignment_id);
CREATE INDEX idx_torrent_tag_votes_user_id ON torrent_tag_votes(user_id);

-- Unique constraint: one vote per user per assignment
CREATE UNIQUE INDEX idx_torrent_tag_votes_unique ON torrent_tag_votes(assignment_id, user_id);

COMMENT ON TABLE torrent_tags IS 'Tags for categorizing and labeling torrents';
COMMENT ON TABLE torrent_tag_assignments IS 'Many-to-many relationship between torrents and tags with voting';
COMMENT ON TABLE torrent_tag_votes IS 'Individual user votes on tag assignments';
COMMENT ON COLUMN torrent_tag_assignments.score IS 'Net score calculated as upvotes minus downvotes';
