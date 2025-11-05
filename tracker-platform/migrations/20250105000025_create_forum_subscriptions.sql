-- Create forum_subscriptions table
-- User subscriptions to forum topics

CREATE TABLE forum_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    topic_id UUID NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE,

    -- Notification settings
    notify_on_reply BOOLEAN NOT NULL DEFAULT true,
    notify_email BOOLEAN NOT NULL DEFAULT false,

    -- Tracking
    last_read_post_id UUID REFERENCES forum_posts(id) ON DELETE SET NULL,
    last_read_at TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_forum_subscriptions_user_id ON forum_subscriptions(user_id);
CREATE INDEX idx_forum_subscriptions_topic_id ON forum_subscriptions(topic_id);
CREATE INDEX idx_forum_subscriptions_notify ON forum_subscriptions(notify_on_reply) WHERE notify_on_reply = true;

-- Unique constraint: one subscription per user per topic
CREATE UNIQUE INDEX idx_forum_subscriptions_unique ON forum_subscriptions(user_id, topic_id);

-- Composite index for user's subscriptions
CREATE INDEX idx_forum_subscriptions_user_updated ON forum_subscriptions(user_id, updated_at DESC);

COMMENT ON TABLE forum_subscriptions IS 'User subscriptions to forum topics for notifications';
COMMENT ON COLUMN forum_subscriptions.last_read_post_id IS 'Last post user has read for tracking unread posts';
COMMENT ON COLUMN forum_subscriptions.notify_email IS 'Send email notifications for new replies';
