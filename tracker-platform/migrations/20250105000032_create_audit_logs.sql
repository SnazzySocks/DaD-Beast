-- Create audit_logs table
-- Complete audit trail for all important actions

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Actor
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,

    -- Action
    action VARCHAR(100) NOT NULL, -- create, update, delete, login, logout, etc.
    entity_type VARCHAR(50) NOT NULL, -- user, torrent, forum_post, etc.
    entity_id UUID, -- ID of affected entity

    -- Details
    description TEXT,
    changes JSONB, -- Before/after values for updates
    metadata JSONB, -- Additional context

    -- Result
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,

    -- Context
    request_id UUID, -- For correlating related actions
    session_id UUID,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_entity_type ON audit_logs(entity_type);
CREATE INDEX idx_audit_logs_entity_id ON audit_logs(entity_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_ip_address ON audit_logs(ip_address);
CREATE INDEX idx_audit_logs_request_id ON audit_logs(request_id);
CREATE INDEX idx_audit_logs_session_id ON audit_logs(session_id);
CREATE INDEX idx_audit_logs_success ON audit_logs(success) WHERE success = false;

-- Composite index for user actions
CREATE INDEX idx_audit_logs_user_created ON audit_logs(user_id, created_at DESC);

-- Composite index for entity history
CREATE INDEX idx_audit_logs_entity_history ON audit_logs(entity_type, entity_id, created_at DESC);

-- GIN indexes for JSONB
CREATE INDEX idx_audit_logs_changes ON audit_logs USING gin(changes);
CREATE INDEX idx_audit_logs_metadata ON audit_logs USING gin(metadata);

COMMENT ON TABLE audit_logs IS 'Complete audit trail of all important system actions';
COMMENT ON COLUMN audit_logs.action IS 'Action performed: create, update, delete, login, etc.';
COMMENT ON COLUMN audit_logs.entity_type IS 'Type of entity affected (user, torrent, etc.)';
COMMENT ON COLUMN audit_logs.changes IS 'JSONB object with before/after values for updates';
COMMENT ON COLUMN audit_logs.metadata IS 'Additional context and metadata for the action';
COMMENT ON COLUMN audit_logs.request_id IS 'UUID for correlating related actions in a single request';

-- Note: This table will grow large. Consider:
-- 1. Partitioning by created_at (monthly)
-- 2. Archiving old data
-- 3. Using TimescaleDB for better time-series performance
