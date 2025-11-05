-- Create peers table
-- Active peer tracking for seeders and leechers

CREATE TABLE peers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Peer identification
    peer_id VARCHAR(40) NOT NULL, -- Client peer ID
    ip_address INET NOT NULL,
    port INTEGER NOT NULL,

    -- Client information
    user_agent VARCHAR(200), -- BitTorrent client
    client_name VARCHAR(100), -- Parsed client name
    client_version VARCHAR(50), -- Parsed client version

    -- Transfer state
    uploaded BIGINT NOT NULL DEFAULT 0, -- Bytes uploaded this session
    downloaded BIGINT NOT NULL DEFAULT 0, -- Bytes downloaded this session
    left_bytes BIGINT NOT NULL, -- Bytes remaining to download

    -- Peer type
    is_seeder BOOLEAN NOT NULL DEFAULT false,
    is_partial_seed BOOLEAN NOT NULL DEFAULT false, -- Has complete files but not all

    -- Connection details
    crypto_level VARCHAR(20), -- none, supported, required
    is_connectable BOOLEAN NOT NULL DEFAULT true,

    -- Activity tracking
    announces_count INTEGER NOT NULL DEFAULT 1,
    last_announce_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Freeleech tracking
    is_freeleech BOOLEAN NOT NULL DEFAULT false,
    is_token_freeleech BOOLEAN NOT NULL DEFAULT false,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_peers_torrent_id ON peers(torrent_id);
CREATE INDEX idx_peers_user_id ON peers(user_id);
CREATE INDEX idx_peers_is_seeder ON peers(is_seeder);
CREATE INDEX idx_peers_last_announce_at ON peers(last_announce_at);
CREATE INDEX idx_peers_created_at ON peers(created_at);

-- Composite indexes for common queries
CREATE INDEX idx_peers_torrent_seeder ON peers(torrent_id, is_seeder);
CREATE INDEX idx_peers_user_torrent ON peers(user_id, torrent_id);

-- Unique constraint: one active peer per user per torrent per IP
CREATE UNIQUE INDEX idx_peers_unique ON peers(torrent_id, user_id, ip_address, peer_id);

-- Index for cleanup of stale peers
CREATE INDEX idx_peers_stale ON peers(last_announce_at) WHERE last_announce_at < CURRENT_TIMESTAMP - INTERVAL '1 hour';

COMMENT ON TABLE peers IS 'Active peers currently seeding or leeching torrents';
COMMENT ON COLUMN peers.peer_id IS 'Client-generated peer ID from announce';
COMMENT ON COLUMN peers.left_bytes IS 'Bytes remaining to complete download (0 for seeders)';
COMMENT ON COLUMN peers.is_partial_seed IS 'Has some complete files but not entire torrent';
COMMENT ON COLUMN peers.is_token_freeleech IS 'Using a freeleech token for this download';
