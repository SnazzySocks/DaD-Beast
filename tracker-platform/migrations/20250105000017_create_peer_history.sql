-- Create peer_history table
-- Historical peer data for time-series analytics (TimescaleDB recommended)

CREATE TABLE peer_history (
    time TIMESTAMP WITH TIME ZONE NOT NULL,
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Snapshot data
    uploaded BIGINT NOT NULL,
    downloaded BIGINT NOT NULL,
    left_bytes BIGINT NOT NULL,
    is_seeder BOOLEAN NOT NULL,

    -- Peer info
    ip_address INET,
    user_agent VARCHAR(200)
);

-- Create hypertable if using TimescaleDB
-- SELECT create_hypertable('peer_history', 'time', chunk_time_interval => INTERVAL '1 day');

-- Create indexes
CREATE INDEX idx_peer_history_time ON peer_history(time DESC);
CREATE INDEX idx_peer_history_torrent_id ON peer_history(torrent_id, time DESC);
CREATE INDEX idx_peer_history_user_id ON peer_history(user_id, time DESC);

-- Composite index for torrent analytics
CREATE INDEX idx_peer_history_torrent_time ON peer_history(torrent_id, time DESC, is_seeder);

COMMENT ON TABLE peer_history IS 'Time-series peer snapshots for analytics (use TimescaleDB for best performance)';
COMMENT ON COLUMN peer_history.time IS 'Timestamp of the snapshot (use TimescaleDB hypertable)';

-- Note: This table is optimized for TimescaleDB
-- For standard PostgreSQL, consider partitioning by time
-- Old data should be aggregated and archived
