# Unified Tracker Platform

A modern, scalable BitTorrent tracker platform built with Rust, featuring a comprehensive REST API, GraphQL interface, and advanced community features.

## Architecture

The platform is built using a modular monolith architecture with the following components:

### Core Services
- **Auth Service** - User authentication, JWT tokens, 2FA
- **Tracker Service** - BitTorrent announce/scrape protocol
- **Torrent Service** - Torrent management and metadata
- **User Service** - User profiles and management
- **Search Service** - Full-text search with Meilisearch
- **Media Service** - Image and file uploads
- **Community Service** - Forums, comments, and social features
- **API Service** - REST API and GraphQL endpoints

### Infrastructure
- **PostgreSQL 17** - Primary database
- **Redis 7.4** - Caching and session storage
- **Meilisearch 1.10+** - Search engine
- **Prometheus** - Metrics collection
- **Grafana** - Metrics visualization

## Quick Start

### Prerequisites
- Docker and Docker Compose
- Rust 1.75+ (for local development)

### Using Docker Compose (Recommended)

1. Clone the repository:
```bash
git clone <repository-url>
cd tracker-platform
```

2. Copy the environment file:
```bash
cp .env.example .env
```

3. **IMPORTANT**: Edit `.env` and change the `JWT_SECRET`:
```bash
# Generate a secure secret (at least 32 characters)
export JWT_SECRET=$(openssl rand -base64 32)
sed -i "s/change-me-in-production-min-32-characters-long-secret-key/$JWT_SECRET/g" .env
```

4. Start all services:
```bash
docker-compose up -d
```

5. Check service health:
```bash
curl http://localhost:8080/health
```

### Local Development

1. Install dependencies:
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev libpq-dev libsasl2-dev libzstd-dev cmake

# macOS
brew install postgresql openssl cmake
```

2. Start infrastructure services:
```bash
docker-compose up -d postgres redis meilisearch
```

3. Set up environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. Run migrations:
```bash
cargo install sqlx-cli
sqlx migrate run
```

5. Run the application:
```bash
cargo run --bin tracker-platform
```

## Application Structure

```
tracker-platform/
├── app/                    # Main application binary
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── config.rs      # Configuration management
│   │   ├── state.rs       # Application state
│   │   ├── routes.rs      # Route definitions
│   │   ├── middleware.rs  # HTTP middleware
│   │   ├── telemetry.rs   # Observability
│   │   └── shutdown.rs    # Graceful shutdown
│   └── Cargo.toml
├── crates/                 # Service crates
│   ├── shared/            # Shared utilities
│   ├── auth/              # Authentication
│   ├── tracker/           # BitTorrent tracker
│   ├── torrent/           # Torrent management
│   ├── user/              # User management
│   ├── search/            # Search functionality
│   ├── media/             # Media handling
│   ├── community/         # Community features
│   └── api/               # API endpoints
├── migrations/            # Database migrations
├── config/                # Configuration files
│   ├── prometheus.yml     # Prometheus config
│   └── grafana/           # Grafana dashboards
├── docker-compose.yml     # Docker services
├── Dockerfile             # Multi-stage build
└── .env.example           # Environment template
```

## API Endpoints

### Health Check
- `GET /health` - Overall health status
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe

### Metrics
- `GET /metrics` - Prometheus metrics

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login
- `POST /api/v1/auth/logout` - Logout
- `POST /api/v1/auth/refresh` - Refresh token
- `POST /api/v1/auth/enable-2fa` - Enable 2FA
- `POST /api/v1/auth/verify-2fa` - Verify 2FA

### Users
- `GET /api/v1/users/me` - Get current user
- `GET /api/v1/users/:id` - Get user by ID
- `GET /api/v1/users/:id/stats` - Get user statistics

### Torrents
- `GET /api/v1/torrents` - List torrents
- `POST /api/v1/torrents` - Upload torrent
- `GET /api/v1/torrents/:id` - Get torrent details
- `GET /api/v1/torrents/:id/download` - Download torrent file
- `GET /api/v1/torrents/:id/peers` - Get peer list

### Search
- `GET /api/v1/search/torrents` - Search torrents
- `GET /api/v1/search/users` - Search users
- `GET /api/v1/search/suggest` - Search suggestions

### GraphQL
- `POST /graphql` - GraphQL endpoint
- `GET /graphql/playground` - GraphQL playground

### Tracker (BitTorrent Protocol)
- `GET /tracker/announce` - BitTorrent announce
- `GET /tracker/scrape` - BitTorrent scrape

## Configuration

Configuration is loaded from multiple sources in order of precedence:

1. Environment variables (prefix: `APP__`)
2. Config files (`config/default.toml`, `config/{environment}.toml`)
3. `.env` file
4. Default values

### Key Configuration Options

#### Server
```bash
APP__SERVER__HOST=0.0.0.0
APP__SERVER__PORT=8080
```

#### Database
```bash
APP__DATABASE__URL=postgresql://user:pass@host:5432/db
APP__DATABASE__MAX_CONNECTIONS=20
```

#### Redis
```bash
APP__REDIS__URL=redis://:password@host:6379
```

#### Authentication
```bash
APP__AUTH__JWT_SECRET=your-secret-key-min-32-chars
APP__AUTH__JWT_EXPIRATION_HOURS=24
```

#### Telemetry
```bash
APP__TELEMETRY__LOG_LEVEL=info
APP__TELEMETRY__LOG_FORMAT=pretty  # or json
APP__TELEMETRY__METRICS_ENABLED=true
```

## Monitoring

### Prometheus Metrics

The application exposes Prometheus metrics on port 9090:

- HTTP request metrics (count, duration)
- Database query metrics
- Cache metrics (hits, misses)
- Tracker metrics (announces, scrapes, peers)
- Custom business metrics

Access metrics: http://localhost:9090/metrics

### Grafana Dashboards

Grafana is available at http://localhost:3001 (default credentials: admin/admin)

Preconfigured dashboards include:
- Application overview
- HTTP requests
- Database performance
- Tracker statistics

## Development

### Running Tests
```bash
cargo test
```

### Running with Hot Reload
```bash
cargo install cargo-watch
cargo watch -x 'run --bin tracker-platform'
```

### Building for Release
```bash
cargo build --release
```

### Database Migrations

Create a new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

Revert last migration:
```bash
sqlx migrate revert
```

## Deployment

### Production Checklist

- [ ] Change `JWT_SECRET` to a secure random value (32+ characters)
- [ ] Update `POSTGRES_PASSWORD` in docker-compose.yml
- [ ] Update `redis` password in docker-compose.yml
- [ ] Set `MEILI_MASTER_KEY` to a secure value
- [ ] Set `APP__TELEMETRY__ENVIRONMENT=production`
- [ ] Set `APP__TELEMETRY__LOG_FORMAT=json`
- [ ] Configure CORS allowed origins
- [ ] Set up SSL/TLS certificates
- [ ] Configure backup strategy for PostgreSQL
- [ ] Set up log aggregation
- [ ] Configure alerting in Prometheus

### Docker Deployment

Build and deploy:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Kubernetes Deployment

(Kubernetes manifests would be added separately)

## Troubleshooting

### Database Connection Issues
```bash
# Check PostgreSQL is running
docker-compose ps postgres

# View PostgreSQL logs
docker-compose logs postgres

# Test connection
psql postgresql://tracker:tracker_password@localhost:5432/tracker
```

### Redis Connection Issues
```bash
# Check Redis is running
docker-compose ps redis

# Test connection
redis-cli -h localhost -p 6379 -a redis_password ping
```

### Application Logs
```bash
# View application logs
docker-compose logs -f app

# View all logs
docker-compose logs -f
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

MIT License - see LICENSE file for details
