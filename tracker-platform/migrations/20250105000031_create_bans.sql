-- Create bans table
-- User bans and suspensions

CREATE TABLE bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Ban type
    ban_type VARCHAR(50) NOT NULL, -- account, ip, email, upload, download, chat, forum

    -- Ban details
    reason TEXT NOT NULL,
    private_notes TEXT,
    public_reason VARCHAR(500), -- Shown to user

    -- Duration
    is_permanent BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMP WITH TIME ZONE, -- NULL for permanent

    -- Ban scope
    ip_address INET,
    ip_range CIDR, -- For range bans

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Resolution
    lifted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    lifted_at TIMESTAMP WITH TIME ZONE,
    lift_reason TEXT,

    -- Related warnings
    warning_id UUID REFERENCES warnings(id) ON DELETE SET NULL,

    -- Appeal
    appeal_text TEXT,
    appeal_at TIMESTAMP WITH TIME ZONE,
    appeal_reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    appeal_reviewed_at TIMESTAMP WITH TIME ZONE,
    appeal_decision TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_bans_user_id ON bans(user_id);
CREATE INDEX idx_bans_banned_by ON bans(banned_by);
CREATE INDEX idx_bans_ban_type ON bans(ban_type);
CREATE INDEX idx_bans_is_active ON bans(is_active) WHERE is_active = true;
CREATE INDEX idx_bans_is_permanent ON bans(is_permanent) WHERE is_permanent = true;
CREATE INDEX idx_bans_expires_at ON bans(expires_at);
CREATE INDEX idx_bans_ip_address ON bans(ip_address);
CREATE INDEX idx_bans_ip_range ON bans(ip_range) WHERE ip_range IS NOT NULL;
CREATE INDEX idx_bans_created_at ON bans(created_at DESC);

-- Composite index for active bans
CREATE INDEX idx_bans_user_active ON bans(user_id, is_active) WHERE is_active = true;

-- Composite index for IP checking
CREATE INDEX idx_bans_ip_active ON bans(ip_address, is_active) WHERE is_active = true;

-- Index for expiration cleanup
CREATE INDEX idx_bans_expiry_cleanup ON bans(expires_at, is_active)
WHERE expires_at < CURRENT_TIMESTAMP AND is_active = true AND is_permanent = false;

-- Index for appeals
CREATE INDEX idx_bans_appeal ON bans(appeal_at) WHERE appeal_at IS NOT NULL AND appeal_reviewed_at IS NULL;

COMMENT ON TABLE bans IS 'User bans and suspensions';
COMMENT ON COLUMN bans.ban_type IS 'Type of ban: account, ip, email, upload, download, chat, forum';
COMMENT ON COLUMN bans.ip_range IS 'CIDR range for IP range bans';
COMMENT ON COLUMN bans.public_reason IS 'Reason shown to the banned user';
COMMENT ON COLUMN bans.private_notes IS 'Internal notes visible only to moderators';
