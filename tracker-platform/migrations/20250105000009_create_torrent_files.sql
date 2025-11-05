-- Create torrent_files table
-- Individual files within torrents

CREATE TABLE torrent_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,

    -- File details
    file_path TEXT NOT NULL, -- Full path within torrent
    file_name VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    file_index INTEGER NOT NULL, -- Index in torrent file list

    -- File type detection
    file_extension VARCHAR(20),
    mime_type VARCHAR(100),

    -- Media file information
    is_media BOOLEAN NOT NULL DEFAULT false,
    duration_seconds INTEGER, -- For video/audio files
    width INTEGER, -- For video files
    height INTEGER, -- For video files
    bitrate_kbps INTEGER,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_torrent_files_torrent_id ON torrent_files(torrent_id);
CREATE INDEX idx_torrent_files_file_name ON torrent_files(file_name);
CREATE INDEX idx_torrent_files_file_extension ON torrent_files(file_extension);
CREATE INDEX idx_torrent_files_is_media ON torrent_files(is_media) WHERE is_media = true;

-- Unique constraint for file index within torrent
CREATE UNIQUE INDEX idx_torrent_files_unique ON torrent_files(torrent_id, file_index);

COMMENT ON TABLE torrent_files IS 'Individual files contained within torrents';
COMMENT ON COLUMN torrent_files.file_path IS 'Full path of file within torrent structure';
COMMENT ON COLUMN torrent_files.file_index IS 'Zero-based index of file in torrent';
COMMENT ON COLUMN torrent_files.is_media IS 'Whether file is a media file (video/audio)';
