-- Create warnings table
-- User warnings and infractions system

CREATE TABLE warnings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    issued_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Warning details
    warning_type VARCHAR(50) NOT NULL, -- ratio, hit_and_run, inactivity, conduct, spam, etc.
    severity VARCHAR(20) NOT NULL DEFAULT 'minor', -- minor, moderate, severe, critical

    -- Reason
    reason TEXT NOT NULL,
    private_notes TEXT, -- Internal moderator notes

    -- Points system
    points INTEGER NOT NULL DEFAULT 1, -- Warning points (accumulate toward ban)

    -- Linked entities
    related_torrent_id UUID REFERENCES torrents(id) ON DELETE SET NULL,
    related_report_id UUID REFERENCES reports(id) ON DELETE SET NULL,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    acknowledged BOOLEAN NOT NULL DEFAULT false,
    acknowledged_at TIMESTAMP WITH TIME ZONE,

    -- Expiration
    expires_at TIMESTAMP WITH TIME ZONE, -- When warning points expire

    -- Resolution
    revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_by UUID REFERENCES users(id) ON DELETE SET NULL,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoke_reason TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_warnings_user_id ON warnings(user_id);
CREATE INDEX idx_warnings_issued_by ON warnings(issued_by);
CREATE INDEX idx_warnings_warning_type ON warnings(warning_type);
CREATE INDEX idx_warnings_severity ON warnings(severity);
CREATE INDEX idx_warnings_is_active ON warnings(is_active) WHERE is_active = true;
CREATE INDEX idx_warnings_acknowledged ON warnings(acknowledged) WHERE acknowledged = false;
CREATE INDEX idx_warnings_expires_at ON warnings(expires_at);
CREATE INDEX idx_warnings_created_at ON warnings(created_at DESC);

-- Composite index for active warnings
CREATE INDEX idx_warnings_user_active ON warnings(user_id, is_active, created_at DESC)
WHERE is_active = true AND revoked = false;

-- Index for expiration cleanup
CREATE INDEX idx_warnings_expiry_cleanup ON warnings(expires_at, is_active)
WHERE expires_at < CURRENT_TIMESTAMP AND is_active = true;

COMMENT ON TABLE warnings IS 'User warnings and infractions system';
COMMENT ON COLUMN warnings.points IS 'Warning points that accumulate toward automatic actions';
COMMENT ON COLUMN warnings.severity IS 'Severity level: minor, moderate, severe, critical';
COMMENT ON COLUMN warnings.expires_at IS 'When warning points expire and become inactive';
COMMENT ON COLUMN warnings.private_notes IS 'Internal notes visible only to moderators';
