-- Create torrent_statistics table
-- Aggregated statistics for torrents

CREATE TABLE torrent_statistics (
    torrent_id UUID PRIMARY KEY REFERENCES torrents(id) ON DELETE CASCADE,

    -- Current state
    seeders INTEGER NOT NULL DEFAULT 0,
    leechers INTEGER NOT NULL DEFAULT 0,
    times_completed INTEGER NOT NULL DEFAULT 0,

    -- Historical peaks
    peak_seeders INTEGER NOT NULL DEFAULT 0,
    peak_leechers INTEGER NOT NULL DEFAULT 0,
    peak_seeders_at TIMESTAMP WITH TIME ZONE,
    peak_leechers_at TIMESTAMP WITH TIME ZONE,

    -- Transfer totals (across all users)
    total_uploaded BIGINT NOT NULL DEFAULT 0,
    total_downloaded BIGINT NOT NULL DEFAULT 0,

    -- Unique statistics
    unique_seeders_count INTEGER NOT NULL DEFAULT 0, -- Total unique seeders ever
    unique_leechers_count INTEGER NOT NULL DEFAULT 0, -- Total unique leechers ever

    -- Speed statistics (in bytes per second)
    average_upload_speed BIGINT NOT NULL DEFAULT 0,
    average_download_speed BIGINT NOT NULL DEFAULT 0,

    -- Health metrics
    seed_time_total BIGINT NOT NULL DEFAULT 0, -- Total seed time in seconds
    seed_time_average BIGINT NOT NULL DEFAULT 0, -- Average seed time per seeder

    -- Last activity
    last_seeder_at TIMESTAMP WITH TIME ZONE,
    last_leecher_at TIMESTAMP WITH TIME ZONE,
    last_completed_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_torrent_statistics_seeders ON torrent_statistics(seeders DESC);
CREATE INDEX idx_torrent_statistics_leechers ON torrent_statistics(leechers DESC);
CREATE INDEX idx_torrent_statistics_times_completed ON torrent_statistics(times_completed DESC);
CREATE INDEX idx_torrent_statistics_last_seeder_at ON torrent_statistics(last_seeder_at DESC);
CREATE INDEX idx_torrent_statistics_updated_at ON torrent_statistics(updated_at DESC);

-- Index for dead torrent detection
CREATE INDEX idx_torrent_statistics_dead ON torrent_statistics(last_seeder_at)
WHERE seeders = 0 AND last_seeder_at < CURRENT_TIMESTAMP - INTERVAL '30 days';

COMMENT ON TABLE torrent_statistics IS 'Aggregated statistics and metrics for torrents';
COMMENT ON COLUMN torrent_statistics.times_completed IS 'Number of times torrent has been fully downloaded';
COMMENT ON COLUMN torrent_statistics.unique_seeders_count IS 'Total unique users who have seeded this torrent';
COMMENT ON COLUMN torrent_statistics.seed_time_average IS 'Average seeding duration per user in seconds';
