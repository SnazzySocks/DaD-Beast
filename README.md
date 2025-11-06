# Projects-1

This repository contains multiple projects and experiments.

## Projects

### 1. Preseed Framework

A simple framework for generating Debian preseed files.

Location: `preseed-framework/`
See: `preseed-framework/README.md` for usage details.

### 2. Unified Tracker Platform

A next-generation private BitTorrent tracker platform combining the best features from Gazelle, Ocelot, and Unit3d.

**Status**: Architecture & Planning Phase
**Documentation**:
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Detailed system architecture
- [RECOMMENDATIONS.md](./RECOMMENDATIONS.md) - 105 feature recommendations

**Key Features**:
- High-performance Rust-based tracker (inspired by Ocelot)
- Modern SvelteKit frontend
- Microservices architecture
- 28 cherry-picked features from proven tracker platforms

**Technology Stack**:
- Backend: Rust with Axum, PostgreSQL 17, Redis 7.4, Kafka
- Frontend: SvelteKit 2.0, TailwindCSS 4, WebSockets
- Infrastructure: Docker, Kubernetes, Traefik, Prometheus

See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete details.
