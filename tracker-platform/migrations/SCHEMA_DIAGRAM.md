# Database Schema Diagram

## Entity Relationship Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          CORE USER SYSTEM                            │
└─────────────────────────────────────────────────────────────────────┘

user_groups (13 pre-defined groups)
    ↓ (group_id)
users (UUID, passkey, email, password_hash)
    ├── user_sessions (JWT tokens, device fingerprinting)
    ├── user_privacy (paranoia levels, notifications)
    ├── user_statistics (ratio, bonus points, transfer stats)
    ├── user_achievements (badges, gamification)
    └── user_2fa (TOTP secrets, backup codes)


┌─────────────────────────────────────────────────────────────────────┐
│                        TORRENT SYSTEM                                │
└─────────────────────────────────────────────────────────────────────┘

torrent_categories (hierarchical, parent_id)
    ↓ (category_id)
torrents (info_hash, name, description, metadata)
    ├── torrent_files (individual files in torrent)
    ├── torrent_tags ←→ torrent_tag_assignments ←→ torrent_tag_votes
    ├── torrent_moderation (history, changes)
    ├── torrent_metadata (TMDB, IGDB, MusicBrainz)
    ├── comments (polymorphic)
    └── torrent_statistics (aggregated metrics)

torrent_requests (bounty system)
    ├── torrent_request_bounties (contributions)
    └── torrent_request_votes (community votes)

torrent_collections (playlists/collages)
    ├── torrent_collection_items (torrents in collection)
    └── torrent_collection_subscriptions (followers)


┌─────────────────────────────────────────────────────────────────────┐
│                        TRACKER SYSTEM                                │
└─────────────────────────────────────────────────────────────────────┘

peers (active seeders/leechers)
    ├── user_id → users
    └── torrent_id → torrents

announces (historical announce data)
    ├── user_id → users
    └── torrent_id → torrents

peer_history (time-series snapshots for TimescaleDB)
    ├── user_id → users
    └── torrent_id → torrents

torrent_statistics (denormalized metrics)
    └── torrent_id → torrents


┌─────────────────────────────────────────────────────────────────────┐
│                        BONUS SYSTEM                                  │
└─────────────────────────────────────────────────────────────────────┘

bonus_rules (earning conditions, JSONB)
    ↓ (bonus_rule_id)
bonus_transactions (complete ledger)
    └── user_id → users

freeleech_tokens (token activation & tracking)
    ├── user_id → users
    └── torrent_id → torrents


┌─────────────────────────────────────────────────────────────────────┐
│                        COMMUNITY SYSTEM                              │
└─────────────────────────────────────────────────────────────────────┘

forums (hierarchical categories)
    ↓ (forum_id)
forum_topics (discussion threads)
    ├── forum_subscriptions (topic followers)
    └── forum_posts (individual posts)
        └── author_id → users

private_messages (PM system)
    ├── sender_id → users
    └── recipient_id → users

chat_rooms (real-time chat definitions)
    ↓ (room_id)
chat_messages (chat message history)
    └── user_id → users


┌─────────────────────────────────────────────────────────────────────┐
│                        MODERATION SYSTEM                             │
└─────────────────────────────────────────────────────────────────────┘

reports (user-submitted reports)
    ├── reporter_id → users
    ├── assigned_to → users
    └── resolved_by → users

warnings (infractions with points)
    ├── user_id → users
    └── issued_by → users

bans (temporary/permanent bans)
    ├── user_id → users
    └── banned_by → users

moderation_queue (pending items)
    ├── submitted_by → users
    ├── assigned_to → users
    └── resolved_by → users

audit_logs (complete action history)
    └── user_id → users


┌─────────────────────────────────────────────────────────────────────┐
│                        SEARCH SYSTEM                                 │
└─────────────────────────────────────────────────────────────────────┘

search_index_queue (async Meilisearch indexing)
    └── entity_id (polymorphic)


┌─────────────────────────────────────────────────────────────────────┐
│                        SHARED FEATURES                               │
└─────────────────────────────────────────────────────────────────────┘

comments (polymorphic: torrents, requests, collections)
    ├── author_id → users
    └── commentable_id (polymorphic reference)
```

## Core Relationships

### User-Centric Relationships

```
                        ┌──────────┐
                        │  users   │
                        └────┬─────┘
                             │
         ┌───────────────────┼──────────────────┐
         ↓                   ↓                  ↓
    ┌─────────┐        ┌──────────┐      ┌──────────┐
    │torrents │        │  peers   │      │  forum   │
    │(upload) │        │(seeding) │      │  posts   │
    └─────────┘        └──────────┘      └──────────┘
         ↓                   ↓                  ↓
    ┌─────────┐        ┌──────────┐      ┌──────────┐
    │ torrent │        │announces │      │ private  │
    │  files  │        │(history) │      │ messages │
    └─────────┘        └──────────┘      └──────────┘
```

### Torrent Lifecycle

```
1. UPLOAD
   user → torrents (moderation_status: pending)
        → moderation_queue (item_type: torrent)
        → torrent_files (parsed from .torrent)

2. MODERATION
   moderator → moderation_queue (status: in_progress)
             → torrent_moderation (action: approved/rejected)
             → torrents (moderation_status: approved)
             → search_index_queue (operation: index)

3. TRACKING
   client → announces (event: started)
         → peers (is_seeder: false)
         → peer_history (time-series)
         → torrent_statistics (updated aggregates)

4. COMPLETION
   client → announces (event: completed)
         → peers (is_seeder: true, left_bytes: 0)
         → user_statistics (torrents_snatched++)
         → torrent_statistics (times_completed++)

5. BONUS EARNING
   cron → peers (active seeders)
       → bonus_rules (evaluate conditions)
       → bonus_transactions (amount: earned)
       → user_statistics (seedbonus++)
```

### Request-to-Fill Flow

```
1. REQUEST
   user → torrent_requests (status: pending)
        → torrent_request_bounties (initial_bounty)
        → user_statistics (bounty_spent++)

2. VOTING & BOUNTY
   users → torrent_request_votes (votes++)
         → torrent_request_bounties (additional contributions)
         → torrent_requests (total_bounty++)

3. FILL
   user → torrents (upload matching content)
        → torrent_requests (status: filled, filled_torrent_id)
        → bonus_transactions (bounty payout to filler)
        → user_statistics (bounty_earned++, bounty_spent--)
```

### Moderation Workflow

```
1. REPORT
   user → reports (status: pending, report_type: torrent)
        → moderation_queue (queue_type: investigation)

2. ASSIGNMENT
   moderator → moderation_queue (assigned_to: moderator_id)
             → reports (status: investigating)

3. ACTION
   moderator → warnings (user_id: offender, points: 3)
             → audit_logs (action: warn_user)
             → reports (status: resolved, action_taken: warned)

4. BAN (if points threshold reached)
   system → bans (user_id: offender, ban_type: account)
          → users (is_banned: true)
          → audit_logs (action: ban_user)
```

## Table Size Estimates (1 year, 10k users, 50k torrents)

| Table | Estimated Rows | Est. Size | Growth Rate |
|-------|----------------|-----------|-------------|
| **users** | 10,000 | 5 MB | Slow |
| **user_statistics** | 10,000 | 3 MB | Slow |
| **user_sessions** | 15,000 | 8 MB | Medium |
| **torrents** | 50,000 | 50 MB | Medium |
| **torrent_files** | 500,000 | 100 MB | Medium |
| **peers** | 100,000 | 40 MB | High (volatile) |
| **announces** | 500M+ | 150 GB | Very High* |
| **peer_history** | 100M+ | 50 GB | Very High* |
| **bonus_transactions** | 5M | 2 GB | High |
| **forum_posts** | 100,000 | 50 MB | Medium |
| **chat_messages** | 1M | 500 MB | High* |
| **audit_logs** | 10M | 5 GB | High* |
| **comments** | 50,000 | 15 MB | Medium |

\* *Requires partitioning and/or regular archival*

## Index Strategy by Query Pattern

### High-Read Tables
**torrents, users, torrent_categories**
- Multiple covering indexes
- Partial indexes for common filters (is_active, is_freeleech)
- Full-text search indexes

### High-Write Tables
**peers, announces, audit_logs**
- Minimal indexes (FK + time-based)
- Partition by time
- Regular archival

### Junction Tables
**torrent_tag_assignments, forum_subscriptions**
- Composite indexes on both FKs
- Unique constraints to prevent duplicates

## Performance Optimization Checklist

### Indexes
- ✅ All foreign keys indexed
- ✅ Common WHERE clauses indexed
- ✅ Partial indexes for filtered queries
- ✅ Composite indexes for multi-column queries
- ✅ GIN indexes for arrays/JSONB
- ✅ Full-text search indexes

### Constraints
- ✅ Foreign keys with appropriate ON DELETE
- ✅ Unique constraints
- ✅ Check constraints
- ✅ NOT NULL where appropriate

### Denormalization
- ✅ Statistics tables for aggregates
- ✅ Counter caches (post counts, etc.)
- ✅ Last post tracking

### Time-Series
- ✅ peer_history (TimescaleDB ready)
- ✅ announces (partition by month)
- ✅ audit_logs (partition by month)

### Cleanup Jobs Required
1. **Daily**: Expire old sessions, stale peers
2. **Weekly**: Archive old chat messages
3. **Monthly**: Archive announces, aggregate peer_history
4. **Quarterly**: Archive audit_logs

## Technology Integration Points

### SQLx (Rust)
```rust
// Example model
#[derive(sqlx::FromRow)]
struct Torrent {
    id: Uuid,
    info_hash: String,
    name: String,
    category_id: i32,
    uploader_id: Uuid,
    // ... other fields
}
```

### Meilisearch
```
Indexed entities:
- torrents → torrent_index
- users → user_index
- forum_posts → forum_index

Via: search_index_queue table
```

### TimescaleDB
```sql
-- Convert to hypertable
SELECT create_hypertable('peer_history', 'time',
    chunk_time_interval => INTERVAL '1 day');

-- Enable compression
ALTER TABLE peer_history SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'torrent_id,user_id'
);
```

### Redis (Recommended for)
- Announce caching
- Peer list caching
- Session storage
- Rate limiting

## Security Considerations

### Sensitive Data
- `users.password_hash` - Argon2id hashed
- `users.passkey` - Unique, unguessable
- `user_2fa.secret` - Encrypted at rest
- `user_2fa.backup_codes` - Hashed
- `*.ip_address` - Privacy considerations (GDPR)

### Access Patterns
```
Public:
  - torrents (approved only)
  - torrent_categories
  - forums (respecting permissions)

Authenticated:
  - user_statistics (own + based on privacy)
  - peers (own)
  - private_messages (own)

Moderator:
  - reports
  - moderation_queue
  - warnings
  - *.ip_address fields

Admin:
  - audit_logs
  - bans
  - All data
```

## Backup Strategy

### Critical Tables (Daily backups)
- users
- user_statistics
- torrents
- torrent_files
- bonus_transactions

### Important Tables (Weekly backups)
- forum_posts
- private_messages
- warnings
- bans

### Archival Tables (Monthly export)
- announces
- peer_history
- audit_logs
- chat_messages

### Rebuildable (Don't backup)
- peers (current state)
- user_sessions
- search_index_queue
