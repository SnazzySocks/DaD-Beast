/// Integration tests for user service
use common::{TestContext, fixtures::TestUser};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_get_user_profile() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("profile_user_{}", ctx.test_id)),
        Some(format!("profile_{}@example.com", ctx.test_id))
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
    // 1. Make GET request to /api/v1/users/:id
    // 2. Verify response contains user profile info
    // 3. Verify sensitive data (email, password_hash) is excluded for other users
    // 4. Verify own profile includes private data

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_update_user_profile() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("update_user_{}", ctx.test_id)),
        Some(format!("update_{}@example.com", ctx.test_id))
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
    // 1. Login as user
    // 2. Make PATCH request to /api/v1/users/me with updates
    // 3. Verify profile is updated
    // 4. Verify validation works (e.g., unique email)

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_statistics() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("stats_user_{}", ctx.test_id)),
        Some(format!("stats_{}@example.com", ctx.test_id))
    );

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, is_verified, uploaded, downloaded, ratio)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        user.id,
        user.username,
        user.email,
        user.password_hash,
        user.is_active,
        user.is_verified,
        10_000_000_000i64, // 10GB uploaded
        5_000_000_000i64,  // 5GB downloaded
        2.0f64             // ratio 2.0
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Make GET request to /api/v1/users/:id/stats
    // 2. Verify response contains:
    //    - upload/download totals
    //    - ratio
    //    - number of torrents uploaded
    //    - number of active torrents seeding
    //    - bonus points

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_bonus_system() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("bonus_user_{}", ctx.test_id)),
        Some(format!("bonus_{}@example.com", ctx.test_id))
    );

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, is_verified, bonus_points)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user.id,
        user.username,
        user.email,
        user.password_hash,
        user.is_active,
        user.is_verified,
        1000i32  // 1000 bonus points
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Verify bonus points accumulate over time
    // 2. Test spending bonus points (e.g., for upload credit)
    // 3. Verify bonus points are awarded for seeding
    // 4. Test bonus point transactions

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_invitations() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("invite_user_{}", ctx.test_id)),
        Some(format!("invite_{}@example.com", ctx.test_id))
    );

    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, is_verified, invites)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user.id,
        user.username,
        user.email,
        user.password_hash,
        user.is_active,
        user.is_verified,
        5i32  // 5 invites available
    )
    .execute(&ctx.db)
    .await
    .unwrap();

    // In a real test, you would:
    // 1. Create invitation code
    // 2. Verify invitation is saved
    // 3. Verify invite count decremented
    // 4. Register new user with invitation code
    // 5. Verify invitation is marked as used
    // 6. Verify inviter gets credit

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_warnings() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("warn_user_{}", ctx.test_id)),
        Some(format!("warn_{}@example.com", ctx.test_id))
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
    // 1. Login as admin/moderator
    // 2. Issue warning to user
    // 3. Verify warning is recorded
    // 4. Verify user is notified
    // 5. Test automatic actions after X warnings

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_ban() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("ban_user_{}", ctx.test_id)),
        Some(format!("ban_{}@example.com", ctx.test_id))
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
    // 1. Login as admin
    // 2. Ban user with reason and duration
    // 3. Verify user cannot login
    // 4. Verify user's torrents are hidden
    // 5. Test temporary vs permanent bans
    // 6. Test IP bans

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_roles_and_permissions() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("role_user_{}", ctx.test_id)),
        Some(format!("role_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create user with different roles (user, uploader, moderator, admin)
    // 2. Verify each role has appropriate permissions
    // 3. Test permission checks for various actions
    // 4. Test role promotion/demotion

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_privacy_settings() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("privacy_user_{}", ctx.test_id)),
        Some(format!("privacy_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Update privacy settings (hide stats, hide torrents, etc.)
    // 2. Verify settings are saved
    // 3. Verify other users cannot see hidden information
    // 4. Verify user can still see their own data

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_notification_preferences() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("notif_user_{}", ctx.test_id)),
        Some(format!("notif_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Update notification preferences
    // 2. Verify preferences are saved
    // 3. Trigger events that generate notifications
    // 4. Verify only enabled notifications are sent

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_activity_log() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("activity_user_{}", ctx.test_id)),
        Some(format!("activity_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Perform various actions as user
    // 2. Verify actions are logged
    // 3. Get activity log
    // 4. Verify log contains correct entries

    ctx.cleanup().await.unwrap();
}
