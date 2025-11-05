-- Create freeleech_tokens table
-- Freeleech token management and usage tracking

CREATE TABLE freeleech_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,

    -- Token details
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, expired, completed, cancelled

    -- Usage tracking
    downloaded_bytes BIGINT NOT NULL DEFAULT 0, -- Bytes downloaded with token active
    uploaded_bytes BIGINT NOT NULL DEFAULT 0, -- Still counts toward ratio

    -- Time tracking
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL, -- Token expiry (e.g., 48 hours)
    completed_at TIMESTAMP WITH TIME ZONE, -- When download completed

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_freeleech_tokens_user_id ON freeleech_tokens(user_id);
CREATE INDEX idx_freeleech_tokens_torrent_id ON freeleech_tokens(torrent_id);
CREATE INDEX idx_freeleech_tokens_status ON freeleech_tokens(status);
CREATE INDEX idx_freeleech_tokens_expires_at ON freeleech_tokens(expires_at);
CREATE INDEX idx_freeleech_tokens_created_at ON freeleech_tokens(created_at DESC);

-- Unique constraint: one active token per user per torrent
CREATE UNIQUE INDEX idx_freeleech_tokens_unique ON freeleech_tokens(user_id, torrent_id)
WHERE status = 'active';

-- Composite index for active tokens
CREATE INDEX idx_freeleech_tokens_active ON freeleech_tokens(user_id, status)
WHERE status = 'active';

-- Index for expiry cleanup
CREATE INDEX idx_freeleech_tokens_expiry_cleanup ON freeleech_tokens(expires_at, status)
WHERE status = 'active' AND expires_at < CURRENT_TIMESTAMP;

COMMENT ON TABLE freeleech_tokens IS 'Freeleech token usage and tracking';
COMMENT ON COLUMN freeleech_tokens.status IS 'Token status: active, expired, completed, cancelled';
COMMENT ON COLUMN freeleech_tokens.downloaded_bytes IS 'Bytes downloaded while token was active (not counted against ratio)';
COMMENT ON COLUMN freeleech_tokens.expires_at IS 'Token expiration timestamp (typically 24-72 hours from activation)';
