-- Create search_index_queue table
-- Queue for indexing content into Meilisearch

CREATE TABLE search_index_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Entity to index
    entity_type VARCHAR(50) NOT NULL, -- torrent, user, forum_post, etc.
    entity_id UUID NOT NULL,

    -- Operation
    operation VARCHAR(20) NOT NULL, -- index, update, delete

    -- Payload
    payload JSONB, -- Full document to index

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, processing, completed, failed
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,

    -- Error tracking
    last_error TEXT,
    last_attempted_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes
CREATE INDEX idx_search_index_queue_entity_type ON search_index_queue(entity_type);
CREATE INDEX idx_search_index_queue_entity_id ON search_index_queue(entity_id);
CREATE INDEX idx_search_index_queue_operation ON search_index_queue(operation);
CREATE INDEX idx_search_index_queue_status ON search_index_queue(status);
CREATE INDEX idx_search_index_queue_created_at ON search_index_queue(created_at);

-- Composite index for pending items
CREATE INDEX idx_search_index_queue_pending ON search_index_queue(status, created_at)
WHERE status = 'pending';

-- Composite index for failed items that can be retried
CREATE INDEX idx_search_index_queue_retry ON search_index_queue(status, attempts, created_at)
WHERE status = 'failed' AND attempts < max_attempts;

-- GIN index for payload search
CREATE INDEX idx_search_index_queue_payload ON search_index_queue USING gin(payload);

-- Unique constraint: prevent duplicate pending operations
CREATE UNIQUE INDEX idx_search_index_queue_unique_pending ON search_index_queue(entity_type, entity_id, operation)
WHERE status = 'pending';

COMMENT ON TABLE search_index_queue IS 'Queue for asynchronous indexing into Meilisearch';
COMMENT ON COLUMN search_index_queue.entity_type IS 'Type of entity to index (torrent, user, etc.)';
COMMENT ON COLUMN search_index_queue.operation IS 'Index operation: index, update, delete';
COMMENT ON COLUMN search_index_queue.payload IS 'Full document payload to send to Meilisearch';
COMMENT ON COLUMN search_index_queue.attempts IS 'Number of processing attempts (for retry logic)';

-- Note: Completed/old items should be cleaned up periodically
