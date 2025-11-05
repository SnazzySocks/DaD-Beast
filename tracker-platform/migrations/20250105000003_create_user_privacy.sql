-- Create user_privacy table
-- Privacy settings including Gazelle's paranoia system

CREATE TABLE user_privacy (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,

    -- Paranoia levels (0 = public, 1 = friends, 2 = private)
    -- Based on Gazelle's comprehensive paranoia system
    show_profile INTEGER NOT NULL DEFAULT 0,
    show_stats INTEGER NOT NULL DEFAULT 0,
    show_uploaded INTEGER NOT NULL DEFAULT 0,
    show_downloaded INTEGER NOT NULL DEFAULT 0,
    show_ratio INTEGER NOT NULL DEFAULT 0,
    show_seedbonus INTEGER NOT NULL DEFAULT 1,
    show_snatched INTEGER NOT NULL DEFAULT 1,
    show_seeding INTEGER NOT NULL DEFAULT 1,
    show_leeching INTEGER NOT NULL DEFAULT 1,
    show_uploads INTEGER NOT NULL DEFAULT 0,
    show_requests INTEGER NOT NULL DEFAULT 1,
    show_bounty_spent INTEGER NOT NULL DEFAULT 1,
    show_bounty_earned INTEGER NOT NULL DEFAULT 1,
    show_invites INTEGER NOT NULL DEFAULT 2,
    show_lastseen INTEGER NOT NULL DEFAULT 0,
    show_collages INTEGER NOT NULL DEFAULT 0,
    show_achievements INTEGER NOT NULL DEFAULT 0,

    -- Notification preferences
    notify_upload_comments BOOLEAN NOT NULL DEFAULT true,
    notify_torrent_comments BOOLEAN NOT NULL DEFAULT true,
    notify_forum_replies BOOLEAN NOT NULL DEFAULT true,
    notify_private_messages BOOLEAN NOT NULL DEFAULT true,
    notify_forum_subscriptions BOOLEAN NOT NULL DEFAULT true,
    notify_request_fills BOOLEAN NOT NULL DEFAULT true,
    notify_request_bounty BOOLEAN NOT NULL DEFAULT true,
    notify_news BOOLEAN NOT NULL DEFAULT true,
    notify_announcements BOOLEAN NOT NULL DEFAULT true,

    -- Email preferences
    email_on_private_message BOOLEAN NOT NULL DEFAULT true,
    email_on_forum_reply BOOLEAN NOT NULL DEFAULT false,
    email_on_request_fill BOOLEAN NOT NULL DEFAULT true,
    email_digest BOOLEAN NOT NULL DEFAULT false,
    email_digest_frequency VARCHAR(20) DEFAULT 'weekly', -- daily, weekly, monthly

    -- Privacy flags
    hide_online_status BOOLEAN NOT NULL DEFAULT false,
    disable_inbox BOOLEAN NOT NULL DEFAULT false, -- Disable receiving PMs
    require_encryption BOOLEAN NOT NULL DEFAULT false, -- Require HTTPS announces

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_privacy_updated_at ON user_privacy(updated_at DESC);

COMMENT ON TABLE user_privacy IS 'User privacy settings and paranoia levels based on Gazelle';
COMMENT ON COLUMN user_privacy.show_profile IS 'Who can view profile: 0=public, 1=friends, 2=private';
COMMENT ON COLUMN user_privacy.show_stats IS 'Who can view detailed statistics';
COMMENT ON COLUMN user_privacy.require_encryption IS 'Require HTTPS for tracker announces';
