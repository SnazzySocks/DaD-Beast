-- Create announces table
-- Announce history for tracking and debugging

CREATE TABLE announces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Announce details
    peer_id VARCHAR(40) NOT NULL,
    ip_address INET NOT NULL,
    port INTEGER NOT NULL,

    -- Announce event
    event VARCHAR(20), -- started, completed, stopped, NULL for regular

    -- Transfer stats reported
    uploaded BIGINT NOT NULL DEFAULT 0,
    downloaded BIGINT NOT NULL DEFAULT 0,
    left_bytes BIGINT NOT NULL,

    -- Response
    interval_seconds INTEGER NOT NULL DEFAULT 1800, -- Announce interval given
    min_interval_seconds INTEGER NOT NULL DEFAULT 900,
    seeders_returned INTEGER NOT NULL DEFAULT 0,
    leechers_returned INTEGER NOT NULL DEFAULT 0,

    -- Client info
    user_agent VARCHAR(200),

    -- Tracking
    is_seeder BOOLEAN NOT NULL DEFAULT false,

    -- Timestamp
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Partition by month for time-series data (example for PostgreSQL 10+)
-- This table should be partitioned by created_at for better performance

-- Create indexes
CREATE INDEX idx_announces_torrent_id ON announces(torrent_id);
CREATE INDEX idx_announces_user_id ON announces(user_id);
CREATE INDEX idx_announces_created_at ON announces(created_at DESC);
CREATE INDEX idx_announces_event ON announces(event) WHERE event IS NOT NULL;

-- Composite index for user activity
CREATE INDEX idx_announces_user_created ON announces(user_id, created_at DESC);

COMMENT ON TABLE announces IS 'Historical announce data for tracking and analytics';
COMMENT ON COLUMN announces.event IS 'Announce event: started, completed, stopped, or NULL for regular update';
COMMENT ON COLUMN announces.interval_seconds IS 'Announce interval returned to client';

-- Note: In production, this table should be partitioned by created_at (monthly)
-- and old partitions should be archived or dropped to manage table size
