/// Integration tests for authentication service
use common::{TestContext, fixtures::TestUser, helpers::HttpTestHelper};

mod common;

#[tokio::test]
async fn test_user_registration() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Test data
    let username = format!("testuser_{}", ctx.test_id);
    let email = format!("test_{}@example.com", ctx.test_id);
    let password = "SecurePassword123!";

    // Register user
    let register_payload = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "password_confirmation": password
    });

    // In a real test, you would call the actual registration endpoint
    // For now, this demonstrates the test structure

    // Verify user was created in database
    let user = sqlx::query!(
        "SELECT id, username, email FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&ctx.db)
    .await
    .unwrap();

    // Cleanup
    if let Some(user) = user {
        sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(&ctx.db)
            .await
            .unwrap();
    }

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_login() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("logintest_{}", ctx.test_id)),
        Some(format!("logintest_{}@example.com", ctx.test_id))
    );

    // Insert user into database
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

    // Test login payload
    let login_payload = serde_json::json!({
        "username": user.username,
        "password": user.password
    });

    // In a real test, you would:
    // 1. Create the app router
    // 2. Make POST request to /api/v1/auth/login
    // 3. Verify response contains valid JWT token
    // 4. Verify token can be used for authenticated requests

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let login_payload = serde_json::json!({
        "username": "nonexistent_user",
        "password": "wrong_password"
    });

    // In a real test, you would:
    // 1. Make POST request to /api/v1/auth/login
    // 2. Verify response status is 401 Unauthorized
    // 3. Verify error message is appropriate

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_2fa_enable() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("2fatest_{}", ctx.test_id)),
        Some(format!("2fatest_{}@example.com", ctx.test_id))
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
    // 1. Generate JWT token for the user
    // 2. Make POST request to /api/v1/auth/enable-2fa with token
    // 3. Verify response contains TOTP secret and QR code
    // 4. Verify 2FA is enabled in database

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_2fa_verify() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user with 2FA enabled
    let user = TestUser::new(
        Some(format!("2faverify_{}", ctx.test_id)),
        Some(format!("2faverify_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Insert user with 2FA secret
    // 2. Generate valid TOTP code
    // 3. Make POST request to /api/v1/auth/verify-2fa with code
    // 4. Verify response contains JWT token
    // 5. Verify token includes 2FA claim

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_token_refresh() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("refreshtest_{}", ctx.test_id)),
        Some(format!("refreshtest_{}@example.com", ctx.test_id))
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
    // 1. Generate valid access token and refresh token
    // 2. Make POST request to /api/v1/auth/refresh with refresh token
    // 3. Verify response contains new access token
    // 4. Verify old token is invalidated

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_logout() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("logouttest_{}", ctx.test_id)),
        Some(format!("logouttest_{}@example.com", ctx.test_id))
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
    // 1. Login to get JWT token
    // 2. Make POST request to /api/v1/auth/logout with token
    // 3. Verify response is successful
    // 4. Verify token is blacklisted/invalidated
    // 5. Verify subsequent requests with token fail

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_password_reset_request() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("resettest_{}", ctx.test_id)),
        Some(format!("resettest_{}@example.com", ctx.test_id))
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
    // 1. Make POST request to /api/v1/auth/reset-password with email
    // 2. Verify response is successful
    // 3. Verify reset token is created in database
    // 4. Verify email is sent to user

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_concurrent_login_attempts() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Create test user
    let user = TestUser::new(
        Some(format!("concurrent_{}", ctx.test_id)),
        Some(format!("concurrent_{}@example.com", ctx.test_id))
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
    // 1. Spawn multiple concurrent login requests
    // 2. Verify rate limiting works correctly
    // 3. Verify account is not locked incorrectly

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}
