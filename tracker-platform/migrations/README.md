# Database Schema Documentation

## Overview

This directory contains SQLx migration files for the unified tracker platform. The schema is designed to support a comprehensive BitTorrent tracker with features from Gazelle, Ocelot, and Unit3d.

**Total Migrations**: 36
**Total Tables**: 44+ (including junction tables)
**Database**: PostgreSQL 14+
**Migration Tool**: SQLx

## Migration Order

Migrations are executed in chronological order based on their timestamp prefix (YYYYMMDDHHMMSS format).

## Schema Organization

### Core User System (7 tables)

#### 1. **user_groups** - Permission Groups
- **File**: `20250105000000_create_user_groups.sql`
- **Purpose**: Role-based access control with 13 default groups
- **Key Features**:
  - Hierarchical permission levels (0-25)
  - Granular permissions (upload, download, moderate, etc.)
  - Bonus point multipliers per group
  - Invite system management
  - Pre-populated with groups: Banned, Member, Power User, Elite, Torrent Master, VIP, Uploader, Moderator, Administrator, Sysop

#### 2. **users** - Core User Accounts
- **File**: `20250105000001_create_users.sql`
- **Purpose**: User authentication and profiles
- **Key Features**:
  - UUID primary keys for security
  - Passkey for tracker announces (32-char unique)
  - Argon2id password hashing
  - Email verification workflow
  - Soft delete support
  - Activity tracking (last login, last access)
  - Invite tree tracking

#### 3. **user_sessions** - Active Sessions
- **File**: `20250105000002_create_user_sessions.sql`
- **Purpose**: JWT token management and session tracking
- **Key Features**:
  - SHA256 token hashing
  - Device fingerprinting
  - Geolocation tracking
  - Session expiration management

#### 4. **user_privacy** - Privacy Settings
- **File**: `20250105000003_create_user_privacy.sql`
- **Purpose**: Gazelle-style paranoia system
- **Key Features**:
  - 17 granular visibility settings
  - Email notification preferences
  - Digest settings
  - Online status hiding

#### 5. **user_statistics** - User Stats
- **File**: `20250105000004_create_user_statistics.sql`
- **Purpose**: Upload/download tracking and ratio management
- **Key Features**:
  - Raw and adjusted transfer stats (for freeleech)
  - Ratio calculation support
  - Bonus point balance and history
  - Torrent counts (uploaded, seeding, leeching, snatched)
  - Request statistics
  - Seedtime tracking

#### 6. **user_achievements** - Gamification
- **File**: `20250105000005_create_user_achievements.sql`
- **Purpose**: Badge and achievement system
- **Key Features**:
  - Tiered achievements (Bronze to Diamond)
  - Bonus point rewards
  - Invite rewards
  - Visibility controls

#### 7. **user_2fa** - Two-Factor Authentication
- **File**: `20250105000006_create_user_2fa.sql`
- **Purpose**: TOTP 2FA security
- **Key Features**:
  - Base32 TOTP secrets
  - Backup recovery codes
  - Failed attempt tracking

---

### Torrent System (15 tables)

#### 8. **torrent_categories** - Categories
- **File**: `20250105000007_create_torrent_categories.sql`
- **Purpose**: Hierarchical torrent organization
- **Key Features**:
  - Parent-child relationships
  - Pre-populated categories (Movies, TV, Music, Games, etc.)
  - Subcategories (HD, SD, 4K, FLAC, etc.)
  - Custom icons and colors

#### 9. **torrents** - Main Torrent Table
- **File**: `20250105000008_create_torrents.sql`
- **Purpose**: Core torrent metadata and tracking
- **Key Features**:
  - SHA1 info_hash (v1) and SHA256 (v2) support
  - Media metadata (resolution, codec, source, release group)
  - Moderation workflow
  - Freeleech/double upload flags
  - Featured/sticky flags
  - Anonymous upload support
  - Statistics (seeders, leechers, completed)
  - NFO support
  - Full-text search

#### 10. **torrent_files** - File Lists
- **File**: `20250105000009_create_torrent_files.sql`
- **Purpose**: Individual files within torrents
- **Key Features**:
  - Full file paths
  - Media file detection
  - Video metadata (duration, resolution, bitrate)

#### 11-13. **torrent_tags*** - Tag System (3 tables)
- **Files**: `20250105000010_create_torrent_tags.sql`
- **Purpose**: Community tag system with voting
- **Tables**:
  - `torrent_tags`: Tag definitions
  - `torrent_tag_assignments`: Torrent-tag relationships
  - `torrent_tag_votes`: User votes on tags
- **Key Features**:
  - User-created tags
  - Upvote/downvote system
  - Tag types (user, system, genre, quality)

#### 14. **torrent_moderation** - Moderation History
- **File**: `20250105000011_create_torrent_moderation.sql`
- **Purpose**: Complete audit trail for torrent moderation
- **Key Features**:
  - Action tracking (approved, rejected, edited, deleted)
  - Change history (JSONB)
  - Moderator notes

#### 15. **torrent_metadata** - External Metadata
- **File**: `20250105000012_create_torrent_metadata.sql`
- **Purpose**: Links to TMDB, IGDB, MusicBrainz, etc.
- **Key Features**:
  - Multiple external database IDs
  - Rich metadata (cast, directors, genres, plot)
  - Poster/backdrop images
  - Rating aggregation
  - Full JSONB storage of API responses

#### 16-18. **torrent_requests*** - Request System (3 tables)
- **Files**: `20250105000013_create_torrent_requests.sql`
- **Purpose**: User requests with bounty system
- **Tables**:
  - `torrent_requests`: Request definitions
  - `torrent_request_bounties`: Bounty contributions
  - `torrent_request_votes`: Community votes
- **Key Features**:
  - Bonus point bounties
  - Multiple contributors
  - Voting system
  - Fulfillment tracking

#### 19-21. **torrent_collections*** - Collections (3 tables)
- **Files**: `20250105000014_create_torrent_collections.sql`
- **Purpose**: Curated playlists/collages
- **Tables**:
  - `torrent_collections`: Collection definitions
  - `torrent_collection_items`: Torrents in collections
  - `torrent_collection_subscriptions`: User subscriptions
- **Key Features**:
  - Public/private collections
  - Collaborative editing
  - Ordering support
  - Staff picks

#### 22. **comments** - Comment System
- **File**: `20250105000035_create_comments.sql`
- **Purpose**: Comments on torrents, requests, collections
- **Key Features**:
  - Polymorphic commentable entities
  - Threaded discussions
  - Edit history
  - Soft delete

---

### Tracker System (4 tables)

#### 23. **peers** - Active Peers
- **File**: `20250105000015_create_peers.sql`
- **Purpose**: Real-time peer tracking
- **Key Features**:
  - Seeder/leecher identification
  - Client detection
  - Transfer statistics
  - Freeleech token tracking
  - IP/port tracking

#### 24. **announces** - Announce History
- **File**: `20250105000016_create_announces.sql`
- **Purpose**: Historical announce data
- **Key Features**:
  - Event tracking (started, completed, stopped)
  - Transfer statistics
  - Should be partitioned by time

#### 25. **peer_history** - Time-Series Data
- **File**: `20250105000017_create_peer_history.sql`
- **Purpose**: Historical peer snapshots for analytics
- **Key Features**:
  - Optimized for TimescaleDB
  - Periodic snapshots
  - Analytics-ready format

#### 26. **torrent_statistics** - Aggregated Stats
- **File**: `20250105000018_create_torrent_statistics.sql`
- **Purpose**: Denormalized torrent statistics
- **Key Features**:
  - Current seeder/leecher counts
  - Peak tracking
  - Speed averages
  - Health metrics
  - Dead torrent detection

---

### Bonus System (3 tables)

#### 27. **bonus_rules** - Earning Rules
- **File**: `20250105000019_create_bonus_rules.sql`
- **Purpose**: Configurable bonus earning system
- **Key Features**:
  - Rule-based earning (time, size, ratio)
  - JSONB conditions
  - Category restrictions
  - Size-based multipliers
  - Pre-populated default rules

#### 28. **bonus_transactions** - Transaction Ledger
- **File**: `20250105000020_create_bonus_transactions.sql`
- **Purpose**: Complete transaction history
- **Key Features**:
  - Earned/spent tracking
  - Balance snapshots
  - Source tracking
  - Admin adjustments
  - Transfer/gift support

#### 29. **freeleech_tokens** - Token Management
- **File**: `20250105000021_create_freeleech_tokens.sql`
- **Purpose**: Freeleech token activation and tracking
- **Key Features**:
  - Token expiration (24-72 hours typical)
  - Usage tracking
  - Completion detection

---

### Community System (8 tables)

#### 30. **forums** - Forum Categories
- **File**: `20250105000022_create_forums.sql`
- **Purpose**: Forum structure
- **Key Features**:
  - Hierarchical forums
  - Permission levels
  - Auto-lock settings
  - Pre-populated default forums

#### 31. **forum_topics** - Discussion Threads
- **File**: `20250105000023_create_forum_topics.sql`
- **Purpose**: Forum topics
- **Key Features**:
  - Sticky/locked flags
  - Poll support
  - View tracking
  - Subscription counts

#### 32. **forum_posts** - Posts
- **File**: `20250105000024_create_forum_posts.sql`
- **Purpose**: Individual forum posts
- **Key Features**:
  - Edit history
  - Soft delete
  - Pre-rendered HTML
  - Full-text search

#### 33. **forum_subscriptions** - Topic Following
- **File**: `20250105000025_create_forum_subscriptions.sql`
- **Purpose**: User subscriptions to topics
- **Key Features**:
  - Read tracking
  - Notification preferences

#### 34. **private_messages** - PM System
- **File**: `20250105000026_create_private_messages.sql`
- **Purpose**: User-to-user messaging
- **Key Features**:
  - Threading support
  - Independent deletion per user
  - Read receipts
  - Full-text search

#### 35-36. **chat_rooms** & **chat_messages** - Live Chat
- **Files**:
  - `20250105000027_create_chat_rooms.sql`
  - `20250105000028_create_chat_messages.sql`
- **Purpose**: Real-time chat system
- **Key Features**:
  - Room types (public, private, staff)
  - User bans per room
  - Room moderators
  - Slow mode
  - Mention system
  - Auto-cleanup of old messages

---

### Moderation System (5 tables)

#### 37. **reports** - User Reports
- **File**: `20250105000029_create_reports.sql`
- **Purpose**: Content reporting system
- **Key Features**:
  - Polymorphic reported entities
  - Priority levels
  - Assignment system
  - Evidence storage (URLs, metadata)
  - Resolution tracking

#### 38. **warnings** - User Warnings
- **File**: `20250105000030_create_warnings.sql`
- **Purpose**: Warning/infraction system
- **Key Features**:
  - Points system
  - Severity levels
  - Expiration dates
  - Acknowledgment tracking
  - Revocation support

#### 39. **bans** - User Bans
- **File**: `20250105000031_create_bans.sql`
- **Purpose**: Ban management
- **Key Features**:
  - Multiple ban types (account, IP, email, upload, download, chat, forum)
  - IP range bans (CIDR)
  - Temporary/permanent bans
  - Appeal system
  - Lift tracking

#### 40. **audit_logs** - Audit Trail
- **File**: `20250105000032_create_audit_logs.sql`
- **Purpose**: Complete system audit logging
- **Key Features**:
  - Action tracking
  - Before/after changes (JSONB)
  - Request correlation
  - Success/failure tracking
  - Should be partitioned by time

#### 41. **moderation_queue** - Mod Queue
- **File**: `20250105000033_create_moderation_queue.sql`
- **Purpose**: Pending moderation items
- **Key Features**:
  - Polymorphic items
  - Priority system
  - Assignment
  - Auto-flag support (ML confidence scores)
  - Prevents duplicate queue entries

---

### Search System (1 table)

#### 42. **search_index_queue** - Search Indexing
- **File**: `20250105000034_create_search_index_queue.sql`
- **Purpose**: Async indexing to Meilisearch
- **Key Features**:
  - Operation types (index, update, delete)
  - Retry logic
  - JSONB payload storage
  - Status tracking

---

## Key Design Patterns

### 1. **UUID Primary Keys**
Most tables use UUID primary keys for:
- Security (non-enumerable IDs)
- Distributed system support
- Better for caching

### 2. **Soft Deletes**
Many tables support soft deletion via `deleted_at` timestamp:
- users
- torrents
- comments
- forum_posts

### 3. **Polymorphic Relationships**
Several tables support multiple entity types:
- comments (commentable_type/commentable_id)
- reports (report_type/reported_entity_id)
- moderation_queue (item_type/item_id)
- audit_logs (entity_type/entity_id)

### 4. **JSONB for Flexibility**
JSONB columns for semi-structured data:
- torrent_moderation.changes
- torrent_metadata.metadata_json
- bonus_rules.conditions
- audit_logs.changes/metadata
- moderation_queue.metadata

### 5. **Array Columns**
PostgreSQL arrays for lists:
- user_2fa.backup_codes
- torrent_metadata.genres/directors/actors
- chat_rooms.allowed_user_ids/banned_user_ids
- reports.evidence_urls

### 6. **Time-Series Optimization**
Tables designed for time-series:
- peer_history (use TimescaleDB hypertables)
- announces (partition by month)
- audit_logs (partition by month)

### 7. **Denormalization for Performance**
Strategic denormalization:
- torrents.seeders/leechers (from peers count)
- user_statistics (aggregated from multiple sources)
- torrent_statistics (aggregated metrics)
- forums/forum_topics (post counts, last post)

### 8. **Full-Text Search**
GIN indexes on tsvector for PostgreSQL full-text search:
- torrents (name + description)
- forum_posts (body)
- private_messages (subject + body)
- comments (body)

### 9. **Comprehensive Indexing**
Each table includes:
- Foreign key indexes
- Common query patterns
- Partial indexes for filtered queries
- Composite indexes for complex queries
- GIN indexes for arrays and JSONB

## Performance Considerations

### Recommended Extensions
```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";  -- UUID generation
CREATE EXTENSION IF NOT EXISTS "pg_trgm";    -- Trigram search
CREATE EXTENSION IF NOT EXISTS "btree_gin";  -- GIN indexes on btrees
```

### Recommended Setup

1. **TimescaleDB** for time-series tables:
   - peer_history
   - announces (optional)

2. **Partitioning** recommendations:
   - announces: Partition by month (created_at)
   - audit_logs: Partition by month (created_at)

3. **Connection Pooling**:
   - Use PgBouncer or similar
   - Recommended pool size: 20-50 connections

4. **Vacuum Strategy**:
   - Aggressive autovacuum on high-write tables (peers, announces)

5. **Meilisearch Integration**:
   - Process search_index_queue asynchronously
   - Batch operations for efficiency

## Security Features

1. **Password Security**: Argon2id hashing (implemented in application)
2. **Passkey Generation**: Cryptographically secure 32-char keys
3. **IP Tracking**: For moderation and security
4. **Token Hashing**: SHA256 for session tokens
5. **2FA Support**: TOTP-based two-factor authentication
6. **Audit Logging**: Complete action tracking
7. **Permission System**: Fine-grained RBAC

## Data Retention

### Cleanup Recommendations

1. **chat_messages**: Respect `chat_rooms.message_retention_days`
2. **announces**: Archive monthly, keep 6-12 months
3. **peer_history**: Aggregate and archive quarterly
4. **audit_logs**: Keep 1-2 years, then archive
5. **user_sessions**: Clean expired sessions daily
6. **search_index_queue**: Clean completed items after 7 days

## Migration Usage

### Apply Migrations
```bash
# Using SQLx CLI
sqlx migrate run

# Check migration status
sqlx migrate info
```

### Rollback
SQLx migrations are typically one-way. For rollback, you'll need to:
1. Create down migrations manually
2. Use database backups for major rollbacks

### Development
```bash
# Create new migration
sqlx migrate add <name>

# This creates: migrations/<timestamp>_<name>.sql
```

## Statistics

- **Total Tables**: 44+
- **Total Indexes**: 300+
- **Foreign Keys**: 100+
- **Default Data**: 30+ rows (user groups, categories, forums, bonus rules)
- **Estimated Base Size**: ~50MB (empty schema with default data)

## License

See main project LICENSE file.

## Contributing

When adding new migrations:
1. Use descriptive names
2. Include comments explaining purpose
3. Add appropriate indexes
4. Consider soft deletes for user data
5. Use UUIDs for new entity tables
6. Document in this README
