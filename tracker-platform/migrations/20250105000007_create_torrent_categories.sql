-- Create torrent_categories table
-- Categories for organizing torrents (movies, TV, music, games, etc.)

CREATE TABLE torrent_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    parent_id INTEGER REFERENCES torrent_categories(id) ON DELETE SET NULL,

    -- Display settings
    icon VARCHAR(50), -- Icon class or emoji
    color VARCHAR(7), -- Hex color
    image_url VARCHAR(500),
    description TEXT,

    -- Ordering and hierarchy
    position INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 0, -- Depth in hierarchy

    -- Category settings
    is_active BOOLEAN NOT NULL DEFAULT true,
    requires_moderation BOOLEAN NOT NULL DEFAULT false,

    -- Statistics
    torrent_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_torrent_categories_slug ON torrent_categories(slug);
CREATE INDEX idx_torrent_categories_parent_id ON torrent_categories(parent_id);
CREATE INDEX idx_torrent_categories_position ON torrent_categories(position);
CREATE INDEX idx_torrent_categories_is_active ON torrent_categories(is_active) WHERE is_active = true;

-- Insert default categories
INSERT INTO torrent_categories (name, display_name, slug, icon, color, position) VALUES
('movies', 'Movies', 'movies', '🎬', '#E74C3C', 1),
('tv', 'TV Shows', 'tv', '📺', '#3498DB', 2),
('music', 'Music', 'music', '🎵', '#9B59B6', 3),
('games', 'Games', 'games', '🎮', '#2ECC71', 4),
('software', 'Software', 'software', '💻', '#1ABC9C', 5),
('books', 'Books', 'books', '📚', '#F39C12', 6),
('anime', 'Anime', 'anime', '🎌', '#E91E63', 7),
('xxx', 'XXX', 'xxx', '🔞', '#95A5A6', 8);

-- Insert subcategories for Movies
INSERT INTO torrent_categories (name, display_name, slug, parent_id, position)
SELECT
    'movies_' || slug,
    name,
    'movies-' || slug,
    (SELECT id FROM torrent_categories WHERE name = 'movies'),
    position
FROM (VALUES
    ('hd', 'HD', 1),
    ('sd', 'SD', 2),
    ('uhd', '4K/UHD', 3),
    ('remux', 'Remux', 4),
    ('bluray', 'BluRay', 5),
    ('dvdr', 'DVDR', 6)
) AS subcats(slug, name, position);

-- Insert subcategories for TV
INSERT INTO torrent_categories (name, display_name, slug, parent_id, position)
SELECT
    'tv_' || slug,
    name,
    'tv-' || slug,
    (SELECT id FROM torrent_categories WHERE name = 'tv'),
    position
FROM (VALUES
    ('hd', 'HD', 1),
    ('sd', 'SD', 2),
    ('uhd', '4K/UHD', 3),
    ('web', 'WEB-DL', 4)
) AS subcats(slug, name, position);

-- Insert subcategories for Music
INSERT INTO torrent_categories (name, display_name, slug, parent_id, position)
SELECT
    'music_' || slug,
    name,
    'music-' || slug,
    (SELECT id FROM torrent_categories WHERE name = 'music'),
    position
FROM (VALUES
    ('mp3', 'MP3', 1),
    ('flac', 'FLAC', 2),
    ('vinyl', 'Vinyl', 3),
    ('live', 'Live', 4)
) AS subcats(slug, name, position);

-- Update level for subcategories
UPDATE torrent_categories SET level = 1 WHERE parent_id IS NOT NULL;

COMMENT ON TABLE torrent_categories IS 'Hierarchical categories for organizing torrents';
COMMENT ON COLUMN torrent_categories.parent_id IS 'Parent category for hierarchical organization';
COMMENT ON COLUMN torrent_categories.level IS 'Depth in category hierarchy (0=root, 1=subcategory)';
