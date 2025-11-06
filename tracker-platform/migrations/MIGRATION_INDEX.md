# Migration Index - Quick Reference

## All Migrations (36 files)

| # | Timestamp | File | Table(s) | Dependencies |
|---|-----------|------|----------|--------------|
| 1 | 20250105000000 | create_user_groups.sql | user_groups | None |
| 2 | 20250105000001 | create_users.sql | users | user_groups |
| 3 | 20250105000002 | create_user_sessions.sql | user_sessions | users |
| 4 | 20250105000003 | create_user_privacy.sql | user_privacy | users |
| 5 | 20250105000004 | create_user_statistics.sql | user_statistics | users |
| 6 | 20250105000005 | create_user_achievements.sql | user_achievements | users |
| 7 | 20250105000006 | create_user_2fa.sql | user_2fa | users |
| 8 | 20250105000007 | create_torrent_categories.sql | torrent_categories | None |
| 9 | 20250105000008 | create_torrents.sql | torrents | torrent_categories, users |
| 10 | 20250105000009 | create_torrent_files.sql | torrent_files | torrents |
| 11 | 20250105000010 | create_torrent_tags.sql | torrent_tags, torrent_tag_assignments, torrent_tag_votes | torrents, users |
| 12 | 20250105000011 | create_torrent_moderation.sql | torrent_moderation | torrents, users |
| 13 | 20250105000012 | create_torrent_metadata.sql | torrent_metadata | torrents |
| 14 | 20250105000013 | create_torrent_requests.sql | torrent_requests, torrent_request_bounties, torrent_request_votes | torrent_categories, torrents, users |
| 15 | 20250105000014 | create_torrent_collections.sql | torrent_collections, torrent_collection_items, torrent_collection_subscriptions | torrent_categories, torrents, users |
| 16 | 20250105000015 | create_peers.sql | peers | torrents, users |
| 17 | 20250105000016 | create_announces.sql | announces | torrents, users |
| 18 | 20250105000017 | create_peer_history.sql | peer_history | torrents, users |
| 19 | 20250105000018 | create_torrent_statistics.sql | torrent_statistics | torrents |
| 20 | 20250105000019 | create_bonus_rules.sql | bonus_rules | None |
| 21 | 20250105000020 | create_bonus_transactions.sql | bonus_transactions | bonus_rules, torrents, users |
| 22 | 20250105000021 | create_freeleech_tokens.sql | freeleech_tokens | torrents, users |
| 23 | 20250105000022 | create_forums.sql | forums | None |
| 24 | 20250105000023 | create_forum_topics.sql | forum_topics | forums, users |
| 25 | 20250105000024 | create_forum_posts.sql | forum_posts | forum_topics, users |
| 26 | 20250105000025 | create_forum_subscriptions.sql | forum_subscriptions | forum_topics, forum_posts, users |
| 27 | 20250105000026 | create_private_messages.sql | private_messages | users |
| 28 | 20250105000027 | create_chat_rooms.sql | chat_rooms | None |
| 29 | 20250105000028 | create_chat_messages.sql | chat_messages | chat_rooms, users |
| 30 | 20250105000029 | create_reports.sql | reports | users |
| 31 | 20250105000030 | create_warnings.sql | warnings | reports, torrents, users |
| 32 | 20250105000031 | create_bans.sql | bans | warnings, users |
| 33 | 20250105000032 | create_audit_logs.sql | audit_logs | users |
| 34 | 20250105000033 | create_moderation_queue.sql | moderation_queue | users |
| 35 | 20250105000034 | create_search_index_queue.sql | search_index_queue | None |
| 36 | 20250105000035 | create_comments.sql | comments | users |

## Tables by Category

### Core User System (7 tables)
- user_groups
- users
- user_sessions
- user_privacy
- user_statistics
- user_achievements
- user_2fa

### Torrent System (15 tables)
- torrent_categories
- torrents
- torrent_files
- torrent_tags
- torrent_tag_assignments
- torrent_tag_votes
- torrent_moderation
- torrent_metadata
- torrent_requests
- torrent_request_bounties
- torrent_request_votes
- torrent_collections
- torrent_collection_items
- torrent_collection_subscriptions
- comments

### Tracker System (4 tables)
- peers
- announces
- peer_history
- torrent_statistics

### Bonus System (3 tables)
- bonus_rules
- bonus_transactions
- freeleech_tokens

### Community System (8 tables)
- forums
- forum_topics
- forum_posts
- forum_subscriptions
- private_messages
- chat_rooms
- chat_messages
- comments

### Moderation System (5 tables)
- reports
- warnings
- bans
- audit_logs
- moderation_queue

### Search System (1 table)
- search_index_queue

## Default Data Included

### user_groups (13 rows)
- banned, validating, member, power_user, elite, torrent_master, vip, uploader, designer, forum_moderator, moderator, administrator, sysop

### torrent_categories (28 rows)
- 8 main categories: movies, tv, music, games, software, books, anime, xxx
- 20 subcategories (HD, SD, 4K, FLAC, etc.)

### forums (7 rows)
- announcements, general, support, requests, off-topic, vip-lounge, staff

### chat_rooms (5 rows)
- general, support, trading, vip, staff

### bonus_rules (4 rows)
- standard_seeding, low_seeder_bonus, freeleech_seeding, large_file_bonus

## Column Type Reference

### Common Types Used
- **UUID**: User IDs, entity IDs (gen_random_uuid())
- **VARCHAR(N)**: Strings with known max length
- **TEXT**: Unlimited text
- **BIGINT**: Large numbers (file sizes, transfer stats)
- **INTEGER**: Counts, IDs
- **DECIMAL(20,2)**: Money/bonus points
- **BOOLEAN**: Flags
- **TIMESTAMP WITH TIME ZONE**: All timestamps
- **INET**: IP addresses
- **CIDR**: IP ranges
- **JSONB**: Semi-structured data
- **TEXT[]**: Arrays

### Special PostgreSQL Features
- **SERIAL**: Auto-incrementing integers (for small lookup tables)
- **GIN indexes**: For arrays, JSONB, full-text search
- **Partial indexes**: For filtered queries (WHERE clauses)
- **to_tsvector()**: Full-text search vectors

## Common Patterns

### Polymorphic Relationships
```sql
-- Pattern used in: comments, reports, moderation_queue, audit_logs
commentable_type VARCHAR(50),  -- 'torrent', 'request', etc.
commentable_id UUID             -- References the entity
```

### Soft Delete
```sql
-- Pattern used in: users, torrents, comments, forum_posts
deleted_at TIMESTAMP WITH TIME ZONE
-- Query: WHERE deleted_at IS NULL
```

### Audit Fields
```sql
-- Standard pattern across all tables
created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
```

### Tracking User Actions
```sql
-- Pattern used in: moderation, reports, bans, warnings
action_user_id UUID REFERENCES users(id),
action_ip INET,
action_at TIMESTAMP WITH TIME ZONE
```

## Foreign Key Constraints

### ON DELETE Behaviors Used

#### RESTRICT
Used for core references that shouldn't be deleted:
- uploader_id → users (keep upload history)
- author_id → users (preserve authorship)

#### CASCADE
Used when child should be deleted with parent:
- user_statistics → users
- torrent_files → torrents
- peers → torrents

#### SET NULL
Used when reference is optional:
- invited_by → users
- filled_by → users (requests)
- moderator references

## Index Types

### B-tree (default)
- Primary keys
- Foreign keys
- Equality and range queries
- Most indexes

### GIN (Generalized Inverted Index)
- JSONB columns
- Array columns
- Full-text search (tsvector)

### Partial Indexes
```sql
-- Example: Only index active items
WHERE is_active = true
WHERE deleted_at IS NULL
WHERE status = 'pending'
```

### Unique Indexes
```sql
-- Example: Prevent duplicates in junction tables
UNIQUE INDEX (user_id, torrent_id)
UNIQUE INDEX (collection_id, torrent_id)
```

## Maintenance Queries

### Check Migration Status
```bash
sqlx migrate info
```

### Apply All Migrations
```bash
sqlx migrate run
```

### Database Size
```sql
SELECT
    pg_size_pretty(pg_database_size(current_database())) as db_size;
```

### Table Sizes
```sql
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Index Usage
```sql
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

### Find Unused Indexes
```sql
SELECT
    schemaname,
    tablename,
    indexname
FROM pg_stat_user_indexes
WHERE idx_scan = 0
AND indexname NOT LIKE '%_pkey';
```

## Next Steps

1. **Review**: Examine migration files for project-specific needs
2. **Customize**: Adjust default data (user groups, categories, etc.)
3. **Test**: Run migrations in development environment
4. **Seed**: Add additional seed data as needed
5. **Monitor**: Set up monitoring for table sizes and query performance
6. **Partition**: Implement partitioning for high-volume tables
7. **Archive**: Set up archival strategies for time-series data
8. **Backup**: Implement backup schedule for critical tables

## Useful Commands

### Create New Migration
```bash
sqlx migrate add <name>
```

### Revert Last Migration (manual)
```sql
-- SQLx doesn't support down migrations by default
-- You'll need to manually write and execute revert SQL
```

### Force Migration Version
```bash
sqlx migrate add --reversible <name>
# Creates both up and down files
```

### Database Connection String Format
```
postgresql://user:password@localhost:5432/database?sslmode=require
```

## Support

For questions or issues:
1. Check README.md for detailed documentation
2. Review SCHEMA_DIAGRAM.md for visual relationships
3. Examine individual migration files for specifics
4. Consult SQLx documentation: https://github.com/launchbadge/sqlx
