-- Create user_groups table
-- Defines permission groups for the tracker system (Admin, Moderator, Power User, etc.)

CREATE TABLE user_groups (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    level INTEGER NOT NULL DEFAULT 0, -- Higher level = more permissions
    can_upload BOOLEAN NOT NULL DEFAULT false,
    can_download BOOLEAN NOT NULL DEFAULT true,
    can_request BOOLEAN NOT NULL DEFAULT false,
    can_moderate BOOLEAN NOT NULL DEFAULT false,
    can_edit_torrents BOOLEAN NOT NULL DEFAULT false,
    can_delete_torrents BOOLEAN NOT NULL DEFAULT false,
    can_manage_users BOOLEAN NOT NULL DEFAULT false,
    can_manage_forums BOOLEAN NOT NULL DEFAULT false,
    can_view_ips BOOLEAN NOT NULL DEFAULT false,
    can_view_emails BOOLEAN NOT NULL DEFAULT false,
    can_send_invites BOOLEAN NOT NULL DEFAULT false,
    max_invites INTEGER NOT NULL DEFAULT 0,
    invite_regeneration_days INTEGER NOT NULL DEFAULT 0, -- Days to regenerate 1 invite
    download_slots INTEGER NOT NULL DEFAULT -1, -- -1 = unlimited
    is_immune BOOLEAN NOT NULL DEFAULT false, -- Immune from ratio requirements
    is_freeleech BOOLEAN NOT NULL DEFAULT false, -- Permanent freeleech
    bonus_multiplier DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    icon VARCHAR(50), -- Icon class or emoji
    color VARCHAR(7), -- Hex color for display
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_groups_level ON user_groups(level DESC);

-- Insert default groups (matching Gazelle's system)
INSERT INTO user_groups (name, display_name, level, can_upload, can_download, can_request, can_send_invites, max_invites, icon, color) VALUES
('banned', 'Banned', 0, false, false, false, false, 0, '🚫', '#FF0000'),
('validating', 'Validating', 1, false, true, false, false, 0, '📧', '#CCCCCC'),
('member', 'Member', 2, true, true, false, false, 0, '👤', '#4A90E2'),
('power_user', 'Power User', 3, true, true, true, true, 5, '⭐', '#F5A623'),
('elite', 'Elite', 4, true, true, true, true, 10, '💎', '#7B68EE'),
('torrent_master', 'Torrent Master', 5, true, true, true, true, 20, '🏆', '#FFD700'),
('vip', 'VIP', 6, true, true, true, true, 50, '👑', '#FF1493'),
('uploader', 'Uploader', 7, true, true, true, true, 10, '📤', '#50C878'),
('designer', 'Designer', 8, true, true, true, true, 10, '🎨', '#FF6B6B'),
('forum_moderator', 'Forum Moderator', 10, true, true, true, true, 20, '🛡️', '#3498DB'),
('moderator', 'Moderator', 15, true, true, true, true, 50, '⚖️', '#9B59B6'),
('administrator', 'Administrator', 20, true, true, true, true, 100, '👮', '#E74C3C'),
('sysop', 'Sysop', 25, true, true, true, true, -1, '🔧', '#34495E');

-- Update all groups to have moderation capabilities based on level
UPDATE user_groups SET can_moderate = true, can_edit_torrents = true WHERE level >= 10;
UPDATE user_groups SET can_delete_torrents = true, can_manage_users = true WHERE level >= 15;
UPDATE user_groups SET can_manage_forums = true, can_view_ips = true, can_view_emails = true WHERE level >= 15;
UPDATE user_groups SET is_immune = true WHERE level >= 6;
UPDATE user_groups SET bonus_multiplier = 1.5 WHERE name IN ('power_user', 'uploader');
UPDATE user_groups SET bonus_multiplier = 2.0 WHERE name IN ('elite', 'vip');
UPDATE user_groups SET bonus_multiplier = 2.5 WHERE name IN ('torrent_master');

COMMENT ON TABLE user_groups IS 'User permission groups with role-based access control';
COMMENT ON COLUMN user_groups.level IS 'Permission level - higher values grant more privileges';
COMMENT ON COLUMN user_groups.is_immune IS 'Immune from ratio requirements and auto-disable';
COMMENT ON COLUMN user_groups.bonus_multiplier IS 'Multiplier for bonus point earnings';
