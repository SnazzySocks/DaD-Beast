-- Create torrent_collections table
-- Playlists/collages for curated torrent collections

CREATE TABLE torrent_collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Collection details
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,

    -- Collection type
    collection_type VARCHAR(50) NOT NULL DEFAULT 'personal', -- personal, staff_picks, theme, genre
    category_id INTEGER REFERENCES torrent_categories(id) ON DELETE SET NULL,

    -- Display settings
    cover_image_url VARCHAR(500),
    background_image_url VARCHAR(500),

    -- Privacy
    is_public BOOLEAN NOT NULL DEFAULT true,
    is_featured BOOLEAN NOT NULL DEFAULT false,
    is_locked BOOLEAN NOT NULL DEFAULT false, -- Prevent editing

    -- Collaboration
    is_collaborative BOOLEAN NOT NULL DEFAULT false,
    collaborators UUID[], -- Array of user IDs who can edit

    -- Statistics
    torrents_count INTEGER NOT NULL DEFAULT 0,
    subscribers_count INTEGER NOT NULL DEFAULT 0,
    views_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create torrent_collection_items table for torrents in collections
CREATE TABLE torrent_collection_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID NOT NULL REFERENCES torrent_collections(id) ON DELETE CASCADE,
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Ordering
    position INTEGER NOT NULL DEFAULT 0,

    -- Optional notes
    notes TEXT,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create torrent_collection_subscriptions table
CREATE TABLE torrent_collection_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID NOT NULL REFERENCES torrent_collections(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Notification preferences
    notify_on_update BOOLEAN NOT NULL DEFAULT true,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for torrent_collections
CREATE INDEX idx_torrent_collections_creator_id ON torrent_collections(creator_id);
CREATE INDEX idx_torrent_collections_slug ON torrent_collections(slug);
CREATE INDEX idx_torrent_collections_category_id ON torrent_collections(category_id);
CREATE INDEX idx_torrent_collections_collection_type ON torrent_collections(collection_type);
CREATE INDEX idx_torrent_collections_is_public ON torrent_collections(is_public) WHERE is_public = true;
CREATE INDEX idx_torrent_collections_is_featured ON torrent_collections(is_featured) WHERE is_featured = true;
CREATE INDEX idx_torrent_collections_created_at ON torrent_collections(created_at DESC);
CREATE INDEX idx_torrent_collections_subscribers_count ON torrent_collections(subscribers_count DESC);

-- GIN index for collaborators array
CREATE INDEX idx_torrent_collections_collaborators ON torrent_collections USING gin(collaborators);

-- Create indexes for torrent_collection_items
CREATE INDEX idx_torrent_collection_items_collection_id ON torrent_collection_items(collection_id);
CREATE INDEX idx_torrent_collection_items_torrent_id ON torrent_collection_items(torrent_id);
CREATE INDEX idx_torrent_collection_items_added_by ON torrent_collection_items(added_by);
CREATE INDEX idx_torrent_collection_items_position ON torrent_collection_items(collection_id, position);

-- Unique constraint: prevent duplicate torrents in same collection
CREATE UNIQUE INDEX idx_torrent_collection_items_unique ON torrent_collection_items(collection_id, torrent_id);

-- Create indexes for torrent_collection_subscriptions
CREATE INDEX idx_torrent_collection_subscriptions_collection_id ON torrent_collection_subscriptions(collection_id);
CREATE INDEX idx_torrent_collection_subscriptions_user_id ON torrent_collection_subscriptions(user_id);

-- Unique constraint: one subscription per user per collection
CREATE UNIQUE INDEX idx_torrent_collection_subscriptions_unique ON torrent_collection_subscriptions(collection_id, user_id);

COMMENT ON TABLE torrent_collections IS 'Curated collections/playlists of torrents';
COMMENT ON TABLE torrent_collection_items IS 'Torrents included in collections with ordering';
COMMENT ON TABLE torrent_collection_subscriptions IS 'User subscriptions to collections';
COMMENT ON COLUMN torrent_collections.is_collaborative IS 'Allow collaborators to add/remove torrents';
COMMENT ON COLUMN torrent_collections.collaborators IS 'Array of user IDs who can edit this collection';
