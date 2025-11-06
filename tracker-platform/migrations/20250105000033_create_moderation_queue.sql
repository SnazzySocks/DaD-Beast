-- Create moderation_queue table
-- Queue for pending moderation items

CREATE TABLE moderation_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Item to moderate (polymorphic)
    item_type VARCHAR(50) NOT NULL, -- torrent, user, comment, forum_post, etc.
    item_id UUID NOT NULL,

    -- Submitter
    submitted_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Queue details
    queue_type VARCHAR(50) NOT NULL, -- approval, review, investigation
    priority VARCHAR(20) NOT NULL DEFAULT 'normal', -- low, normal, high, urgent

    -- Reason for moderation
    reason VARCHAR(100),
    notes TEXT,

    -- Auto-generated flags
    auto_flagged BOOLEAN NOT NULL DEFAULT false,
    auto_flag_reasons TEXT[], -- Array of automated detection reasons
    confidence_score DECIMAL(3,2), -- 0.00 to 1.00 for ML-based detection

    -- Assignment
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMP WITH TIME ZONE,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, in_progress, resolved

    -- Resolution
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolution VARCHAR(50), -- approved, rejected, escalated, etc.
    resolution_notes TEXT,

    -- Metadata
    metadata JSONB, -- Additional context about the item

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_moderation_queue_item_type ON moderation_queue(item_type);
CREATE INDEX idx_moderation_queue_item_id ON moderation_queue(item_id);
CREATE INDEX idx_moderation_queue_submitted_by ON moderation_queue(submitted_by);
CREATE INDEX idx_moderation_queue_queue_type ON moderation_queue(queue_type);
CREATE INDEX idx_moderation_queue_priority ON moderation_queue(priority);
CREATE INDEX idx_moderation_queue_status ON moderation_queue(status);
CREATE INDEX idx_moderation_queue_assigned_to ON moderation_queue(assigned_to);
CREATE INDEX idx_moderation_queue_resolved_by ON moderation_queue(resolved_by);
CREATE INDEX idx_moderation_queue_created_at ON moderation_queue(created_at DESC);
CREATE INDEX idx_moderation_queue_auto_flagged ON moderation_queue(auto_flagged) WHERE auto_flagged = true;

-- Composite index for pending queue
CREATE INDEX idx_moderation_queue_pending ON moderation_queue(status, priority DESC, created_at)
WHERE status = 'pending';

-- Composite index for assigned items
CREATE INDEX idx_moderation_queue_assigned ON moderation_queue(assigned_to, status, created_at DESC)
WHERE status IN ('pending', 'in_progress');

-- Composite index for type and status
CREATE INDEX idx_moderation_queue_type_status ON moderation_queue(item_type, status, created_at DESC);

-- GIN indexes
CREATE INDEX idx_moderation_queue_auto_flag_reasons ON moderation_queue USING gin(auto_flag_reasons);
CREATE INDEX idx_moderation_queue_metadata ON moderation_queue USING gin(metadata);

-- Unique constraint: prevent duplicate items in queue
CREATE UNIQUE INDEX idx_moderation_queue_unique_item ON moderation_queue(item_type, item_id)
WHERE status IN ('pending', 'in_progress');

COMMENT ON TABLE moderation_queue IS 'Queue of items pending moderation review';
COMMENT ON COLUMN moderation_queue.item_type IS 'Type of item to moderate (torrent, user, comment, etc.)';
COMMENT ON COLUMN moderation_queue.auto_flagged IS 'Whether item was automatically flagged by system';
COMMENT ON COLUMN moderation_queue.confidence_score IS 'ML confidence score for auto-flagged items (0.00-1.00)';
COMMENT ON COLUMN moderation_queue.metadata IS 'Additional JSON context about the moderation item';
