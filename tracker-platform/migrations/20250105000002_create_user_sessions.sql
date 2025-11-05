-- Create user_sessions table
-- Tracks active user sessions with JWT tokens

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE, -- SHA256 hash of JWT token

    -- Session metadata
    ip_address INET NOT NULL,
    user_agent TEXT,
    device_type VARCHAR(50), -- mobile, desktop, tablet
    device_name VARCHAR(100),
    browser VARCHAR(100),
    platform VARCHAR(100),

    -- Geolocation
    country_code CHAR(2),
    city VARCHAR(100),

    -- Session management
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_user_sessions_is_active ON user_sessions(is_active) WHERE is_active = true;
CREATE INDEX idx_user_sessions_last_activity ON user_sessions(last_activity_at DESC);

-- Composite index for active user sessions
CREATE INDEX idx_user_sessions_user_active ON user_sessions(user_id, is_active) WHERE is_active = true;

COMMENT ON TABLE user_sessions IS 'Active user sessions for authentication and security';
COMMENT ON COLUMN user_sessions.token_hash IS 'SHA256 hash of the JWT token for validation';
COMMENT ON COLUMN user_sessions.last_activity_at IS 'Updated on each request to track session activity';
