# API Crate

Unified API layer providing both GraphQL and REST endpoints for the tracker platform.

## Overview

This crate serves as the main entry point for all client requests to the tracker platform. It provides a comprehensive API with both GraphQL and RESTful endpoints, complete with authentication, rate limiting, webhooks, and OpenAPI documentation.

## Features

### GraphQL API
- **Full-featured Schema**: Queries, mutations, and subscriptions
- **DataLoaders**: Efficient batching and caching to prevent N+1 queries
- **Real-time Updates**: WebSocket-based subscriptions for live data
- **Type-safe**: Strong typing with async-graphql
- **Playground**: Interactive GraphQL playground for development

### REST API
- **Versioned Endpoints**: All endpoints are versioned (e.g., `/api/v1/`)
- **OpenAPI Documentation**: Auto-generated with Swagger UI, RapiDoc, and ReDoc
- **Pagination**: Cursor-based and offset-based pagination support
- **Filtering & Sorting**: Comprehensive query parameters

### Security
- **JWT Authentication**: Token-based authentication
- **Rate Limiting**: Token bucket algorithm with Redis backend
- **HMAC Signatures**: Webhook signature verification
- **CORS**: Configurable cross-origin resource sharing

### Infrastructure
- **Webhooks**: Event-driven system with retry logic
- **Observability**: Built-in tracing and logging
- **Health Checks**: Readiness and liveness endpoints
- **Connection Pooling**: Efficient database and Redis connection management

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        API Layer                            │
├─────────────────────┬───────────────────────────────────────┤
│   GraphQL           │           REST                        │
│  - Queries          │  - GET /api/v1/torrents               │
│  - Mutations        │  - POST /api/v1/torrents/upload       │
│  - Subscriptions    │  - GET /api/v1/users/:id              │
└─────────────────────┴───────────────────────────────────────┘
              │                        │
              ├────────────────────────┤
              ▼                        ▼
┌────────────────────────────────────────────────────────────┐
│     Domain Services (torrent, user, community, etc.)       │
└────────────────────────────────────────────────────────────┘
```

## File Structure

```
crates/api/
├── Cargo.toml                    # Dependencies (108 lines)
├── README.md                     # This file
└── src/
    ├── lib.rs                    # Main library and service setup (373 lines)
    ├── graphql/
    │   ├── mod.rs               # GraphQL setup and dataloaders (312 lines)
    │   ├── schema.rs            # Root query, mutation, subscription (313 lines)
    │   ├── types.rs             # GraphQL types and DTOs (459 lines)
    │   ├── queries.rs           # Query resolvers (377 lines)
    │   ├── mutations.rs         # Mutation resolvers (450 lines)
    │   └── subscriptions.rs     # Real-time subscriptions (379 lines)
    ├── rest/
    │   ├── mod.rs               # REST API setup (201 lines)
    │   ├── torrents.rs          # Torrent endpoints (447 lines)
    │   └── users.rs             # User endpoints (311 lines)
    ├── webhooks.rs              # Webhook system (388 lines)
    ├── rate_limit.rs            # Rate limiting (374 lines)
    └── openapi.rs               # OpenAPI documentation (216 lines)

Total: 4,708 lines of Rust code
```

## Usage

### Starting the API Service

```rust
use api::{ApiConfig, ApiService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the API
    let config = ApiConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        enable_graphql_playground: true,
        enable_swagger_ui: true,
        enable_cors: true,
        rate_limit_per_minute: 60,
        jwt_secret: "your-secret-key".to_string(),
        database_url: "postgres://localhost/tracker".to_string(),
        redis_url: "redis://localhost/".to_string(),
    };

    // Create and run the service
    let service = ApiService::new(config).await?;
    service.run().await?;

    Ok(())
}
```

### GraphQL Endpoints

- **GraphQL Query/Mutation**: `POST /graphql`
- **GraphQL Playground**: `GET /graphql` (if enabled)
- **GraphQL Subscriptions**: `WS /graphql/ws`

Example GraphQL query:
```graphql
query {
  torrents(first: 10, category: "movies") {
    edges {
      node {
        id
        name
        seeders
        leechers
        uploader {
          username
        }
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

### REST Endpoints

#### Torrents
- `GET /api/v1/torrents` - List torrents with filtering
- `GET /api/v1/torrents/:id` - Get torrent by ID
- `POST /api/v1/torrents` - Upload new torrent (authenticated)
- `PATCH /api/v1/torrents/:id` - Update torrent (authenticated)
- `GET /api/v1/torrents/:id/download` - Download torrent file (authenticated)

#### Users
- `GET /api/v1/users/:id` - Get user profile
- `PATCH /api/v1/users/:id` - Update user profile (authenticated)
- `GET /api/v1/users/:id/stats` - Get user statistics
- `GET /api/v1/users/:id/torrents` - Get user's torrents

#### Health Checks
- `GET /health` - Simple health check
- `GET /ready` - Readiness check (verifies DB and Redis)

### Documentation Endpoints

- **Swagger UI**: `GET /swagger-ui`
- **RapiDoc**: `GET /rapidoc`
- **ReDoc**: `GET /redoc`
- **OpenAPI JSON**: `GET /api-docs/openapi.json`

## GraphQL Features

### Queries

```graphql
# Get current user
query {
  me {
    id
    username
    email
    ratio
  }
}

# Search torrents
query {
  torrents(query: "ubuntu", first: 20) {
    edges {
      node {
        id
        name
        size
        seeders
      }
    }
    totalCount
  }
}

# Get platform statistics
query {
  statistics {
    totalUsers
    totalTorrents
    totalSeeders
    totalLeechers
  }
}
```

### Mutations

```graphql
# Upload a torrent
mutation {
  uploadTorrent(input: {
    name: "Ubuntu 22.04 LTS"
    category: "software"
    infoHash: "..."
    torrentFile: [...]
  }) {
    id
    name
    createdAt
  }
}

# Update profile
mutation {
  updateProfile(input: {
    email: "new@example.com"
  }) {
    id
    email
  }
}

# Send a message
mutation {
  sendMessage(input: {
    recipientId: "..."
    subject: "Hello"
    content: "Message content"
  }) {
    id
    createdAt
  }
}
```

### Subscriptions

```graphql
# Subscribe to new torrents
subscription {
  torrentAdded(category: "movies") {
    id
    name
    uploader {
      username
    }
  }
}

# Subscribe to messages
subscription {
  messageReceived {
    id
    sender {
      username
    }
    subject
  }
}
```

## Rate Limiting

The API implements rate limiting using a token bucket algorithm with Redis:

- **Authenticated users**: 120 requests/minute
- **Unauthenticated (IP-based)**: 30 requests/minute
- **API keys**: 1000 requests/minute

Rate limit headers are returned with every response:
```
X-RateLimit-Limit: 120
X-RateLimit-Remaining: 119
X-RateLimit-Reset: 1234567890
```

## Webhooks

### Registering a Webhook

```rust
use api::webhooks::{WebhookManager, WebhookEvent};

let manager = WebhookManager::new(db_pool, redis_client);

manager.register_webhook(
    user_id,
    "https://your-server.com/webhooks".to_string(),
    vec![
        WebhookEvent::TorrentAdded,
        WebhookEvent::MessageReceived,
    ],
    Some("your-secret".to_string()),
).await?;
```

### Webhook Payload

```json
{
  "event": "torrent_added",
  "timestamp": "2024-01-01T00:00:00Z",
  "data": {
    "id": "...",
    "name": "Ubuntu 22.04 LTS",
    "category": "software"
  }
}
```

Webhooks include an `X-Webhook-Signature` header with HMAC-SHA256 signature for verification.

## Testing

```bash
# Run tests
cargo test -p api

# Run with logging
RUST_LOG=debug cargo test -p api

# Run specific test
cargo test -p api test_name
```

## Dependencies

Key dependencies:
- **axum**: Web framework
- **async-graphql**: GraphQL implementation
- **sqlx**: Database access
- **redis**: Caching and rate limiting
- **utoipa**: OpenAPI documentation
- **tokio**: Async runtime
- **tower-http**: HTTP middleware

## Development

### Running Locally

```bash
# Start dependencies
docker-compose up -d postgres redis

# Run the API service
cargo run -p api

# Access endpoints
# GraphQL Playground: http://localhost:8080/graphql
# Swagger UI: http://localhost:8080/swagger-ui
# Health check: http://localhost:8080/health
```

### Environment Variables

```env
DATABASE_URL=postgres://user:pass@localhost/tracker
REDIS_URL=redis://localhost/
JWT_SECRET=your-secret-key
API_HOST=0.0.0.0
API_PORT=8080
```

## Performance Considerations

1. **DataLoaders**: GraphQL queries use DataLoaders to batch and cache database queries
2. **Connection Pooling**: Database connections are pooled (max 50 connections)
3. **Redis Caching**: Frequently accessed data is cached in Redis
4. **Rate Limiting**: Prevents abuse and ensures fair resource allocation
5. **Compression**: HTTP responses are compressed with gzip

## Security

1. **JWT Authentication**: All authenticated endpoints require valid JWT tokens
2. **Rate Limiting**: Prevents brute force and DoS attacks
3. **Input Validation**: All inputs are validated before processing
4. **SQL Injection Prevention**: Uses parameterized queries
5. **CORS**: Configurable cross-origin policies
6. **Webhook Signatures**: HMAC verification for webhook deliveries

## Future Enhancements

- [ ] GraphQL query complexity analysis
- [ ] Field-level permissions
- [ ] API key management
- [ ] Request/response caching
- [ ] Metrics export (Prometheus)
- [ ] GraphQL persisted queries
- [ ] Batch operations
- [ ] File upload via multipart

## License

MIT
