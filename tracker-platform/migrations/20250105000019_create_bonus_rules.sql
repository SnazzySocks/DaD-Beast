-- Create bonus_rules table
-- Rule-based bonus point earning system

CREATE TABLE bonus_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Rule identification
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    rule_type VARCHAR(50) NOT NULL, -- seed_time, upload_size, ratio, longevity, etc.

    -- Rule conditions
    conditions JSONB NOT NULL, -- JSON conditions for rule evaluation

    -- Rewards
    points_per_hour DECIMAL(10,2), -- For time-based rules
    points_per_gb DECIMAL(10,2), -- For size-based rules
    one_time_points DECIMAL(10,2), -- For achievement-based rules

    -- Multipliers
    base_multiplier DECIMAL(4,2) NOT NULL DEFAULT 1.00,
    size_multiplier_enabled BOOLEAN NOT NULL DEFAULT true, -- Larger torrents earn more

    -- Limits
    max_points_per_torrent DECIMAL(10,2), -- Cap per torrent
    max_points_per_day DECIMAL(10,2), -- Daily cap per user
    min_seeders INTEGER, -- Only apply if seeders < threshold
    max_seeders INTEGER, -- Only apply if seeders > threshold

    -- Torrent requirements
    min_torrent_size_bytes BIGINT,
    max_torrent_size_bytes BIGINT,
    min_seed_time_hours INTEGER,
    category_ids INTEGER[], -- Limit to specific categories

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0, -- Higher priority rules evaluated first

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    effective_from TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    effective_until TIMESTAMP WITH TIME ZONE -- NULL = no expiry
);

-- Create indexes
CREATE INDEX idx_bonus_rules_rule_type ON bonus_rules(rule_type);
CREATE INDEX idx_bonus_rules_is_active ON bonus_rules(is_active) WHERE is_active = true;
CREATE INDEX idx_bonus_rules_priority ON bonus_rules(priority DESC);
CREATE INDEX idx_bonus_rules_effective_from ON bonus_rules(effective_from);
CREATE INDEX idx_bonus_rules_effective_until ON bonus_rules(effective_until) WHERE effective_until IS NOT NULL;

-- GIN index for JSONB conditions
CREATE INDEX idx_bonus_rules_conditions ON bonus_rules USING gin(conditions);

-- GIN index for category array
CREATE INDEX idx_bonus_rules_category_ids ON bonus_rules USING gin(category_ids);

-- Insert default bonus rules
INSERT INTO bonus_rules (name, description, rule_type, points_per_hour, base_multiplier, size_multiplier_enabled, min_seeders, conditions) VALUES
('standard_seeding', 'Standard seeding bonus', 'seed_time', 1.00, 1.00, true, NULL, '{"min_ratio": 0.5}'::jsonb),
('low_seeder_bonus', 'Extra bonus for low-seeder torrents', 'seed_time', 2.00, 1.50, true, 5, '{"min_ratio": 0.5}'::jsonb),
('freeleech_seeding', 'Bonus for seeding freeleech torrents', 'seed_time', 1.50, 1.25, true, NULL, '{"is_freeleech": true}'::jsonb),
('large_file_bonus', 'Bonus for seeding large files', 'seed_time', 1.50, 1.50, true, NULL, '{"min_size_gb": 50}'::jsonb);

COMMENT ON TABLE bonus_rules IS 'Configurable rules for earning bonus points';
COMMENT ON COLUMN bonus_rules.conditions IS 'JSON conditions that must be met for rule to apply';
COMMENT ON COLUMN bonus_rules.size_multiplier_enabled IS 'Apply multiplier based on torrent size';
COMMENT ON COLUMN bonus_rules.min_seeders IS 'Only apply rule if torrent has fewer than this many seeders';
