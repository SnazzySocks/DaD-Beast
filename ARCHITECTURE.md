# Unified Tracker Platform - Architecture Document

## Project Overview

This project represents a next-generation private BitTorrent tracker platform that combines the best features from three proven tracker systems:

- **Gazelle** - Legacy PHP tracker with excellent music metadata and permission systems
- **Ocelot** - High-performance C++ BitTorrent tracker with exceptional scalability
- **Unit3d** - Modern Laravel-based tracker with comprehensive features and modern UI

## Analysis Summary

### Gazelle Analysis
- **Size**: 1,000,000+ lines, 633 PHP files
- **Key Strengths**:
  - Sophisticated multi-tier caching system with versioning
  - Comprehensive permission hierarchy (20+ user classes)
  - Advanced music metadata and artist database
  - Sphinx-based full-text search
  - Tag voting and community categorization
  - Complete audit trails for compliance
  - Paranoia/privacy control system

### Ocelot Analysis
- **Size**: 3,233 lines of C++
- **Key Strengths**:
  - Event-driven architecture with libev
  - Database write batching (hundreds of updates → single query)
  - Lock-free atomic statistics
  - Peer selection algorithms (round-robin seeder distribution)
  - Two-tier update strategy (heavy vs. light)
  - Graceful shutdown and signal-based reload
  - Exceptional performance (1000s announces/second)

### Unit3d Analysis
- **Size**: 116 models, 363 migrations, 80+ controllers
- **Key Strengths**:
  - Modern Laravel 12 with PHP 8.4+ strict typing
  - Flexible rule-based bonus/reward system
  - Comprehensive moderation tools (71 staff controllers)
  - Multi-factor authentication (TOTP, WebAuthn)
  - Real-time features (Livewire, Socket.io)
  - Meilisearch integration for advanced search
  - Media integration (TMDB, IGDB, IMDB, TVDB)
  - Security-first design with comprehensive headers
  - Complete API structure

## Proposed Architecture

### Technology Stack (Bleeding-Edge)

#### Backend Services
- **Language**: Rust (performance + memory safety)
- **Web Framework**: Axum (async, type-safe routing)
- **Database**: PostgreSQL 17 with TimescaleDB extension
- **Cache**: Redis 7.4 with RedisJSON module
- **Search**: Meilisearch 1.10+
- **Message Queue**: Apache Kafka (event streaming)
- **Service Communication**: gRPC with Protocol Buffers

#### Frontend
- **Framework**: SvelteKit 2.0 with TypeScript
- **Styling**: TailwindCSS 4 with dynamic theming
- **Components**: Web Components for reusability
- **Real-time**: Native WebSockets (replacing Socket.io)
- **State Management**: Svelte stores + GraphQL client

#### Infrastructure
- **Containerization**: Docker with multi-stage builds
- **Orchestration**: Kubernetes with Helm charts
- **API Gateway**: Traefik v3
- **Observability**: Prometheus + Grafana + OpenTelemetry
- **Secrets**: HashiCorp Vault
- **CI/CD**: GitHub Actions / GitLab CI

### Microservices Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway (Traefik)                     │
│                   GraphQL + REST + WebSocket                 │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │  Auth   │          │ Tracker │          │ Torrent │
   │ Service │          │ Service │          │ Service │
   │ (Rust)  │          │ (Rust)  │          │ (Rust)  │
   └────┬────┘          └────┬────┘          └────┬────┘
        │                    │                     │
        │         ┌──────────┼──────────┐         │
        │         │          │          │         │
   ┌────▼────┐   │    ┌────▼────┐ ┌───▼─────┐   │
   │  User   │   │    │ Search  │ │  Media  │   │
   │ Service │   │    │ Service │ │ Scraper │   │
   │ (Rust)  │   │    │ (Rust)  │ │ Service │   │
   └────┬────┘   │    └────┬────┘ └───┬─────┘   │
        │         │         │          │         │
        └─────────┴─────────┴──────────┴─────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
   │PostgreSQL│      │  Redis  │      │  Kafka  │
   │   17    │      │  7.4    │      │ Events  │
   └─────────┘      └─────────┘      └─────────┘
```

### Service Responsibilities

1. **Auth Service**
   - User authentication (JWT, OAuth2, WebAuthn)
   - Two-factor authentication
   - Session management
   - API key generation
   - Permission verification

2. **Tracker Service** (High-performance, Ocelot-inspired)
   - BitTorrent announce protocol
   - Peer tracking and management
   - IPv4/IPv6 support
   - UDP announce support
   - Peer selection algorithms
   - Real-time statistics

3. **Torrent Service**
   - Torrent upload/management
   - Metadata extraction
   - File validation
   - Moderation workflow
   - Category/tag management
   - Duplicate detection

4. **User Service**
   - User profiles
   - Privacy controls (Gazelle paranoia system)
   - Statistics tracking
   - Achievement/badge system
   - Bonus point management
   - Donation tracking

5. **Search Service**
   - Meilisearch integration
   - Advanced filtering
   - Faceted search
   - Autocomplete
   - Search analytics

6. **Media Scraper Service**
   - TMDB API integration
   - IGDB API integration
   - IMDB/TVDB/MAL scraping
   - Metadata enrichment
   - Background job processing

### Data Flow Patterns

#### Write Operations (Event-Driven)
```
Client Request → Service → Kafka Event → Multiple Consumers
                    ↓
                Database Write (async)
                    ↓
                Cache Invalidation
```

#### Read Operations (CQRS)
```
Client Request → Service → Check Redis Cache
                              ↓ (miss)
                    Query PostgreSQL Read Replica
                              ↓
                    Update Redis Cache
                              ↓
                    Return Response
```

## Cherry-Picked Features

### From Gazelle (1-8)
1. ✅ Permission hierarchy with 20+ user classes
2. ✅ Artist database with aliasing
3. ✅ Multi-tier caching with versioning
4. ✅ Tag voting system
5. ✅ Comprehensive audit trails
6. ✅ Multi-currency donation support
7. ✅ Granular privacy controls
8. ✅ Collages/curated collections

### From Ocelot (9-16)
9. ✅ Event-driven architecture pattern
10. ✅ Database write batching
11. ✅ Peer selection algorithms
12. ✅ Two-tier update strategy
13. ✅ Atomic statistics
14. ✅ Graceful shutdown patterns
15. ✅ BitTorrent protocol compliance
16. ✅ Smart memory management

### From Unit3d (17-28)
17. ✅ Flexible rule-based bonus system
18. ✅ Modern multi-factor authentication
19. ✅ Comprehensive moderation tools
20. ✅ Media integration architecture
21. ✅ Real-time features
22. ✅ Meilisearch integration
23. ✅ Playlist system
24. ✅ Ticket/support system
25. ✅ Security headers implementation
26. ✅ RESTful API structure
27. ✅ Request/bounty system
28. ✅ Three-tier freeleech management

## Implementation Priorities

### Phase 1: Core Infrastructure (MVP)
- [ ] Set up Rust workspace with Axum
- [ ] PostgreSQL schema design
- [ ] Redis caching layer
- [ ] Basic authentication service
- [ ] BitTorrent tracker service (Ocelot patterns)
- [ ] Basic torrent management
- [ ] Simple web UI with SvelteKit

### Phase 2: Essential Features
- [ ] User management with permissions
- [ ] Search integration (Meilisearch)
- [ ] Media metadata scraping
- [ ] Bonus/reward system
- [ ] Basic moderation tools
- [ ] Forum system

### Phase 3: Advanced Features
- [ ] Real-time chat/messaging
- [ ] Advanced analytics
- [ ] Recommendation engine
- [ ] Mobile app (React Native)
- [ ] API marketplace
- [ ] Plugin system

### Phase 4: Scale & Optimization
- [ ] Kubernetes deployment
- [ ] Multi-region replication
- [ ] Advanced caching strategies
- [ ] Performance optimization
- [ ] Load testing
- [ ] Security hardening

## Key Architectural Decisions

### Why Rust?
- Memory safety without garbage collection
- Performance comparable to C++ (Ocelot-level)
- Modern language features (traits, enums, pattern matching)
- Excellent async ecosystem (Tokio, Axum)
- Strong type system prevents entire classes of bugs
- Growing ecosystem for web services

### Why Microservices?
- Independent scaling of components
- Technology flexibility per service
- Fault isolation (tracker failure doesn't affect auth)
- Team autonomy (parallel development)
- Easier to maintain and test

### Why PostgreSQL over MySQL?
- Superior JSON/JSONB support
- TimescaleDB for time-series peer data
- Better full-text search (if Meilisearch unavailable)
- Advanced indexing options (GiST, GIN)
- Strong ACID compliance
- Better extension ecosystem

### Why Kafka?
- Event sourcing for complete audit trail
- Service decoupling
- Replay capability for debugging
- Scalable event streaming
- Multiple consumers per event

### Why GraphQL Gateway?
- Single API endpoint for frontend
- Client-specified data fetching (no over/under-fetching)
- Real-time subscriptions
- Strong typing with schema
- API versioning handled gracefully

## Security Considerations

### Authentication Layers
1. JWT tokens (short-lived access tokens)
2. Refresh tokens (HTTP-only cookies)
3. API keys (for external integrations)
4. Passkeys (BitTorrent client authentication)
5. OAuth2 providers (Discord, GitHub, Google)
6. WebAuthn (hardware keys)

### Data Protection
- Encryption at rest (PostgreSQL TDE)
- Encryption in transit (TLS 1.3)
- Field-level encryption for sensitive data
- Vault for secrets management
- Regular security audits

### Rate Limiting
- Per-user limits (prevent abuse)
- Per-IP limits (DDoS protection)
- Per-endpoint limits (resource protection)
- Distributed rate limiting (Redis-based)

### Compliance
- GDPR (data export, deletion, consent)
- DMCA (takedown procedures)
- Privacy policy enforcement
- Audit logging (immutable)

## Performance Targets

### Tracker Service
- **Throughput**: 10,000+ announces/second
- **Latency**: <10ms p99 response time
- **Concurrency**: 100,000+ simultaneous peers
- **Availability**: 99.99% uptime

### Web Application
- **Page Load**: <1s Time to Interactive
- **API Response**: <100ms p99
- **Search**: <50ms query time
- **Real-time**: <100ms message delivery

### Database
- **Write Throughput**: 1,000+ transactions/second
- **Read Latency**: <5ms p99
- **Replication Lag**: <1s
- **Backup Window**: <1 hour

## Monitoring & Observability

### Metrics (Prometheus)
- Request rate, error rate, duration (RED)
- CPU, memory, disk, network (USE)
- Business metrics (signups, uploads, downloads)
- Custom metrics per service

### Logging (Loki)
- Structured JSON logs
- Correlation IDs across services
- Log levels (debug, info, warn, error)
- Long-term retention

### Tracing (OpenTelemetry)
- Distributed tracing across services
- Performance profiling
- Bottleneck identification
- Service dependency mapping

### Alerting
- PagerDuty for critical incidents
- Slack for warnings
- Email for non-urgent notifications
- Runbooks for common issues

## Development Workflow

### Local Development
```bash
# Start all services with docker-compose
docker-compose up -d

# Run specific service in dev mode
cd services/tracker
cargo watch -x run

# Frontend dev server
cd frontend
npm run dev
```

### Testing Strategy
- **Unit Tests**: Rust (cargo test), TypeScript (Vitest)
- **Integration Tests**: Testcontainers for DB/Redis
- **E2E Tests**: Playwright
- **Load Tests**: k6
- **Security Tests**: OWASP ZAP

### CI/CD Pipeline
1. **Commit** → Lint + Format checks
2. **Push** → Unit tests + Build
3. **PR** → Integration tests + Security scan
4. **Merge** → Deploy to staging
5. **Tag** → Deploy to production

## Questions & Decisions Needed

Before implementation begins, the following need to be decided:

1. **Deployment Target**: Self-hosted, Cloud (AWS/GCP/Azure), or Hybrid?
2. **Feature Priority**: Which of the 105 recommendations to implement first?
3. **Authentication Strategy**: Which auth methods to support in MVP?
4. **Budget for External Services**: TMDB, IGDB, Meilisearch Cloud, etc.
5. **MVP Scope**: Full platform or phased approach?
6. **Team Size**: Solo project or team? (affects architecture complexity)
7. **Timeline**: 3 months? 6 months? 1 year?
8. **License**: Open source (AGPL like Unit3d) or proprietary?

## References

- Source repositories cloned in `source-projects/` (gitignored)
- Gazelle: PHP-based tracker framework (What.CD legacy)
- Ocelot: C++ BitTorrent tracker (high-performance)
- Unit3d: Laravel 12 modern tracker platform

## Next Steps

1. Await user decisions on questions above
2. Create initial Rust workspace structure
3. Design database schema
4. Implement auth service (MVP)
5. Implement tracker service (MVP)
6. Basic frontend with SvelteKit
7. Deploy locally for testing

---

**Document Version**: 1.0
**Last Updated**: 2025-11-05
**Author**: Claude (Systems Architect)
