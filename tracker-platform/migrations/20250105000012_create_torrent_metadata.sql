-- Create torrent_metadata table
-- External media metadata (TMDB, IGDB, MusicBrainz, etc.)

CREATE TABLE torrent_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,

    -- External database links
    tmdb_id INTEGER, -- The Movie Database ID
    imdb_id VARCHAR(20), -- IMDB ID (tt1234567)
    tvdb_id INTEGER, -- TheTVDB ID
    igdb_id INTEGER, -- Internet Game Database ID
    musicbrainz_id UUID, -- MusicBrainz ID
    mal_id INTEGER, -- MyAnimeList ID
    anidb_id INTEGER, -- AniDB ID

    -- Media-specific metadata
    media_type VARCHAR(50), -- movie, tv_show, tv_episode, album, game, etc.

    -- Movie/TV metadata
    title VARCHAR(500),
    original_title VARCHAR(500),
    release_date DATE,
    runtime_minutes INTEGER,
    genres TEXT[], -- Array of genres
    directors TEXT[], -- Array of director names
    actors TEXT[], -- Array of main actor names
    plot_summary TEXT,
    poster_url VARCHAR(500),
    backdrop_url VARCHAR(500),
    trailer_url VARCHAR(500),

    -- TV-specific metadata
    season_number INTEGER,
    episode_number INTEGER,
    episode_title VARCHAR(500),
    series_name VARCHAR(500),

    -- Music metadata
    artist VARCHAR(500),
    album VARCHAR(500),
    record_label VARCHAR(200),
    catalog_number VARCHAR(100),

    -- Game metadata
    publisher VARCHAR(200),
    developer VARCHAR(200),
    platform TEXT[], -- Array of platforms

    -- Ratings
    rating_average DECIMAL(3,1), -- Average rating (e.g., 8.5)
    rating_count INTEGER, -- Number of ratings
    metacritic_score INTEGER,

    -- Additional data
    metadata_json JSONB, -- Full JSON response from external API

    -- Status
    is_verified BOOLEAN NOT NULL DEFAULT false, -- Verified by moderator
    last_updated TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_torrent_metadata_torrent_id ON torrent_metadata(torrent_id);
CREATE INDEX idx_torrent_metadata_tmdb_id ON torrent_metadata(tmdb_id);
CREATE INDEX idx_torrent_metadata_imdb_id ON torrent_metadata(imdb_id);
CREATE INDEX idx_torrent_metadata_igdb_id ON torrent_metadata(igdb_id);
CREATE INDEX idx_torrent_metadata_media_type ON torrent_metadata(media_type);
CREATE INDEX idx_torrent_metadata_release_date ON torrent_metadata(release_date);
CREATE INDEX idx_torrent_metadata_is_verified ON torrent_metadata(is_verified) WHERE is_verified = true;

-- GIN index for JSONB metadata search
CREATE INDEX idx_torrent_metadata_json ON torrent_metadata USING gin(metadata_json);

-- GIN indexes for array columns
CREATE INDEX idx_torrent_metadata_genres ON torrent_metadata USING gin(genres);
CREATE INDEX idx_torrent_metadata_platform ON torrent_metadata USING gin(platform);

COMMENT ON TABLE torrent_metadata IS 'External media metadata from TMDB, IGDB, MusicBrainz, etc.';
COMMENT ON COLUMN torrent_metadata.tmdb_id IS 'The Movie Database (TMDB) identifier';
COMMENT ON COLUMN torrent_metadata.metadata_json IS 'Full JSON response from external API for complete data';
COMMENT ON COLUMN torrent_metadata.is_verified IS 'Whether metadata has been verified by a moderator';
