//! # GraphQL Queries
//!
//! Query resolvers for fetching data from the database.

use async_graphql::Result;
use redis::AsyncCommands;
use tracing::instrument;

use super::{types::*, GraphQLContext};

/// Get the current authenticated user
#[instrument(skip(ctx))]
pub async fn get_current_user(ctx: &GraphQLContext) -> Result<User> {
    let user_id = ctx.require_auth()?;

    let user = ctx
        .user_loader
        .load_one(user_id)
        .await?
        .ok_or_else(|| async_graphql::Error::new("User not found"))?;

    Ok(user)
}

/// Get a user by ID
#[instrument(skip(ctx))]
pub async fn get_user_by_id(ctx: &GraphQLContext, id: uuid::Uuid) -> Result<Option<User>> {
    let user = ctx.user_loader.load_one(id).await?;
    Ok(user)
}

/// Get a user by username
#[instrument(skip(ctx))]
pub async fn get_user_by_username(
    ctx: &GraphQLContext,
    username: String,
) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
         passkey, is_active, is_verified FROM users WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(&ctx.db_pool)
    .await?;

    Ok(user)
}

/// Search users with pagination
#[instrument(skip(ctx))]
pub async fn search_users(
    ctx: &GraphQLContext,
    query: Option<String>,
    first: Option<i32>,
    after: Option<String>,
) -> Result<UserConnection> {
    let limit = first.unwrap_or(10).min(100) as i64;
    let offset = after.and_then(|c| c.parse::<i64>().ok()).unwrap_or(0);

    // Build query
    let (sql, count_sql) = if let Some(q) = &query {
        (
            format!(
                "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
                 passkey, is_active, is_verified FROM users
                 WHERE username ILIKE $1 OR email ILIKE $1
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            ),
            format!("SELECT COUNT(*) FROM users WHERE username ILIKE $1 OR email ILIKE $1"),
        )
    } else {
        (
            "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
             passkey, is_active, is_verified FROM users
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"
                .to_string(),
            "SELECT COUNT(*) FROM users".to_string(),
        )
    };

    // Fetch users
    let users: Vec<User> = if let Some(q) = &query {
        let search_pattern = format!("%{}%", q);
        sqlx::query_as(&sql)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&ctx.db_pool)
            .await?
    } else {
        sqlx::query_as(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&ctx.db_pool)
            .await?
    };

    // Get total count
    let total_count: i64 = if let Some(q) = &query {
        let search_pattern = format!("%{}%", q);
        sqlx::query_scalar(&count_sql)
            .bind(&search_pattern)
            .fetch_one(&ctx.db_pool)
            .await?
    } else {
        sqlx::query_scalar(&count_sql)
            .fetch_one(&ctx.db_pool)
            .await?
    };

    // Create edges
    let edges: Vec<UserEdge> = users
        .into_iter()
        .enumerate()
        .map(|(i, user)| UserEdge {
            cursor: (offset + i as i64).to_string(),
            node: user,
        })
        .collect();

    // Create page info
    let has_next_page = offset + limit < total_count;
    let has_previous_page = offset > 0;
    let start_cursor = edges.first().map(|e| e.cursor.clone());
    let end_cursor = edges.last().map(|e| e.cursor.clone());

    Ok(UserConnection {
        edges,
        page_info: PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor,
            end_cursor,
        },
        total_count,
    })
}

/// Get a torrent by ID
#[instrument(skip(ctx))]
pub async fn get_torrent_by_id(ctx: &GraphQLContext, id: uuid::Uuid) -> Result<Option<Torrent>> {
    let torrent = ctx.torrent_loader.load_one(id).await?;
    Ok(torrent)
}

/// Get a torrent by info hash
#[instrument(skip(ctx))]
pub async fn get_torrent_by_info_hash(
    ctx: &GraphQLContext,
    info_hash: String,
) -> Result<Option<Torrent>> {
    let torrent = sqlx::query_as::<_, Torrent>(
        "SELECT * FROM torrents WHERE info_hash = $1",
    )
    .bind(&info_hash)
    .fetch_optional(&ctx.db_pool)
    .await?;

    Ok(torrent)
}

/// Search torrents with pagination
#[instrument(skip(ctx))]
pub async fn search_torrents(
    ctx: &GraphQLContext,
    query: Option<String>,
    category: Option<String>,
    first: Option<i32>,
    after: Option<String>,
) -> Result<TorrentConnection> {
    let limit = first.unwrap_or(10).min(100) as i64;
    let offset = after.and_then(|c| c.parse::<i64>().ok()).unwrap_or(0);

    // Build query based on filters
    let mut conditions = vec![];
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>> = vec![];
    let mut param_count = 1;

    if let Some(q) = &query {
        conditions.push(format!("name ILIKE ${}", param_count));
        param_count += 1;
    }

    if let Some(cat) = &category {
        conditions.push(format!("category = ${}", param_count));
        param_count += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT * FROM torrents {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause, param_count, param_count + 1
    );

    let count_sql = format!("SELECT COUNT(*) FROM torrents {}", where_clause);

    // Fetch torrents (simplified - in production use a query builder)
    let torrents = if query.is_some() || category.is_some() {
        // For simplicity, using direct query
        let mut q = sqlx::query_as::<_, Torrent>(&sql);
        if let Some(query_str) = &query {
            q = q.bind(format!("%{}%", query_str));
        }
        if let Some(cat) = &category {
            q = q.bind(cat);
        }
        q.bind(limit).bind(offset).fetch_all(&ctx.db_pool).await?
    } else {
        sqlx::query_as::<_, Torrent>(&sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&ctx.db_pool)
            .await?
    };

    // Get total count
    let total_count: i64 = if query.is_some() || category.is_some() {
        let mut q = sqlx::query_scalar(&count_sql);
        if let Some(query_str) = &query {
            q = q.bind(format!("%{}%", query_str));
        }
        if let Some(cat) = &category {
            q = q.bind(cat);
        }
        q.fetch_one(&ctx.db_pool).await?
    } else {
        sqlx::query_scalar(&count_sql)
            .fetch_one(&ctx.db_pool)
            .await?
    };

    // Create edges
    let edges: Vec<TorrentEdge> = torrents
        .into_iter()
        .enumerate()
        .map(|(i, torrent)| TorrentEdge {
            cursor: (offset + i as i64).to_string(),
            node: torrent,
        })
        .collect();

    // Create page info
    let has_next_page = offset + limit < total_count;
    let has_previous_page = offset > 0;
    let start_cursor = edges.first().map(|e| e.cursor.clone());
    let end_cursor = edges.last().map(|e| e.cursor.clone());

    Ok(TorrentConnection {
        edges,
        page_info: PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor,
            end_cursor,
        },
        total_count,
    })
}

/// Get latest torrents
#[instrument(skip(ctx))]
pub async fn get_latest_torrents(ctx: &GraphQLContext, limit: Option<i32>) -> Result<Vec<Torrent>> {
    let limit = limit.unwrap_or(10).min(100);

    let torrents = sqlx::query_as::<_, Torrent>(
        "SELECT * FROM torrents ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&ctx.db_pool)
    .await?;

    Ok(torrents)
}

/// Get trending torrents (most downloaded in last 24 hours)
#[instrument(skip(ctx))]
pub async fn get_trending_torrents(
    ctx: &GraphQLContext,
    limit: Option<i32>,
) -> Result<Vec<Torrent>> {
    let limit = limit.unwrap_or(10).min(100);

    let torrents = sqlx::query_as::<_, Torrent>(
        "SELECT t.* FROM torrents t
         WHERE t.created_at > NOW() - INTERVAL '24 hours'
         ORDER BY t.times_completed DESC, t.seeders DESC
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&ctx.db_pool)
    .await?;

    Ok(torrents)
}

/// Get a forum by ID
#[instrument(skip(ctx))]
pub async fn get_forum_by_id(ctx: &GraphQLContext, id: uuid::Uuid) -> Result<Option<Forum>> {
    let forum = ctx.forum_loader.load_one(id).await?;
    Ok(forum)
}

/// Get all forums
#[instrument(skip(ctx))]
pub async fn get_all_forums(ctx: &GraphQLContext) -> Result<Vec<Forum>> {
    let forums = sqlx::query_as::<_, Forum>(
        "SELECT * FROM forums ORDER BY position ASC",
    )
    .fetch_all(&ctx.db_pool)
    .await?;

    Ok(forums)
}

/// Get a topic by ID
#[instrument(skip(ctx))]
pub async fn get_topic_by_id(ctx: &GraphQLContext, id: uuid::Uuid) -> Result<Option<Topic>> {
    let topic = sqlx::query_as::<_, Topic>(
        "SELECT * FROM topics WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&ctx.db_pool)
    .await?;

    Ok(topic)
}

/// Get platform statistics
#[instrument(skip(ctx))]
pub async fn get_platform_statistics(ctx: &GraphQLContext) -> Result<PlatformStatistics> {
    // Get total users
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&ctx.db_pool)
        .await?;

    // Get total torrents
    let total_torrents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM torrents")
        .fetch_one(&ctx.db_pool)
        .await?;

    // Get seeders and leechers from peers table (or aggregate from torrents)
    let (total_seeders, total_leechers): (i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(seeders), 0) as total_seeders,
            COALESCE(SUM(leechers), 0) as total_leechers
         FROM torrents",
    )
    .fetch_one(&ctx.db_pool)
    .await?;

    let total_peers = total_seeders + total_leechers;

    Ok(PlatformStatistics {
        total_users,
        total_torrents,
        total_seeders,
        total_leechers,
        total_peers,
    })
}

/// Fallback dad jokes for when the API is unavailable
const FALLBACK_JOKES: &[(&str, &str)] = &[
    ("fallback-1", "Why don't scientists trust atoms? Because they make up everything!"),
    ("fallback-2", "I only know 25 letters of the alphabet. I don't know y."),
    ("fallback-3", "What do you call a fake noodle? An impasta!"),
    ("fallback-4", "Why did the scarecrow win an award? He was outstanding in his field!"),
    ("fallback-5", "Why don't eggs tell jokes? They'd crack each other up!"),
    ("fallback-6", "What do you call a bear with no teeth? A gummy bear!"),
    ("fallback-7", "Why couldn't the bicycle stand up by itself? It was two tired!"),
    ("fallback-8", "What did the ocean say to the beach? Nothing, it just waved!"),
    ("fallback-9", "How do you organize a space party? You planet!"),
    ("fallback-10", "Why did the math book look so sad? Because it had too many problems!"),
];

/// Get a random dad joke with caching and fallback
#[instrument(skip(ctx))]
pub async fn get_dad_joke(ctx: &GraphQLContext) -> Result<DadJoke> {
    const CACHE_KEY: &str = "dad_jokes:cache";
    const CACHE_DURATION: i64 = 300;

    let mut redis_conn = match ctx.redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => Some(conn),
        Err(e) => {
            tracing::warn!("Failed to connect to Redis for dad joke cache: {}", e);
            None
        }
    };

    if let Some(ref mut conn) = redis_conn {
        if let Ok(cached_jokes) = conn.lrange::<&str, Vec<String>>(CACHE_KEY, 0, -1).await {
            if !cached_jokes.is_empty() {
                use rand::seq::SliceRandom;
                use rand::thread_rng;

                if let Some(joke_json) = cached_jokes.choose(&mut thread_rng()) {
                    if let Ok(joke) = serde_json::from_str::<DadJoke>(joke_json) {
                        tracing::debug!("Returning cached dad joke: {}", joke.id);
                        return Ok(joke);
                    }
                }
            }
        }
    }

    match fetch_joke_from_api().await {
        Ok(joke) => {
            if let Some(ref mut conn) = redis_conn {
                let joke_json = serde_json::to_string(&joke).unwrap_or_default();
                let _: Result<(), redis::RedisError> = conn.rpush(CACHE_KEY, &joke_json).await;
                let _: Result<(), redis::RedisError> = conn.expire(CACHE_KEY, CACHE_DURATION).await;
            }
            Ok(joke)
        }
        Err(e) => {
            tracing::warn!("Failed to fetch joke from API, using fallback: {}", e);
            use rand::seq::SliceRandom;
            use rand::thread_rng;

            let (id, joke_text) = FALLBACK_JOKES
                .choose(&mut thread_rng())
                .unwrap_or(&FALLBACK_JOKES[0]);

            Ok(DadJoke {
                id: id.to_string(),
                joke: joke_text.to_string(),
            })
        }
    }
}

/// Fetch a joke from the icanhazdadjoke API
async fn fetch_joke_from_api() -> Result<DadJoke, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    #[derive(serde::Deserialize)]
    struct ApiResponse {
        id: String,
        joke: String,
    }

    let response = client
        .get("https://icanhazdadjoke.com/")
        .header("Accept", "application/json")
        .header("User-Agent", "Unified Tracker Platform (https://github.com/yourorg/tracker)")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()).into());
    }

    let api_response: ApiResponse = response.json().await?;

    Ok(DadJoke {
        id: api_response.id,
        joke: api_response.joke,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_jokes_not_empty() {
        assert!(!FALLBACK_JOKES.is_empty());
        assert_eq!(FALLBACK_JOKES.len(), 10);
    }

    #[test]
    fn test_fallback_jokes_structure() {
        for (id, joke) in FALLBACK_JOKES {
            assert!(!id.is_empty(), "Joke ID should not be empty");
            assert!(!joke.is_empty(), "Joke text should not be empty");
            assert!(id.starts_with("fallback-"), "Joke ID should start with 'fallback-'");
        }
    }

    #[test]
    fn test_dad_joke_type() {
        let joke = DadJoke {
            id: "test-123".to_string(),
            joke: "Why don't scientists trust atoms? Because they make up everything!".to_string(),
        };

        assert_eq!(joke.id, "test-123");
        assert!(!joke.joke.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_joke_from_api_structure() {
        match fetch_joke_from_api().await {
            Ok(joke) => {
                assert!(!joke.id.is_empty(), "API joke ID should not be empty");
                assert!(!joke.joke.is_empty(), "API joke text should not be empty");
            }
            Err(_) => {
                println!("API fetch failed (expected in CI/test environments without internet)");
            }
        }
    }
}
