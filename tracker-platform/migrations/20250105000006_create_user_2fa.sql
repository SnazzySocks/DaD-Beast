-- Create user_2fa table
-- Two-factor authentication secrets and recovery codes

CREATE TABLE user_2fa (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- TOTP settings
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    secret VARCHAR(32) NOT NULL, -- Base32 encoded secret
    backup_codes TEXT[], -- Array of hashed backup codes

    -- Recovery
    recovery_email VARCHAR(255),

    -- Usage tracking
    last_used_at TIMESTAMP WITH TIME ZONE,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    enabled_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_2fa_is_enabled ON user_2fa(is_enabled) WHERE is_enabled = true;
CREATE INDEX idx_user_2fa_last_used_at ON user_2fa(last_used_at DESC);

COMMENT ON TABLE user_2fa IS 'Two-factor authentication secrets and recovery codes';
COMMENT ON COLUMN user_2fa.secret IS 'Base32 encoded TOTP secret for authenticator apps';
COMMENT ON COLUMN user_2fa.backup_codes IS 'Array of hashed one-time backup codes';
COMMENT ON COLUMN user_2fa.failed_attempts IS 'Count of failed 2FA attempts for rate limiting';
