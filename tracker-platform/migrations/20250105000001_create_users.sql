-- Create users table
-- Core user accounts for the tracker platform

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL, -- Argon2id hash
    passkey CHAR(32) NOT NULL UNIQUE, -- Unique key for tracker announces
    group_id INTEGER NOT NULL REFERENCES user_groups(id) ON DELETE RESTRICT,

    -- Profile information
    avatar_url VARCHAR(500),
    title VARCHAR(100), -- Custom title
    signature TEXT,
    bio TEXT,
    location VARCHAR(100),
    timezone VARCHAR(50) DEFAULT 'UTC',

    -- Status flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_banned BOOLEAN NOT NULL DEFAULT false,
    is_donor BOOLEAN NOT NULL DEFAULT false,
    is_warned BOOLEAN NOT NULL DEFAULT false,

    -- Security
    email_verification_token VARCHAR(64),
    email_verified_at TIMESTAMP WITH TIME ZONE,
    password_reset_token VARCHAR(64),
    password_reset_expires_at TIMESTAMP WITH TIME ZONE,
    last_password_change TIMESTAMP WITH TIME ZONE,

    -- Activity tracking
    last_login_at TIMESTAMP WITH TIME ZONE,
    last_login_ip INET,
    last_access_at TIMESTAMP WITH TIME ZONE, -- Last activity
    login_count INTEGER NOT NULL DEFAULT 0,

    -- Invite system
    invited_by UUID REFERENCES users(id) ON DELETE SET NULL,
    invites_remaining INTEGER NOT NULL DEFAULT 0,

    -- Settings
    locale VARCHAR(10) DEFAULT 'en',
    items_per_page INTEGER NOT NULL DEFAULT 50,
    theme VARCHAR(50) DEFAULT 'default',

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE -- Soft delete
);

-- Create indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_passkey ON users(passkey);
CREATE INDEX idx_users_group_id ON users(group_id);
CREATE INDEX idx_users_invited_by ON users(invited_by);
CREATE INDEX idx_users_is_active ON users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_is_banned ON users(is_banned) WHERE is_banned = true;
CREATE INDEX idx_users_created_at ON users(created_at DESC);
CREATE INDEX idx_users_last_access_at ON users(last_access_at DESC);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NOT NULL;

-- Partial index for active users only
CREATE INDEX idx_users_active_username ON users(username) WHERE is_active = true AND deleted_at IS NULL;

COMMENT ON TABLE users IS 'Core user accounts with authentication and profile information';
COMMENT ON COLUMN users.passkey IS 'Unique 32-character key used for tracker announces';
COMMENT ON COLUMN users.is_active IS 'Account is active and can access the site';
COMMENT ON COLUMN users.is_verified IS 'Email has been verified';
COMMENT ON COLUMN users.is_donor IS 'User has donated and receives perks';
COMMENT ON COLUMN users.last_access_at IS 'Last activity timestamp for auto-pruning inactive accounts';
