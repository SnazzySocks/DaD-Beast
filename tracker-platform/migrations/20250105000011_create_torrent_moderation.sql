-- Create torrent_moderation table
-- Moderation status and history for torrents

CREATE TABLE torrent_moderation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    moderator_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Moderation action
    action VARCHAR(50) NOT NULL, -- approved, rejected, edited, deleted, restored, featured
    previous_status VARCHAR(20),
    new_status VARCHAR(20),

    -- Reason and notes
    reason TEXT,
    moderator_notes TEXT, -- Internal notes not visible to users

    -- Changes made (for edit actions)
    changes JSONB, -- JSON object with changed fields

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_torrent_moderation_torrent_id ON torrent_moderation(torrent_id);
CREATE INDEX idx_torrent_moderation_moderator_id ON torrent_moderation(moderator_id);
CREATE INDEX idx_torrent_moderation_action ON torrent_moderation(action);
CREATE INDEX idx_torrent_moderation_created_at ON torrent_moderation(created_at DESC);

-- Composite index for torrent history
CREATE INDEX idx_torrent_moderation_history ON torrent_moderation(torrent_id, created_at DESC);

COMMENT ON TABLE torrent_moderation IS 'Complete moderation history for torrents';
COMMENT ON COLUMN torrent_moderation.action IS 'Type of moderation action performed';
COMMENT ON COLUMN torrent_moderation.changes IS 'JSON object tracking specific field changes for edit actions';
COMMENT ON COLUMN torrent_moderation.moderator_notes IS 'Internal notes visible only to moderators';
