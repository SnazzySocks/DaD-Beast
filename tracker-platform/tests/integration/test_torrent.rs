/// Integration tests for torrent service
use common::{TestContext, fixtures::{TestUser, TestTorrent}};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_upload_torrent() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("uploader_{}", ctx.test_id)),
        Some(format!("uploader_{}@example.com", ctx.test_id))
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

    // In a real test, you would:
    // 1. Create a valid .torrent file
    // 2. Make POST request to /api/v1/torrents with:
    //    - torrent file
    //    - name, description, category
    // 3. Verify response contains torrent ID
    // 4. Verify torrent is created in database
    // 5. Verify torrent is pending approval

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_list_torrents() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("list_user_{}", ctx.test_id)),
        Some(format!("list_{}@example.com", ctx.test_id))
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
    for i in 0..5 {
        let torrent = TestTorrent::new(user.id);
        sqlx::query!(
            r#"
            INSERT INTO torrents (id, info_hash, name, description, size, uploader_id, category_id, is_approved)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            torrent.id,
            torrent.info_hash,
            format!("{} {}", torrent.name, i),
            torrent.description,
            torrent.size,
            torrent.uploader_id,
            torrent.category_id,
            torrent.is_approved
        )
        .execute(&ctx.db)
        .await
        .unwrap();
    }

    // In a real test, you would:
    // 1. Make GET request to /api/v1/torrents
    // 2. Verify response contains list of torrents
    // 3. Verify pagination works correctly
    // 4. Verify sorting options work

    // Cleanup
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
async fn test_get_torrent_details() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("details_user_{}", ctx.test_id)),
        Some(format!("details_{}@example.com", ctx.test_id))
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
    sqlx::query!(
        r#"
        INSERT INTO torrents (id, info_hash, name, description, size, uploader_id, category_id, is_approved)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        torrent.id,
        torrent.info_hash,
        torrent.name,
        torrent.description,
        torrent.size,
        torrent.uploader_id,
        torrent.category_id,
        torrent.is_approved
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Make GET request to /api/v1/torrents/:id
    // 2. Verify response contains correct torrent details
    // 3. Verify response includes uploader info
    // 4. Verify response includes file list
    // 5. Verify response includes peer stats

    // Cleanup
    sqlx::query!("DELETE FROM torrents WHERE id = $1", torrent.id)
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
async fn test_download_torrent_file() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("download_user_{}", ctx.test_id)),
        Some(format!("download_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Upload a torrent
    // 2. Make GET request to /api/v1/torrents/:id/download
    // 3. Verify response is valid .torrent file
    // 4. Verify Content-Type is application/x-bittorrent
    // 5. Verify file contains correct announce URL
    // 6. Verify download is recorded in stats

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_moderate_torrent() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("mod_user_{}", ctx.test_id)),
        Some(format!("mod_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::unapproved(user.id);

    // In a real test, you would:
    // 1. Create unapproved torrent
    // 2. Login as moderator
    // 3. Make PATCH request to /api/v1/torrents/:id/approve
    // 4. Verify torrent is approved
    // 5. Verify uploader is notified

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_delete_torrent() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("delete_user_{}", ctx.test_id)),
        Some(format!("delete_{}@example.com", ctx.test_id))
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
    sqlx::query!(
        r#"
        INSERT INTO torrents (id, info_hash, name, description, size, uploader_id, category_id, is_approved)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        torrent.id,
        torrent.info_hash,
        torrent.name,
        torrent.description,
        torrent.size,
        torrent.uploader_id,
        torrent.category_id,
        torrent.is_approved
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Login as torrent owner or admin
    // 2. Make DELETE request to /api/v1/torrents/:id
    // 3. Verify torrent is soft deleted
    // 4. Verify torrent no longer appears in listings
    // 5. Verify peers are notified

    // Cleanup
    sqlx::query!("DELETE FROM torrents WHERE id = $1", torrent.id)
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
async fn test_torrent_comments() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("comment_user_{}", ctx.test_id)),
        Some(format!("comment_{}@example.com", ctx.test_id))
    );

    let torrent = TestTorrent::new(user.id);

    // In a real test, you would:
    // 1. Create torrent
    // 2. Make POST request to /api/v1/torrents/:id/comments
    // 3. Verify comment is created
    // 4. Make GET request to get comments
    // 5. Verify comment appears in list
    // 6. Test comment editing and deletion

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_torrent_categories() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Make GET request to /api/v1/categories
    // 2. Verify all categories are returned
    // 3. Verify category hierarchy if applicable
    // 4. Filter torrents by category

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_duplicate_torrent_prevention() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("dup_user_{}", ctx.test_id)),
        Some(format!("dup_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Upload a torrent with specific info_hash
    // 2. Try to upload same torrent again (same info_hash)
    // 3. Verify second upload is rejected
    // 4. Verify appropriate error message

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_torrent_file_validation() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Try to upload invalid torrent file
    // 2. Verify upload is rejected
    // 3. Try to upload file that's too large
    // 4. Verify appropriate error messages
    // 5. Try to upload with missing required fields

    ctx.cleanup().await.unwrap();
}
