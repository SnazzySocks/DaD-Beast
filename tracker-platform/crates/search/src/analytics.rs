//! Search analytics and tracking

use crate::error::{SearchError, SearchResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Search analytics service
pub struct SearchAnalytics {
    db: PgPool,
}

impl SearchAnalytics {
    /// Create a new search analytics service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Track a search query
    pub async fn track_search(
        &self,
        user_id: Option<Uuid>,
        query: &str,
        filters: Option<&str>,
        results_count: u64,
        processing_time_ms: u64,
    ) -> SearchResult<()> {
        sqlx::query(
            r#"
            INSERT INTO search_history 
            (user_id, query, filters, results_count, processing_time_ms, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#
        )
        .bind(user_id)
        .bind(query)
        .bind(filters)
        .bind(results_count as i64)
        .bind(processing_time_ms as i64)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Track when a user clicks on a search result
    pub async fn track_click(
        &self,
        search_id: Option<i64>,
        user_id: Option<Uuid>,
        torrent_id: Uuid,
        position: i32,
    ) -> SearchResult<()> {
        sqlx::query(
            r#"
            INSERT INTO search_clicks 
            (search_id, user_id, torrent_id, position, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#
        )
        .bind(search_id)
        .bind(user_id)
        .bind(torrent_id)
        .bind(position)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get most popular searches
    pub async fn get_popular_searches(
        &self,
        limit: usize,
        days: i64,
    ) -> SearchResult<Vec<PopularSearch>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                query,
                COUNT(*) as search_count,
                AVG(results_count) as avg_results,
                AVG(processing_time_ms) as avg_time_ms
            FROM search_history
            WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            GROUP BY query
            ORDER BY search_count DESC
            LIMIT $2
            "#
        )
        .bind(days)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let searches = rows
            .into_iter()
            .map(|row| PopularSearch {
                query: row.get("query"),
                search_count: row.get("search_count"),
                avg_results: row.get::<Option<f64>, _>("avg_results").unwrap_or(0.0) as u64,
                avg_time_ms: row.get::<Option<f64>, _>("avg_time_ms").unwrap_or(0.0) as u64,
            })
            .collect();

        Ok(searches)
    }

    /// Get searches with no results
    pub async fn get_no_result_searches(
        &self,
        limit: usize,
        days: i64,
    ) -> SearchResult<Vec<NoResultSearch>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                query,
                COUNT(*) as search_count,
                MAX(created_at) as last_searched
            FROM search_history
            WHERE results_count = 0
              AND created_at > NOW() - $1 * INTERVAL '1 day'
            GROUP BY query
            ORDER BY search_count DESC
            LIMIT $2
            "#
        )
        .bind(days)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let searches = rows
            .into_iter()
            .map(|row| NoResultSearch {
                query: row.get("query"),
                search_count: row.get("search_count"),
                last_searched: row.get("last_searched"),
            })
            .collect();

        Ok(searches)
    }

    /// Calculate search-to-click ratio (CTR)
    pub async fn get_click_through_rate(&self, days: i64) -> SearchResult<f64> {
        let row = sqlx::query(
            r#"
            WITH search_counts AS (
                SELECT COUNT(DISTINCT id) as total_searches
                FROM search_history
                WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            ),
            click_counts AS (
                SELECT COUNT(DISTINCT search_id) as searches_with_clicks
                FROM search_clicks
                WHERE created_at > NOW() - $1 * INTERVAL '1 day'
                  AND search_id IS NOT NULL
            )
            SELECT 
                CASE 
                    WHEN sc.total_searches > 0 
                    THEN (cc.searches_with_clicks::float / sc.total_searches::float) * 100
                    ELSE 0
                END as ctr
            FROM search_counts sc, click_counts cc
            "#
        )
        .bind(days)
        .fetch_one(&self.db)
        .await?;

        Ok(row.get::<Option<f64>, _>("ctr").unwrap_or(0.0))
    }

    /// Get search performance statistics
    pub async fn get_performance_stats(&self, days: i64) -> SearchResult<PerformanceStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_searches,
                AVG(processing_time_ms) as avg_time_ms,
                MIN(processing_time_ms) as min_time_ms,
                MAX(processing_time_ms) as max_time_ms,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY processing_time_ms) as median_time_ms,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY processing_time_ms) as p95_time_ms,
                AVG(results_count) as avg_results
            FROM search_history
            WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            "#
        )
        .bind(days)
        .fetch_one(&self.db)
        .await?;

        Ok(PerformanceStats {
            total_searches: row.get("total_searches"),
            avg_time_ms: row.get::<Option<f64>, _>("avg_time_ms").unwrap_or(0.0) as u64,
            min_time_ms: row.get::<Option<i64>, _>("min_time_ms").unwrap_or(0) as u64,
            max_time_ms: row.get::<Option<i64>, _>("max_time_ms").unwrap_or(0) as u64,
            median_time_ms: row.get::<Option<f64>, _>("median_time_ms").unwrap_or(0.0) as u64,
            p95_time_ms: row.get::<Option<f64>, _>("p95_time_ms").unwrap_or(0.0) as u64,
            avg_results: row.get::<Option<f64>, _>("avg_results").unwrap_or(0.0) as u64,
        })
    }

    /// Get user search history
    pub async fn get_user_search_history(
        &self,
        user_id: Uuid,
        limit: usize,
    ) -> SearchResult<Vec<SearchHistoryItem>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                query,
                filters,
                results_count,
                processing_time_ms,
                created_at
            FROM search_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let history = rows
            .into_iter()
            .map(|row| SearchHistoryItem {
                id: row.get("id"),
                query: row.get("query"),
                filters: row.get("filters"),
                results_count: row.get("results_count"),
                processing_time_ms: row.get("processing_time_ms"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(history)
    }

    /// Get top clicked torrents from search results
    pub async fn get_top_clicked_torrents(
        &self,
        limit: usize,
        days: i64,
    ) -> SearchResult<Vec<TopClickedTorrent>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                sc.torrent_id,
                t.name,
                COUNT(*) as click_count,
                AVG(sc.position) as avg_position
            FROM search_clicks sc
            JOIN torrents t ON sc.torrent_id = t.id
            WHERE sc.created_at > NOW() - $1 * INTERVAL '1 day'
            GROUP BY sc.torrent_id, t.name
            ORDER BY click_count DESC
            LIMIT $2
            "#
        )
        .bind(days)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        let torrents = rows
            .into_iter()
            .map(|row| TopClickedTorrent {
                torrent_id: row.get("torrent_id"),
                name: row.get("name"),
                click_count: row.get("click_count"),
                avg_position: row.get::<Option<f64>, _>("avg_position").unwrap_or(0.0),
            })
            .collect();

        Ok(torrents)
    }

    /// Get search trends over time
    pub async fn get_search_trends(&self, days: i64) -> SearchResult<Vec<SearchTrend>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                DATE(created_at) as date,
                COUNT(*) as search_count,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(results_count) as avg_results
            FROM search_history
            WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            GROUP BY DATE(created_at)
            ORDER BY date DESC
            "#
        )
        .bind(days)
        .fetch_all(&self.db)
        .await?;

        let trends = rows
            .into_iter()
            .map(|row| SearchTrend {
                date: row.get("date"),
                search_count: row.get("search_count"),
                unique_users: row.get("unique_users"),
                avg_results: row.get::<Option<f64>, _>("avg_results").unwrap_or(0.0) as u64,
            })
            .collect();

        Ok(trends)
    }

    /// Track A/B test variant
    pub async fn track_ab_test(
        &self,
        user_id: Option<Uuid>,
        test_name: &str,
        variant: &str,
        query: &str,
        results_count: u64,
    ) -> SearchResult<()> {
        sqlx::query(
            r#"
            INSERT INTO search_ab_tests 
            (user_id, test_name, variant, query, results_count, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#
        )
        .bind(user_id)
        .bind(test_name)
        .bind(variant)
        .bind(query)
        .bind(results_count as i64)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get A/B test results
    pub async fn get_ab_test_results(
        &self,
        test_name: &str,
        days: i64,
    ) -> SearchResult<Vec<ABTestResult>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                variant,
                COUNT(*) as total_searches,
                AVG(results_count) as avg_results,
                COUNT(DISTINCT user_id) as unique_users
            FROM search_ab_tests
            WHERE test_name = $1
              AND created_at > NOW() - $2 * INTERVAL '1 day'
            GROUP BY variant
            ORDER BY variant
            "#
        )
        .bind(test_name)
        .bind(days)
        .fetch_all(&self.db)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| ABTestResult {
                variant: row.get("variant"),
                total_searches: row.get("total_searches"),
                avg_results: row.get::<Option<f64>, _>("avg_results").unwrap_or(0.0) as u64,
                unique_users: row.get("unique_users"),
            })
            .collect();

        Ok(results)
    }

    /// Get filter usage statistics
    pub async fn get_filter_usage_stats(&self, days: i64) -> SearchResult<Vec<FilterUsageStat>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                filters,
                COUNT(*) as usage_count,
                AVG(results_count) as avg_results
            FROM search_history
            WHERE filters IS NOT NULL
              AND created_at > NOW() - $1 * INTERVAL '1 day'
            GROUP BY filters
            ORDER BY usage_count DESC
            LIMIT 50
            "#
        )
        .bind(days)
        .fetch_all(&self.db)
        .await?;

        let stats = rows
            .into_iter()
            .map(|row| FilterUsageStat {
                filters: row.get("filters"),
                usage_count: row.get("usage_count"),
                avg_results: row.get::<Option<f64>, _>("avg_results").unwrap_or(0.0) as u64,
            })
            .collect();

        Ok(stats)
    }
}

/// Popular search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularSearch {
    pub query: String,
    pub search_count: i64,
    pub avg_results: u64,
    pub avg_time_ms: u64,
}

/// No result search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoResultSearch {
    pub query: String,
    pub search_count: i64,
    pub last_searched: DateTime<Utc>,
}

/// Performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_searches: i64,
    pub avg_time_ms: u64,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub median_time_ms: u64,
    pub p95_time_ms: u64,
    pub avg_results: u64,
}

/// Search history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryItem {
    pub id: i64,
    pub query: String,
    pub filters: Option<String>,
    pub results_count: i64,
    pub processing_time_ms: i64,
    pub created_at: DateTime<Utc>,
}

/// Top clicked torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopClickedTorrent {
    pub torrent_id: Uuid,
    pub name: String,
    pub click_count: i64,
    pub avg_position: f64,
}

/// Search trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTrend {
    pub date: chrono::NaiveDate,
    pub search_count: i64,
    pub unique_users: i64,
    pub avg_results: u64,
}

/// A/B test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub variant: String,
    pub total_searches: i64,
    pub avg_results: u64,
    pub unique_users: i64,
}

/// Filter usage statistic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterUsageStat {
    pub filters: String,
    pub usage_count: i64,
    pub avg_results: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popular_search_serialization() {
        let search = PopularSearch {
            query: "test".to_string(),
            search_count: 100,
            avg_results: 50,
            avg_time_ms: 25,
        };

        let json = serde_json::to_string(&search).unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_performance_stats_creation() {
        let stats = PerformanceStats {
            total_searches: 1000,
            avg_time_ms: 50,
            min_time_ms: 10,
            max_time_ms: 200,
            median_time_ms: 45,
            p95_time_ms: 150,
            avg_results: 25,
        };

        assert_eq!(stats.total_searches, 1000);
        assert_eq!(stats.avg_time_ms, 50);
    }
}
