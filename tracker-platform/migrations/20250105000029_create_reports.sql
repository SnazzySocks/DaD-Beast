-- Create reports table
-- User reports for content moderation

CREATE TABLE reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    -- Reported entity (polymorphic)
    report_type VARCHAR(50) NOT NULL, -- torrent, user, comment, forum_post, pm, etc.
    reported_entity_id UUID NOT NULL, -- ID of reported entity

    -- Report details
    reason VARCHAR(50) NOT NULL, -- duplicate, fake, malware, copyright, spam, abuse, etc.
    description TEXT NOT NULL,

    -- Evidence
    evidence_urls TEXT[], -- Array of evidence URLs
    evidence_metadata JSONB,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, investigating, resolved, dismissed
    priority VARCHAR(20) NOT NULL DEFAULT 'normal', -- low, normal, high, critical

    -- Assignment
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_at TIMESTAMP WITH TIME ZONE,

    -- Resolution
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolution TEXT,
    action_taken VARCHAR(100), -- deleted, warned, banned, no_action, etc.

    -- Reporter info
    reporter_ip INET,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_reports_reporter_id ON reports(reporter_id);
CREATE INDEX idx_reports_report_type ON reports(report_type);
CREATE INDEX idx_reports_reported_entity_id ON reports(reported_entity_id);
CREATE INDEX idx_reports_status ON reports(status);
CREATE INDEX idx_reports_priority ON reports(priority);
CREATE INDEX idx_reports_assigned_to ON reports(assigned_to);
CREATE INDEX idx_reports_resolved_by ON reports(resolved_by);
CREATE INDEX idx_reports_created_at ON reports(created_at DESC);

-- Composite index for pending reports by priority
CREATE INDEX idx_reports_pending ON reports(status, priority DESC, created_at)
WHERE status = 'pending';

-- Composite index for type and status
CREATE INDEX idx_reports_type_status ON reports(report_type, status, created_at DESC);

-- GIN indexes
CREATE INDEX idx_reports_evidence_urls ON reports USING gin(evidence_urls);
CREATE INDEX idx_reports_evidence_metadata ON reports USING gin(evidence_metadata);

COMMENT ON TABLE reports IS 'User reports for content moderation';
COMMENT ON COLUMN reports.report_type IS 'Type of entity being reported (torrent, user, comment, etc.)';
COMMENT ON COLUMN reports.reported_entity_id IS 'UUID of the reported entity';
COMMENT ON COLUMN reports.evidence_metadata IS 'Additional JSON evidence data';
