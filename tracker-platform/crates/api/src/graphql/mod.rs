//! # GraphQL Module
//!
//! Provides a full-featured GraphQL API for the tracker platform.
//!
//! ## Features
//!
//! - **Queries**: Complex nested queries for torrents, users, forums, etc.
//! - **Mutations**: All CRUD operations
//! - **Subscriptions**: Real-time updates via WebSocket
//! - **DataLoaders**: Efficient batching and caching to prevent N+1 queries
//! - **Authentication**: User context with JWT validation
//! - **Authorization**: Field-level and object-level access control

pub mod mutations;
pub mod queries;
pub mod schema;
pub mod subscriptions;
pub mod types;

use anyhow::Result;
use async_graphql::{
    dataloader::DataLoader,
    extensions::{ApolloTracing, Logger},
    Context, EmptySubscription, Schema, SchemaBuilder,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tracing::instrument;

use crate::ApiState;
use schema::{MutationRoot, QueryRoot, SubscriptionRoot};

/// GraphQL schema type
pub type TrackerSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// GraphQL context containing user info and dataloaders
#[derive(Clone)]
pub struct GraphQLContext {
    /// Database connection pool
    pub db_pool: sqlx::PgPool,
    /// Redis client
    pub redis_client: redis::Client,
    /// Current user ID (if authenticated)
    pub user_id: Option<uuid::Uuid>,
    /// User permissions
    pub permissions: Vec<String>,
    /// Torrent dataloader
    pub torrent_loader: DataLoader<TorrentLoader>,
    /// User dataloader
    pub user_loader: DataLoader<UserLoader>,
    /// Forum dataloader
    pub forum_loader: DataLoader<ForumLoader>,
}

impl GraphQLContext {
    /// Create a new GraphQL context
    pub fn new(
        db_pool: sqlx::PgPool,
        redis_client: redis::Client,
        user_id: Option<uuid::Uuid>,
        permissions: Vec<String>,
    ) -> Self {
        // Create dataloaders
        let torrent_loader = DataLoader::new(
            TorrentLoader {
                db_pool: db_pool.clone(),
            },
            tokio::spawn,
        );

        let user_loader = DataLoader::new(
            UserLoader {
                db_pool: db_pool.clone(),
            },
            tokio::spawn,
        );

        let forum_loader = DataLoader::new(
            ForumLoader {
                db_pool: db_pool.clone(),
            },
            tokio::spawn,
        );

        Self {
            db_pool,
            redis_client,
            user_id,
            permissions,
            torrent_loader,
            user_loader,
            forum_loader,
        }
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Get current user ID or return error
    pub fn require_auth(&self) -> Result<uuid::Uuid, async_graphql::Error> {
        self.user_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))
    }
}

/// DataLoader for batch loading torrents
pub struct TorrentLoader {
    db_pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl async_graphql::dataloader::Loader<uuid::Uuid> for TorrentLoader {
    type Value = types::Torrent;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<std::collections::HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // Batch load torrents
        let torrents = sqlx::query_as::<_, types::Torrent>(
            "SELECT * FROM torrents WHERE id = ANY($1)",
        )
        .bind(keys)
        .fetch_all(&self.db_pool)
        .await
        .map_err(Arc::new)?;

        Ok(torrents.into_iter().map(|t| (t.id, t)).collect())
    }
}

/// DataLoader for batch loading users
pub struct UserLoader {
    db_pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl async_graphql::dataloader::Loader<uuid::Uuid> for UserLoader {
    type Value = types::User;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<std::collections::HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // Batch load users
        let users = sqlx::query_as::<_, types::User>(
            "SELECT id, username, email, created_at, user_class, uploaded, downloaded,
             passkey, is_active, is_verified FROM users WHERE id = ANY($1)",
        )
        .bind(keys)
        .fetch_all(&self.db_pool)
        .await
        .map_err(Arc::new)?;

        Ok(users.into_iter().map(|u| (u.id, u)).collect())
    }
}

/// DataLoader for batch loading forums
pub struct ForumLoader {
    db_pool: sqlx::PgPool,
}

#[async_trait::async_trait]
impl async_graphql::dataloader::Loader<uuid::Uuid> for ForumLoader {
    type Value = types::Forum;
    type Error = Arc<sqlx::Error>;

    async fn load(
        &self,
        keys: &[uuid::Uuid],
    ) -> Result<std::collections::HashMap<uuid::Uuid, Self::Value>, Self::Error> {
        // Batch load forums
        let forums = sqlx::query_as::<_, types::Forum>(
            "SELECT * FROM forums WHERE id = ANY($1)",
        )
        .bind(keys)
        .fetch_all(&self.db_pool)
        .await
        .map_err(Arc::new)?;

        Ok(forums.into_iter().map(|f| (f.id, f)).collect())
    }
}

/// Create the GraphQL schema with all resolvers and extensions
#[instrument(skip(db_pool, redis_client))]
pub async fn create_schema(
    db_pool: sqlx::PgPool,
    redis_client: redis::Client,
) -> Result<TrackerSchema> {
    let schema = Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        SubscriptionRoot::default(),
    )
    .data(db_pool)
    .data(redis_client)
    // Add extensions for better developer experience
    .extension(Logger)
    .extension(ApolloTracing)
    .finish();

    Ok(schema)
}

/// Configure GraphQL routes
pub fn configure_routes(app: Router<Arc<ApiState>>, state: Arc<ApiState>) -> Router<Arc<ApiState>> {
    app.route("/graphql", post(graphql_handler))
        .route("/graphql", get(graphql_playground))
        .route(
            "/graphql/ws",
            get(graphql_subscription_handler).with_state(state),
        )
}

/// GraphQL query handler
#[instrument(skip(state, req))]
async fn graphql_handler(
    State(state): State<Arc<ApiState>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // TODO: Extract user from JWT token in request headers
    let user_id = None;
    let permissions = vec![];

    // Create GraphQL context
    let ctx = GraphQLContext::new(
        state.db_pool.clone(),
        state.redis_client.clone(),
        user_id,
        permissions,
    );

    // Execute query
    state.graphql_schema.execute(req.into_inner().data(ctx)).await.into()
}

/// GraphQL playground UI
#[instrument]
async fn graphql_playground() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>GraphQL Playground</title>
    <link rel="stylesheet" href="//cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
    <link rel="shortcut icon" href="//cdn.jsdelivr.net/npm/graphql-playground-react/build/favicon.png" />
    <script src="//cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
</head>
<body>
    <div id="root"></div>
    <script>
        window.addEventListener('load', function (event) {
            GraphQLPlayground.init(document.getElementById('root'), {
                endpoint: '/graphql',
                subscriptionEndpoint: '/graphql/ws',
                settings: {
                    'request.credentials': 'include',
                }
            })
        })
    </script>
</body>
</html>
        "#,
    )
}

/// GraphQL subscription handler
async fn graphql_subscription_handler(
    State(state): State<Arc<ApiState>>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        GraphQLSubscription::new(schema::SubscriptionRoot::default()).serve(socket)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_context_permissions() {
        let db_pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let redis_client = redis::Client::open("redis://localhost/").unwrap();
        let ctx = GraphQLContext::new(
            db_pool,
            redis_client,
            Some(uuid::Uuid::new_v4()),
            vec!["read:torrents".to_string(), "write:torrents".to_string()],
        );

        assert!(ctx.has_permission("read:torrents"));
        assert!(ctx.has_permission("write:torrents"));
        assert!(!ctx.has_permission("admin:users"));
    }
}
