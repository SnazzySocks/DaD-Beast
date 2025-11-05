-- Create torrent_requests table
-- User requests for torrents with bounty system

CREATE TABLE torrent_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES torrent_categories(id) ON DELETE RESTRICT,

    -- Request details
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,

    -- Media information (similar to torrents)
    media_type VARCHAR(50),
    release_year INTEGER,

    -- External IDs for matching
    tmdb_id INTEGER,
    imdb_id VARCHAR(20),
    igdb_id INTEGER,
    musicbrainz_id UUID,

    -- Quality requirements
    required_resolution VARCHAR(20), -- Minimum resolution
    required_source VARCHAR(50), -- Preferred source
    required_codec VARCHAR(50),

    -- Bounty system
    initial_bounty DECIMAL(20,2) NOT NULL DEFAULT 0.00,
    total_bounty DECIMAL(20,2) NOT NULL DEFAULT 0.00,
    bounty_contributors_count INTEGER NOT NULL DEFAULT 1,

    -- Request status
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, filled, rejected, expired
    filled_by UUID REFERENCES users(id) ON DELETE SET NULL,
    filled_torrent_id UUID REFERENCES torrents(id) ON DELETE SET NULL,
    filled_at TIMESTAMP WITH TIME ZONE,

    -- Voting system
    votes_count INTEGER NOT NULL DEFAULT 0,

    -- Statistics
    views_count INTEGER NOT NULL DEFAULT 0,
    comments_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE -- Auto-expire old requests
);

-- Create torrent_request_bounties table for tracking contributors
CREATE TABLE torrent_request_bounties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES torrent_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount DECIMAL(20,2) NOT NULL,

    -- Status
    is_paid BOOLEAN NOT NULL DEFAULT false,
    paid_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create torrent_request_votes table
CREATE TABLE torrent_request_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES torrent_requests(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for torrent_requests
CREATE INDEX idx_torrent_requests_requester_id ON torrent_requests(requester_id);
CREATE INDEX idx_torrent_requests_category_id ON torrent_requests(category_id);
CREATE INDEX idx_torrent_requests_status ON torrent_requests(status);
CREATE INDEX idx_torrent_requests_filled_by ON torrent_requests(filled_by);
CREATE INDEX idx_torrent_requests_filled_torrent_id ON torrent_requests(filled_torrent_id);
CREATE INDEX idx_torrent_requests_created_at ON torrent_requests(created_at DESC);
CREATE INDEX idx_torrent_requests_total_bounty ON torrent_requests(total_bounty DESC);
CREATE INDEX idx_torrent_requests_votes_count ON torrent_requests(votes_count DESC);
CREATE INDEX idx_torrent_requests_expires_at ON torrent_requests(expires_at) WHERE status = 'pending';

-- Composite index for active requests
CREATE INDEX idx_torrent_requests_active ON torrent_requests(status, total_bounty DESC) WHERE status = 'pending';

-- Create indexes for torrent_request_bounties
CREATE INDEX idx_torrent_request_bounties_request_id ON torrent_request_bounties(request_id);
CREATE INDEX idx_torrent_request_bounties_user_id ON torrent_request_bounties(user_id);
CREATE INDEX idx_torrent_request_bounties_is_paid ON torrent_request_bounties(is_paid) WHERE is_paid = false;

-- Create indexes for torrent_request_votes
CREATE INDEX idx_torrent_request_votes_request_id ON torrent_request_votes(request_id);
CREATE INDEX idx_torrent_request_votes_user_id ON torrent_request_votes(user_id);

-- Unique constraint: one vote per user per request
CREATE UNIQUE INDEX idx_torrent_request_votes_unique ON torrent_request_votes(request_id, user_id);

COMMENT ON TABLE torrent_requests IS 'User requests for torrents with bounty system';
COMMENT ON TABLE torrent_request_bounties IS 'Individual bounty contributions to requests';
COMMENT ON TABLE torrent_request_votes IS 'User votes on torrent requests';
COMMENT ON COLUMN torrent_requests.total_bounty IS 'Total bounty pool from all contributors';
COMMENT ON COLUMN torrent_requests.expires_at IS 'Expiration date for automatic cleanup of old requests';
