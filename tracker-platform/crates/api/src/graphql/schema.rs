//! # GraphQL Schema
//!
//! Defines the root Query, Mutation, and Subscription types for the GraphQL API.

use async_graphql::{Context, Object, Result, Subscription};
use futures_util::Stream;
use std::pin::Pin;
use tracing::instrument;

use super::{mutations, queries, subscriptions, types, GraphQLContext};

/// Root query type
#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get API version
    #[instrument]
    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    /// Get current authenticated user
    #[instrument(skip(ctx))]
    async fn me(&self, ctx: &Context<'_>) -> Result<types::User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_current_user(gql_ctx).await
    }

    /// Get a user by ID
    #[instrument(skip(ctx))]
    async fn user(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<types::User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_user_by_id(gql_ctx, id).await
    }

    /// Get a user by username
    #[instrument(skip(ctx))]
    async fn user_by_username(
        &self,
        ctx: &Context<'_>,
        username: String,
    ) -> Result<Option<types::User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_user_by_username(gql_ctx, username).await
    }

    /// Search users
    #[instrument(skip(ctx))]
    async fn users(
        &self,
        ctx: &Context<'_>,
        query: Option<String>,
        first: Option<i32>,
        after: Option<String>,
    ) -> Result<types::UserConnection> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::search_users(gql_ctx, query, first, after).await
    }

    /// Get a torrent by ID
    #[instrument(skip(ctx))]
    async fn torrent(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<types::Torrent>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_torrent_by_id(gql_ctx, id).await
    }

    /// Get a torrent by info hash
    #[instrument(skip(ctx))]
    async fn torrent_by_info_hash(
        &self,
        ctx: &Context<'_>,
        info_hash: String,
    ) -> Result<Option<types::Torrent>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_torrent_by_info_hash(gql_ctx, info_hash).await
    }

    /// Search torrents
    #[instrument(skip(ctx))]
    async fn torrents(
        &self,
        ctx: &Context<'_>,
        query: Option<String>,
        category: Option<String>,
        first: Option<i32>,
        after: Option<String>,
    ) -> Result<types::TorrentConnection> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::search_torrents(gql_ctx, query, category, first, after).await
    }

    /// Get latest torrents
    #[instrument(skip(ctx))]
    async fn latest_torrents(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
    ) -> Result<Vec<types::Torrent>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_latest_torrents(gql_ctx, limit).await
    }

    /// Get trending torrents
    #[instrument(skip(ctx))]
    async fn trending_torrents(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
    ) -> Result<Vec<types::Torrent>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_trending_torrents(gql_ctx, limit).await
    }

    /// Get a forum by ID
    #[instrument(skip(ctx))]
    async fn forum(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<types::Forum>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_forum_by_id(gql_ctx, id).await
    }

    /// Get all forums
    #[instrument(skip(ctx))]
    async fn forums(&self, ctx: &Context<'_>) -> Result<Vec<types::Forum>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_all_forums(gql_ctx).await
    }

    /// Get a topic by ID
    #[instrument(skip(ctx))]
    async fn topic(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<types::Topic>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_topic_by_id(gql_ctx, id).await
    }

    /// Get platform statistics
    #[instrument(skip(ctx))]
    async fn statistics(&self, ctx: &Context<'_>) -> Result<types::PlatformStatistics> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        queries::get_platform_statistics(gql_ctx).await
    }
}

/// Root mutation type
#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Upload a new torrent
    #[instrument(skip(ctx))]
    async fn upload_torrent(
        &self,
        ctx: &Context<'_>,
        input: types::UploadTorrentInput,
    ) -> Result<types::Torrent> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::upload_torrent(gql_ctx, input).await
    }

    /// Update torrent metadata
    #[instrument(skip(ctx))]
    async fn update_torrent(
        &self,
        ctx: &Context<'_>,
        id: uuid::Uuid,
        input: types::UpdateTorrentInput,
    ) -> Result<types::Torrent> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::update_torrent(gql_ctx, id, input).await
    }

    /// Delete a torrent
    #[instrument(skip(ctx))]
    async fn delete_torrent(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::delete_torrent(gql_ctx, id).await
    }

    /// Update user profile
    #[instrument(skip(ctx))]
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        input: types::UpdateProfileInput,
    ) -> Result<types::User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::update_profile(gql_ctx, input).await
    }

    /// Create a forum topic
    #[instrument(skip(ctx))]
    async fn create_topic(
        &self,
        ctx: &Context<'_>,
        input: types::CreateTopicInput,
    ) -> Result<types::Topic> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::create_topic(gql_ctx, input).await
    }

    /// Post a reply to a topic
    #[instrument(skip(ctx))]
    async fn post_reply(
        &self,
        ctx: &Context<'_>,
        input: types::PostReplyInput,
    ) -> Result<types::Post> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::post_reply(gql_ctx, input).await
    }

    /// Send a private message
    #[instrument(skip(ctx))]
    async fn send_message(
        &self,
        ctx: &Context<'_>,
        input: types::SendMessageInput,
    ) -> Result<types::Message> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::send_message(gql_ctx, input).await
    }

    /// Follow a user
    #[instrument(skip(ctx))]
    async fn follow_user(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::follow_user(gql_ctx, user_id).await
    }

    /// Unfollow a user
    #[instrument(skip(ctx))]
    async fn unfollow_user(&self, ctx: &Context<'_>, user_id: uuid::Uuid) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::unfollow_user(gql_ctx, user_id).await
    }

    /// Add torrent to favorites
    #[instrument(skip(ctx))]
    async fn add_favorite(&self, ctx: &Context<'_>, torrent_id: uuid::Uuid) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::add_favorite(gql_ctx, torrent_id).await
    }

    /// Remove torrent from favorites
    #[instrument(skip(ctx))]
    async fn remove_favorite(&self, ctx: &Context<'_>, torrent_id: uuid::Uuid) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        mutations::remove_favorite(gql_ctx, torrent_id).await
    }
}

/// Root subscription type for real-time updates
#[derive(Default)]
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to new torrents
    #[instrument(skip(self, ctx))]
    async fn torrent_added(
        &self,
        ctx: &Context<'_>,
        category: Option<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = types::Torrent> + Send>>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        subscriptions::subscribe_torrent_added(gql_ctx, category).await
    }

    /// Subscribe to new messages
    #[instrument(skip(self, ctx))]
    async fn message_received(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = types::Message> + Send>>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        subscriptions::subscribe_message_received(gql_ctx).await
    }

    /// Subscribe to torrent updates
    #[instrument(skip(self, ctx))]
    async fn torrent_updated(
        &self,
        ctx: &Context<'_>,
        torrent_id: uuid::Uuid,
    ) -> Result<Pin<Box<dyn Stream<Item = types::Torrent> + Send>>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        subscriptions::subscribe_torrent_updated(gql_ctx, torrent_id).await
    }

    /// Subscribe to new posts in a topic
    #[instrument(skip(self, ctx))]
    async fn topic_posts(
        &self,
        ctx: &Context<'_>,
        topic_id: uuid::Uuid,
    ) -> Result<Pin<Box<dyn Stream<Item = types::Post> + Send>>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        subscriptions::subscribe_topic_posts(gql_ctx, topic_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_root_version() {
        let query = QueryRoot;
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}
