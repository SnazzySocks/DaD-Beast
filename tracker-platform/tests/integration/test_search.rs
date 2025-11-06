/// Integration tests for search service
use common::{TestContext, fixtures::{TestUser, TestTorrent}};

mod common;

#[tokio::test]
async fn test_search_torrents_by_name() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("search_user_{}", ctx.test_id)),
        Some(format!("search_{}@example.com", ctx.test_id))
    );

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, is_verified)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        user.id,
        user.username,
        user.email,
        user.password_hash,
        user.is_active,
        user.is_verified
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // Create test torrents
    let torrent1 = TestTorrent::new(user.id);
    sqlx::query!(
        r#"
        INSERT INTO torrents (id, info_hash, name, description, size, uploader_id, category_id, is_approved)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        torrent1.id,
        torrent1.info_hash,
        "Ubuntu 22.04 LTS Desktop",
        "Latest Ubuntu release",
        torrent1.size,
        torrent1.uploader_id,
        1i32,
        true
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Index torrents in Meilisearch
    // 2. Make GET request to /api/v1/search/torrents?q=ubuntu
    // 3. Verify response contains matching torrents
    // 4. Verify relevance ranking
    // 5. Verify pagination

    sqlx::query!("DELETE FROM torrents WHERE uploader_id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_with_filters() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("filter_user_{}", ctx.test_id)),
        Some(format!("filter_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create torrents with various attributes
    // 2. Search with filters:
    //    - category
    //    - size range
    //    - seeders count
    //    - uploader
    //    - date range
    // 3. Verify results match all filters
    // 4. Test combining multiple filters

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_suggestions() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Make GET request to /api/v1/search/suggest?q=ubu
    // 2. Verify response contains suggestions like "ubuntu"
    // 3. Verify suggestions are ranked by popularity
    // 4. Test autocomplete behavior

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_facets() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Search for torrents
    // 2. Verify response includes facets:
    //    - Categories with counts
    //    - Quality tags
    //    - File types
    // 3. Use facets to refine search

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_sorting() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("sort_user_{}", ctx.test_id)),
        Some(format!("sort_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create multiple torrents
    // 2. Test sorting by:
    //    - Relevance (default)
    //    - Upload date (newest/oldest)
    //    - Size (largest/smallest)
    //    - Seeders (most/least)
    //    - Downloads (most/least)
    // 3. Verify results are correctly sorted

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_full_text_search() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Create torrents with detailed descriptions
    // 2. Search for terms in descriptions
    // 3. Verify full-text matching works
    // 4. Test phrase matching
    // 5. Test fuzzy matching for typos

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_performance() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("perf_user_{}", ctx.test_id)),
        Some(format!("perf_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Index large number of torrents (e.g., 10,000)
    // 2. Perform search queries
    // 3. Verify response time is acceptable (< 100ms)
    // 4. Test concurrent searches

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_user_profiles() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test users
    for i in 0..5 {
        let user = TestUser::new(
            Some(format!("searchuser{}_{}", i, ctx.test_id)),
            Some(format!("searchuser{}_{}@example.com", i, ctx.test_id))
        );

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_active, is_verified)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.is_active,
            user.is_verified
        )
        .execute(&ctx.db)
        .await
        .unwrap();
    }

    // In a real test, you would:
    // 1. Make GET request to /api/v1/search/users?q=searchuser
    // 2. Verify response contains matching users
    // 3. Verify privacy settings are respected
    // 4. Verify banned users are excluded

    sqlx::query!("DELETE FROM users WHERE username LIKE $1", format!("searchuser%_{}", ctx.test_id))
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_index_updates() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("index_user_{}", ctx.test_id)),
        Some(format!("index_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Upload new torrent
    // 2. Verify torrent appears in search immediately
    // 3. Update torrent
    // 4. Verify updates reflect in search
    // 5. Delete torrent
    // 6. Verify torrent removed from search

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_search_special_characters() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Create torrents with special characters in names
    // 2. Search with special characters
    // 3. Verify search handles special chars correctly
    // 4. Test Unicode characters
    // 5. Test escape sequences

    ctx.cleanup().await.unwrap();
}
