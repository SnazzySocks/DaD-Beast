/// Integration tests for BitTorrent tracker service
use common::{TestContext, fixtures::{TestUser, TestTorrent, TestPeer}};

mod common;

#[tokio::test]
async fn test_announce_new_peer() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user and torrent
    let user = TestUser::new(
        Some(format!("tracker_user_{}", ctx.test_id)),
        Some(format!("tracker_{}@example.com", ctx.test_id))
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

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. Make GET request to /tracker/announce with:
    //    - info_hash
    //    - peer_id
    //    - port
    //    - uploaded, downloaded, left
    //    - event=started
    // 2. Verify response is valid bencoded dictionary
    // 3. Verify peer is registered in database/cache
    // 4. Verify response contains interval and peer list

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_update_peer() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user and torrent
    let user = TestUser::new(
        Some(format!("update_user_{}", ctx.test_id)),
        Some(format!("update_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. First announce to register peer
    // 2. Make second announce with updated stats (uploaded, downloaded)
    // 3. Verify peer stats are updated
    // 4. Verify response reflects new stats

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_complete_event() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("complete_user_{}", ctx.test_id)),
        Some(format!("complete_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let mut peer = TestPeer::leecher();

    // In a real test, you would:
    // 1. Announce with event=started as leecher
    // 2. Announce with event=completed, left=0
    // 3. Verify peer is now marked as seeder
    // 4. Verify torrent's seeder count incremented
    // 5. Verify torrent's download count incremented

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_stopped_event() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("stopped_user_{}", ctx.test_id)),
        Some(format!("stopped_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. Announce with event=started
    // 2. Announce with event=stopped
    // 3. Verify peer is removed from swarm
    // 4. Verify response indicates success

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_scrape_single_torrent() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("scrape_user_{}", ctx.test_id)),
        Some(format!("scrape_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);

    // In a real test, you would:
    // 1. Add some peers to the torrent
    // 2. Make GET request to /tracker/scrape?info_hash=<hash>
    // 3. Verify response is bencoded dictionary
    // 4. Verify response contains correct stats:
    //    - complete (seeders)
    //    - incomplete (leechers)
    //    - downloaded (download count)

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_scrape_multiple_torrents() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("multiscrape_user_{}", ctx.test_id)),
        Some(format!("multiscrape_{}@example.com", ctx.test_id))
    );

    let torrent1 = TestTorrent::new(user.id);
    let torrent2 = TestTorrent::new(user.id);

    // In a real test, you would:
    // 1. Make GET request to /tracker/scrape with multiple info_hash params
    // 2. Verify response contains stats for all requested torrents
    // 3. Verify each torrent has correct stats

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_with_invalid_info_hash() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Make announce request with invalid/non-existent info_hash
    // 2. Verify response is error (failure reason in bencoded response)
    // 3. Verify appropriate error message

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_compact_response() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("compact_user_{}", ctx.test_id)),
        Some(format!("compact_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. Make announce request with compact=1
    // 2. Verify response uses compact peer format (6-byte binary)
    // 3. Verify peers can be correctly decoded

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_numwant_parameter() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("numwant_user_{}", ctx.test_id)),
        Some(format!("numwant_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);

    // Add multiple peers
    // In a real test, you would:
    // 1. Add 100 peers to a torrent
    // 2. Make announce request with numwant=20
    // 3. Verify response contains exactly 20 peers
    // 4. Make another request with numwant=0
    // 5. Verify response contains no peers

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_announce_rate_limiting() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("ratelimit_user_{}", ctx.test_id)),
        Some(format!("ratelimit_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. Make multiple rapid announce requests
    // 2. Verify rate limiting is enforced
    // 3. Verify error response when rate limit exceeded

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_peer_timeout() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("timeout_user_{}", ctx.test_id)),
        Some(format!("timeout_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);
    let peer = TestPeer::new();

    // In a real test, you would:
    // 1. Add a peer with announce
    // 2. Wait for peer timeout period (e.g., 30 minutes)
    // 3. Verify peer is automatically removed
    // 4. Verify torrent stats are updated

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_ipv6_peer_support() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("ipv6_user_{}", ctx.test_id)),
        Some(format!("ipv6_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);

    // In a real test, you would:
    // 1. Make announce from IPv6 address
    // 2. Verify peer is registered with IPv6 address
    // 3. Verify IPv6 peers are returned to IPv6 clients
    // 4. Verify separation of IPv4 and IPv6 swarms if configured

    ctx.cleanup().await.unwrap();
}
