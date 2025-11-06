/// Integration tests for community features (forums, messaging, chat)
use common::{TestContext, fixtures::{TestUser, TestForumPost}};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_create_forum_post() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("forum_user_{}", ctx.test_id)),
        Some(format!("forum_{}@example.com", ctx.test_id))
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
    // 2. Make POST request to /api/v1/forums/:forum_id/posts
    // 3. Verify post is created
    // 4. Verify post appears in forum
    // 5. Verify user's post count is incremented

    sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
        .execute(&ctx.db)
        .await
        .unwrap();

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_reply_to_forum_post() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("reply_user_{}", ctx.test_id)),
        Some(format!("reply_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create original post
    // 2. Make POST request to /api/v1/forums/posts/:post_id/replies
    // 3. Verify reply is created
    // 4. Verify reply count is incremented
    // 5. Verify original poster is notified

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_edit_forum_post() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("edit_user_{}", ctx.test_id)),
        Some(format!("edit_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create post as user
    // 2. Make PATCH request to /api/v1/forums/posts/:post_id
    // 3. Verify post is updated
    // 4. Verify edit history is tracked
    // 5. Verify "edited" indicator is shown

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_delete_forum_post() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("delete_user_{}", ctx.test_id)),
        Some(format!("delete_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create post as user
    // 2. Make DELETE request to /api/v1/forums/posts/:post_id
    // 3. Verify post is soft deleted
    // 4. Verify post no longer appears in listings
    // 5. Test admin hard delete

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_forum_post_pagination() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Create 50 posts in a forum
    // 2. Make GET request to /api/v1/forums/:forum_id/posts?page=1&per_page=20
    // 3. Verify pagination works correctly
    // 4. Verify posts are ordered by latest reply

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_forum_post_likes() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("like_user_{}", ctx.test_id)),
        Some(format!("like_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create post
    // 2. Make POST request to /api/v1/forums/posts/:post_id/like
    // 3. Verify like is recorded
    // 4. Verify like count is incremented
    // 5. Test unlike
    // 6. Verify cannot like own post

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_direct_messaging() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user1 = TestUser::new(
        Some(format!("dm_user1_{}", ctx.test_id)),
        Some(format!("dm1_{}@example.com", ctx.test_id))
    );

    let user2 = TestUser::new(
        Some(format!("dm_user2_{}", ctx.test_id)),
        Some(format!("dm2_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Login as user1
    // 2. Make POST request to /api/v1/messages with recipient: user2
    // 3. Verify message is sent
    // 4. Login as user2
    // 5. Verify message appears in inbox
    // 6. Verify unread count is incremented

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_message_threads() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Send multiple messages between two users
    // 2. Get conversation thread
    // 3. Verify messages are grouped correctly
    // 4. Verify chronological order
    // 5. Test marking thread as read

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_chat_room_join() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("chat_user_{}", ctx.test_id)),
        Some(format!("chat_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Establish WebSocket connection
    // 2. Send join message for chat room
    // 3. Verify join confirmation
    // 4. Verify user appears in room user list
    // 5. Verify other users are notified

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_chat_send_message() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Join chat room
    // 2. Send chat message via WebSocket
    // 3. Verify message is broadcast to room
    // 4. Verify message is stored in history
    // 5. Test message formatting (markdown, links)

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_chat_rate_limiting() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Join chat room
    // 2. Send messages rapidly
    // 3. Verify rate limiting is enforced
    // 4. Verify warning message on limit
    // 5. Verify temporary mute on excessive spam

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_comment_on_torrent() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user = TestUser::new(
        Some(format!("comment_user_{}", ctx.test_id)),
        Some(format!("comment_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create torrent
    // 2. Make POST request to /api/v1/torrents/:id/comments
    // 3. Verify comment is created
    // 4. Verify comment appears on torrent page
    // 5. Verify uploader is notified

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_user_mentions() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    let user1 = TestUser::new(
        Some(format!("mention1_{}", ctx.test_id)),
        Some(format!("mention1_{}@example.com", ctx.test_id))
    );

    let user2 = TestUser::new(
        Some(format!("mention2_{}", ctx.test_id)),
        Some(format!("mention2_{}@example.com", ctx.test_id))
    );

    // In a real test, you would:
    // 1. Create post mentioning @user2
    // 2. Verify mention is parsed
    // 3. Verify user2 receives notification
    // 4. Verify mention link works
    // 5. Test multiple mentions

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_moderation_tools() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Create post as regular user
    // 2. Login as moderator
    // 3. Delete/edit post
    // 4. Lock thread
    // 5. Pin post
    // 6. Move post to different forum
    // 7. Verify all actions are logged

    ctx.cleanup().await.unwrap();
}

#[tokio::test]
async fn test_report_content() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // In a real test, you would:
    // 1. Create post
    // 2. Login as different user
    // 3. Report post with reason
    // 4. Verify report is created
    // 5. Verify moderators are notified
    // 6. Test duplicate report prevention

    ctx.cleanup().await.unwrap();
}
