-- Create user_statistics table
-- Tracks upload/download stats, ratio, and bonus points

CREATE TABLE user_statistics (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- Transfer statistics (in bytes)
    uploaded BIGINT NOT NULL DEFAULT 0,
    downloaded BIGINT NOT NULL DEFAULT 0,
    raw_uploaded BIGINT NOT NULL DEFAULT 0, -- Without freeleech
    raw_downloaded BIGINT NOT NULL DEFAULT 0, -- Without freeleech

    -- Ratio calculation
    -- Ratio = uploaded / downloaded (handle division by zero)
    -- Can be computed as: CASE WHEN downloaded > 0 THEN uploaded::DECIMAL / downloaded ELSE 0 END

    -- Buffer for ratio protection (in bytes)
    buffer_bytes BIGINT NOT NULL DEFAULT 0,

    -- Torrent counts
    torrents_uploaded INTEGER NOT NULL DEFAULT 0,
    torrents_active INTEGER NOT NULL DEFAULT 0, -- Currently seeding/leeching
    torrents_seeding INTEGER NOT NULL DEFAULT 0,
    torrents_leeching INTEGER NOT NULL DEFAULT 0,
    torrents_snatched INTEGER NOT NULL DEFAULT 0, -- Completed downloads

    -- Bonus point system
    seedbonus DECIMAL(20,2) NOT NULL DEFAULT 0.00,
    seedbonus_earned DECIMAL(20,2) NOT NULL DEFAULT 0.00, -- Total ever earned
    seedbonus_spent DECIMAL(20,2) NOT NULL DEFAULT 0.00, -- Total ever spent

    -- Request system
    requests_created INTEGER NOT NULL DEFAULT 0,
    requests_filled INTEGER NOT NULL DEFAULT 0,
    requests_voted INTEGER NOT NULL DEFAULT 0,
    bounty_spent DECIMAL(20,2) NOT NULL DEFAULT 0.00,
    bounty_earned DECIMAL(20,2) NOT NULL DEFAULT 0.00,

    -- Freeleech tokens
    freeleech_tokens INTEGER NOT NULL DEFAULT 0,
    freeleech_tokens_used INTEGER NOT NULL DEFAULT 0,

    -- Community engagement
    forum_posts INTEGER NOT NULL DEFAULT 0,
    comments_posted INTEGER NOT NOT NULL DEFAULT 0,
    thanks_given INTEGER NOT NULL DEFAULT 0,
    thanks_received INTEGER NOT NULL DEFAULT 0,

    -- Invite statistics
    invites_sent INTEGER NOT NULL DEFAULT 0,
    successful_invites INTEGER NOT NULL DEFAULT 0, -- Invitees who are still active

    -- Time-based statistics
    total_seedtime BIGINT NOT NULL DEFAULT 0, -- In seconds
    total_leechtime BIGINT NOT NULL DEFAULT 0, -- In seconds
    average_seedtime BIGINT NOT NULL DEFAULT 0, -- Average per torrent

    -- Milestones
    first_upload_at TIMESTAMP WITH TIME ZONE,
    first_download_at TIMESTAMP WITH TIME ZONE,
    largest_upload_bytes BIGINT NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_statistics_uploaded ON user_statistics(uploaded DESC);
CREATE INDEX idx_user_statistics_downloaded ON user_statistics(downloaded DESC);
CREATE INDEX idx_user_statistics_seedbonus ON user_statistics(seedbonus DESC);
CREATE INDEX idx_user_statistics_torrents_uploaded ON user_statistics(torrents_uploaded DESC);
CREATE INDEX idx_user_statistics_updated_at ON user_statistics(updated_at DESC);

-- Index for ratio calculation (users with low ratio)
CREATE INDEX idx_user_statistics_ratio ON user_statistics((uploaded::DECIMAL / NULLIF(downloaded, 0)));

COMMENT ON TABLE user_statistics IS 'User upload/download statistics and bonus points';
COMMENT ON COLUMN user_statistics.uploaded IS 'Total bytes uploaded (with freeleech adjustments)';
COMMENT ON COLUMN user_statistics.raw_uploaded IS 'Actual bytes uploaded without freeleech';
COMMENT ON COLUMN user_statistics.buffer_bytes IS 'Additional upload buffer for ratio protection';
COMMENT ON COLUMN user_statistics.seedbonus IS 'Current available bonus points';
COMMENT ON COLUMN user_statistics.total_seedtime IS 'Total time spent seeding in seconds';
