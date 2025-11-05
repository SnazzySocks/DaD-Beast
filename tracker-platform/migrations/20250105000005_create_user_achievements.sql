-- Create user_achievements table
-- Badge and achievement system for gamification

CREATE TABLE user_achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Achievement definition
    achievement_type VARCHAR(50) NOT NULL, -- upload_milestone, ratio_master, seeding_legend, etc.
    achievement_tier INTEGER NOT NULL DEFAULT 1, -- Bronze, Silver, Gold, etc.
    achievement_name VARCHAR(100) NOT NULL,
    achievement_description TEXT,

    -- Achievement icon/badge
    icon_url VARCHAR(500),
    badge_color VARCHAR(7), -- Hex color

    -- Achievement metadata
    threshold_value DECIMAL(20,2), -- The value achieved (e.g., 1000GB uploaded)
    metadata JSONB, -- Additional data about the achievement

    -- Display settings
    is_visible BOOLEAN NOT NULL DEFAULT true,
    is_featured BOOLEAN NOT NULL DEFAULT false, -- Show on profile
    display_order INTEGER NOT NULL DEFAULT 0,

    -- Points/rewards
    bonus_points_awarded DECIMAL(20,2) NOT NULL DEFAULT 0.00,
    invites_awarded INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    earned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_user_achievements_user_id ON user_achievements(user_id);
CREATE INDEX idx_user_achievements_type ON user_achievements(achievement_type);
CREATE INDEX idx_user_achievements_earned_at ON user_achievements(earned_at DESC);
CREATE INDEX idx_user_achievements_featured ON user_achievements(is_featured) WHERE is_featured = true;

-- Unique constraint: user can't earn same achievement multiple times
CREATE UNIQUE INDEX idx_user_achievements_unique ON user_achievements(user_id, achievement_type, achievement_tier);

COMMENT ON TABLE user_achievements IS 'User badges and achievements for gamification';
COMMENT ON COLUMN user_achievements.achievement_type IS 'Category of achievement (upload_milestone, ratio_master, etc.)';
COMMENT ON COLUMN user_achievements.achievement_tier IS 'Tier level: 1=Bronze, 2=Silver, 3=Gold, 4=Platinum, 5=Diamond';
COMMENT ON COLUMN user_achievements.metadata IS 'Additional JSON data about the achievement';
