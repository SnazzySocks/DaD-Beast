-- Migration: Create search-related tables
-- Description: Tables for search indexing queue, history, clicks, and A/B testing

-- Search index queue for async indexing
CREATE TABLE IF NOT EXISTS search_index_queue (
    id BIGSERIAL PRIMARY KEY,
    torrent_id UUID NOT NULL,
    operation VARCHAR(10) NOT NULL CHECK (operation IN ('upsert', 'delete')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_torrent_queue UNIQUE(torrent_id)
);

CREATE INDEX IF NOT EXISTS idx_search_queue_created ON search_index_queue(created_at);
CREATE INDEX IF NOT EXISTS idx_search_queue_operation ON search_index_queue(operation);

COMMENT ON TABLE search_index_queue IS 'Queue for async Meilisearch indexing operations';
COMMENT ON COLUMN search_index_queue.operation IS 'Operation type: upsert or delete';

-- Search history for analytics and suggestions
CREATE TABLE IF NOT EXISTS search_history (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    query TEXT NOT NULL,
    filters TEXT,
    results_count BIGINT NOT NULL DEFAULT 0,
    processing_time_ms BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_history_user ON search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_search_history_created ON search_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_query ON search_history(query);
CREATE INDEX IF NOT EXISTS idx_search_history_results ON search_history(results_count);

COMMENT ON TABLE search_history IS 'Search query history for analytics and autocomplete';
COMMENT ON COLUMN search_history.filters IS 'JSON or text representation of applied filters';

-- Search clicks for CTR (Click-Through Rate) tracking
CREATE TABLE IF NOT EXISTS search_clicks (
    id BIGSERIAL PRIMARY KEY,
    search_id BIGINT REFERENCES search_history(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    torrent_id UUID NOT NULL REFERENCES torrents(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_clicks_search ON search_clicks(search_id);
CREATE INDEX IF NOT EXISTS idx_search_clicks_user ON search_clicks(user_id);
CREATE INDEX IF NOT EXISTS idx_search_clicks_torrent ON search_clicks(torrent_id);
CREATE INDEX IF NOT EXISTS idx_search_clicks_created ON search_clicks(created_at DESC);

COMMENT ON TABLE search_clicks IS 'Tracks which search results users click on';
COMMENT ON COLUMN search_clicks.position IS 'Position of clicked result in search results (1-indexed)';

-- A/B testing for search algorithms
CREATE TABLE IF NOT EXISTS search_ab_tests (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    test_name VARCHAR(100) NOT NULL,
    variant VARCHAR(50) NOT NULL,
    query TEXT NOT NULL,
    results_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_ab_tests_test ON search_ab_tests(test_name, variant);
CREATE INDEX IF NOT EXISTS idx_search_ab_tests_user ON search_ab_tests(user_id);
CREATE INDEX IF NOT EXISTS idx_search_ab_tests_created ON search_ab_tests(created_at DESC);

COMMENT ON TABLE search_ab_tests IS 'A/B testing data for search algorithm experiments';
COMMENT ON COLUMN search_ab_tests.test_name IS 'Name of the A/B test being run';
COMMENT ON COLUMN search_ab_tests.variant IS 'Test variant (e.g., control, variant_a, variant_b)';

-- Materialized view for popular searches (updated periodically)
CREATE MATERIALIZED VIEW IF NOT EXISTS popular_searches AS
SELECT 
    query,
    COUNT(*) as search_count,
    AVG(results_count) as avg_results,
    MAX(created_at) as last_searched
FROM search_history
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY query
HAVING COUNT(*) >= 5
ORDER BY search_count DESC
LIMIT 1000;

CREATE UNIQUE INDEX IF NOT EXISTS idx_popular_searches_query ON popular_searches(query);

COMMENT ON MATERIALIZED VIEW popular_searches IS 'Popular search queries from last 30 days (refreshed periodically)';

-- Function to refresh popular searches view
CREATE OR REPLACE FUNCTION refresh_popular_searches()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY popular_searches;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_popular_searches() IS 'Refresh the popular_searches materialized view';

-- Function to clean old search history (run periodically)
CREATE OR REPLACE FUNCTION cleanup_old_search_history(days_to_keep INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM search_history
    WHERE created_at < NOW() - (days_to_keep || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_search_history(INTEGER) IS 'Delete search history older than specified days (default: 90)';

-- Trigger function to queue torrents for indexing on insert/update
CREATE OR REPLACE FUNCTION queue_torrent_for_indexing()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        INSERT INTO search_index_queue (torrent_id, operation)
        VALUES (OLD.id, 'delete')
        ON CONFLICT (torrent_id) 
        DO UPDATE SET operation = 'delete', created_at = NOW();
        RETURN OLD;
    ELSE
        INSERT INTO search_index_queue (torrent_id, operation)
        VALUES (NEW.id, 'upsert')
        ON CONFLICT (torrent_id) 
        DO UPDATE SET operation = 'upsert', created_at = NOW();
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION queue_torrent_for_indexing() IS 'Automatically queue torrents for search indexing on changes';

-- Create trigger on torrents table (if not exists)
DROP TRIGGER IF EXISTS trigger_queue_torrent_indexing ON torrents;
CREATE TRIGGER trigger_queue_torrent_indexing
    AFTER INSERT OR UPDATE OR DELETE ON torrents
    FOR EACH ROW
    EXECUTE FUNCTION queue_torrent_for_indexing();

COMMENT ON TRIGGER trigger_queue_torrent_indexing ON torrents IS 'Queue torrent for search indexing on any change';

-- Grant necessary permissions (adjust as needed for your user)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON search_index_queue TO tracker_app;
-- GRANT SELECT, INSERT ON search_history TO tracker_app;
-- GRANT SELECT, INSERT ON search_clicks TO tracker_app;
-- GRANT SELECT, INSERT ON search_ab_tests TO tracker_app;
-- GRANT SELECT ON popular_searches TO tracker_app;
