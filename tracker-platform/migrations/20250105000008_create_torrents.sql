-- Create torrents table
-- Main torrent table with info_hash, name, size, and metadata

CREATE TABLE torrents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Core torrent data
    info_hash CHAR(40) NOT NULL UNIQUE, -- SHA1 hash (hex encoded)
    info_hash_v2 VARCHAR(64) UNIQUE, -- SHA256 hash for BitTorrent v2
    name VARCHAR(500) NOT NULL,
    description TEXT,

    -- Torrent file details
    size_bytes BIGINT NOT NULL,
    file_count INTEGER NOT NULL DEFAULT 1,
    piece_length BIGINT NOT NULL,
    pieces_count INTEGER NOT NULL,

    -- Organization
    category_id INTEGER NOT NULL REFERENCES torrent_categories(id) ON DELETE RESTRICT,
    uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Media information
    media_type VARCHAR(50), -- movie, tv_episode, tv_season, album, game, etc.
    release_year INTEGER,

    -- Quality and format
    resolution VARCHAR(20), -- 720p, 1080p, 2160p, etc.
    source VARCHAR(50), -- BluRay, WEB-DL, HDTV, etc.
    codec_video VARCHAR(50), -- H.264, H.265, AV1, etc.
    codec_audio VARCHAR(50), -- AAC, DTS, TrueHD, FLAC, etc.
    container VARCHAR(20), -- mkv, mp4, avi, etc.

    -- Scene/release information
    release_group VARCHAR(100),
    is_scene BOOLEAN NOT NULL DEFAULT false,
    is_internal BOOLEAN NOT NULL DEFAULT false, -- Internal release

    -- Status flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_freeleech BOOLEAN NOT NULL DEFAULT false,
    is_double_upload BOOLEAN NOT NULL DEFAULT false,
    is_featured BOOLEAN NOT NULL DEFAULT false,
    is_sticky BOOLEAN NOT NULL DEFAULT false,
    is_anonymous BOOLEAN NOT NULL DEFAULT false, -- Hide uploader

    -- Moderation
    moderation_status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, approved, rejected
    moderated_by UUID REFERENCES users(id) ON DELETE SET NULL,
    moderated_at TIMESTAMP WITH TIME ZONE,
    rejection_reason TEXT,

    -- Statistics (denormalized for performance)
    seeders INTEGER NOT NULL DEFAULT 0,
    leechers INTEGER NOT NULL DEFAULT 0,
    times_completed INTEGER NOT NULL DEFAULT 0,
    views INTEGER NOT NULL DEFAULT 0,

    -- Engagement
    comments_count INTEGER NOT NULL DEFAULT 0,
    thanks_count INTEGER NOT NULL DEFAULT 0,

    -- NFO file
    nfo_text TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE, -- Soft delete
    last_seeder_at TIMESTAMP WITH TIME ZONE -- Last time a seeder was active
);

-- Create indexes
CREATE INDEX idx_torrents_info_hash ON torrents(info_hash);
CREATE INDEX idx_torrents_category_id ON torrents(category_id);
CREATE INDEX idx_torrents_uploader_id ON torrents(uploader_id);
CREATE INDEX idx_torrents_name ON torrents(name);
CREATE INDEX idx_torrents_created_at ON torrents(created_at DESC);
CREATE INDEX idx_torrents_seeders ON torrents(seeders DESC);
CREATE INDEX idx_torrents_times_completed ON torrents(times_completed DESC);
CREATE INDEX idx_torrents_is_active ON torrents(is_active) WHERE is_active = true;
CREATE INDEX idx_torrents_is_freeleech ON torrents(is_freeleech) WHERE is_freeleech = true;
CREATE INDEX idx_torrents_is_featured ON torrents(is_featured) WHERE is_featured = true;
CREATE INDEX idx_torrents_moderation_status ON torrents(moderation_status) WHERE moderation_status = 'pending';
CREATE INDEX idx_torrents_deleted_at ON torrents(deleted_at) WHERE deleted_at IS NOT NULL;

-- Composite indexes for common queries
CREATE INDEX idx_torrents_category_created ON torrents(category_id, created_at DESC);
CREATE INDEX idx_torrents_category_seeders ON torrents(category_id, seeders DESC);
CREATE INDEX idx_torrents_active_category ON torrents(category_id, is_active) WHERE is_active = true;

-- Full-text search index on name and description
CREATE INDEX idx_torrents_search ON torrents USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')));

COMMENT ON TABLE torrents IS 'Main torrent table with metadata and statistics';
COMMENT ON COLUMN torrents.info_hash IS '40-character SHA1 hash of torrent info dictionary';
COMMENT ON COLUMN torrents.info_hash_v2 IS 'SHA256 hash for BitTorrent v2 protocol';
COMMENT ON COLUMN torrents.is_internal IS 'Internal release from site release groups';
COMMENT ON COLUMN torrents.moderation_status IS 'Moderation state: pending, approved, rejected';
COMMENT ON COLUMN torrents.last_seeder_at IS 'Used to identify dead torrents for cleanup';
