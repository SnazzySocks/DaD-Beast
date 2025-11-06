# Search Crate

Advanced Meilisearch integration service for the tracker platform, providing full-text search, faceted filtering, autocomplete, and search analytics.

## Features

- **Full-Text Search**: Fast and relevant search using Meilisearch
- **Advanced Filtering**: Category, tags, date range, size, seeders, and more
- **Faceted Search**: Dynamic facets for browsing and filtering
- **Autocomplete**: Real-time search suggestions as you type
- **Search Analytics**: Track queries, clicks, CTR, and performance
- **Background Indexing**: Automatic queue processing for index updates
- **Batch Operations**: Efficient bulk indexing and deletion
- **A/B Testing**: Support for search algorithm experimentation

## Architecture

### Core Modules

1. **client** - Meilisearch client initialization and index management
2. **schema** - Document structure and index configuration
3. **indexer** - Indexing operations and queue processing
4. **query** - Search query builder with filtering and sorting
5. **filters** - Advanced filtering (category, tags, date, size, etc.)
6. **facets** - Faceted search for browsing and aggregation
7. **suggest** - Autocomplete and search suggestions
8. **analytics** - Search tracking and performance analytics

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
search = { path = "../search" }
```

## Quick Start

```rust
use search::{SearchClient, SearchQuery, SortBy};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client
    let client = SearchClient::new(
        "http://localhost:7700",
        "master_key"
    ).await?;
    
    // Build a search query
    let query = SearchQuery::new("ubuntu")
        .with_category("linux")
        .with_sort(SortBy::MostSeeders)
        .with_limit(20);
    
    // Execute search
    let results = client.search(query).await?;
    
    println!("Found {} results", results.hits.len());
    for hit in results.hits {
        println!("- {} ({} seeders)", hit.result.name, hit.result.seeders);
    }
    
    Ok(())
}
```

## Usage Examples

### Basic Search

```rust
use search::{SearchClient, SearchQuery};

let client = SearchClient::new("http://localhost:7700", "key").await?;

// Simple text search
let results = client.search(
    SearchQuery::new("ubuntu 22.04")
).await?;

// Search with pagination
let results = client.search(
    SearchQuery::new("movies")
        .with_page(2)
        .with_limit(50)
).await?;
```

### Advanced Filtering

```rust
use search::filters::*;
use search::query::SearchQuery;

let query = SearchQuery::new("action movies")
    .with_filter(CategoryFilter::single("movies"))
    .with_filter(SeedsFilter::well_seeded())
    .with_filter(SizeRangeFilter::gb(Some(1), Some(10)))
    .with_filter(DateRangeFilter::last_days(30))
    .with_filter(PromotionFilter::freeleech_only());

let results = client.search(query).await?;
```

### Filter Builder

```rust
use search::filters::*;

let filters = FilterBuilder::new()
    .add(CategoryFilter::single("movies"))
    .add(TagFilter::any(vec!["1080p".to_string(), "4K".to_string()]))
    .add(SeedsFilter::new().min_seeders(10))
    .add(ResolutionFilter::hd())
    .build();

let query = SearchQuery::new("action")
    .with_raw_filter(filters);
```

### Faceted Search

```rust
use search::facets::*;

// Get facets for browsing
let faceted = FacetedSearch::new("movies")
    .add_facets(vec![
        "category".to_string(),
        "resolution".to_string(),
        "year".to_string(),
    ]);

let results = faceted.execute(&client.index()).await?;

// Access facet counts
if let Some(categories) = results.category_facets() {
    for value in categories.values {
        println!("{}: {} results", value.value, value.count);
    }
}
```

### Autocomplete

```rust
use search::suggest::AutocompleteService;

let autocomplete = AutocompleteService::new(client, db);

// Get suggestions
let suggestions = autocomplete.suggest("ubun", 10).await?;

for suggestion in suggestions {
    println!("- {}", suggestion.text);
}

// Get popular searches
let popular = autocomplete.get_popular_searches(20).await?;
```

### Indexing

```rust
use search::indexer::{SearchIndexer, IndexOperation, queue_index_operation};
use uuid::Uuid;

let indexer = SearchIndexer::new(client, db);

// Index a single torrent
indexer.index_torrent(torrent_id).await?;

// Index multiple torrents
indexer.index_torrents_batch(vec![id1, id2, id3]).await?;

// Queue an index operation (for async processing)
queue_index_operation(&db, torrent_id, IndexOperation::Upsert).await?;

// Start background indexing job
tokio::spawn(async move {
    indexer.start_background_job().await
});

// Full reindex
let count = indexer.reindex_all().await?;
println!("Reindexed {} torrents", count);
```

### Search Analytics

```rust
use search::analytics::SearchAnalytics;

let analytics = SearchAnalytics::new(db);

// Track a search
analytics.track_search(
    Some(user_id),
    "ubuntu",
    Some("category = \"linux\""),
    results.hits.len() as u64,
    results.processing_time_ms,
).await?;

// Track clicks
analytics.track_click(Some(search_id), Some(user_id), torrent_id, 1).await?;

// Get popular searches
let popular = analytics.get_popular_searches(20, 7).await?;

// Get performance stats
let stats = analytics.get_performance_stats(30).await?;
println!("Avg search time: {}ms", stats.avg_time_ms);
println!("P95 search time: {}ms", stats.p95_time_ms);

// Get click-through rate
let ctr = analytics.get_click_through_rate(7).await?;
println!("CTR: {:.2}%", ctr);
```

## Configuration

### Meilisearch Index Configuration

The default configuration includes:

**Searchable Attributes** (with weights):
- `name` (weight: 10) - Torrent name
- `tags` (weight: 5) - Tags
- `description` (weight: 3) - Description

**Filterable Attributes**:
- `category`, `tags`, `media_type`, `resolution`, `codec`, `quality`
- `uploaded_at`, `size`, `seeders`, `leechers`
- `uploader`, `uploader_id`, `tmdb_id`, `igdb_id`, `year`
- `is_freeleech`, `is_double_upload`, `is_featured`

**Sortable Attributes**:
- `uploaded_at`, `size`, `seeders`, `leechers`, `snatched`
- `rating`, `comment_count`, `is_featured`

**Ranking Rules** (in order):
1. `typo` - Prioritize fewer typos
2. `words` - Prioritize more query words
3. `proximity` - Prioritize closer query words
4. `attribute` - Prioritize important attributes
5. `sort` - Apply custom sorting
6. `exactness` - Prioritize exact matches

### Custom Configuration

```rust
use search::schema::SearchConfig;

let config = SearchConfig::new("custom_index")
    .add_searchable_attribute("custom_field")
    .add_filterable_attribute("filter_field")
    .add_sortable_attribute("sort_field")
    .add_synonym("movie", vec!["film".to_string(), "cinema".to_string()]);

let client = SearchClient::with_config(
    "http://localhost:7700",
    "key",
    config
).await?;
```

## Database Schema

### Required Tables

```sql
-- Search index queue for async indexing
CREATE TABLE search_index_queue (
    id BIGSERIAL PRIMARY KEY,
    torrent_id UUID NOT NULL,
    operation VARCHAR(10) NOT NULL, -- 'upsert' or 'delete'
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(torrent_id)
);

-- Search history for analytics
CREATE TABLE search_history (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID,
    query TEXT NOT NULL,
    filters TEXT,
    results_count BIGINT NOT NULL,
    processing_time_ms BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_history_user ON search_history(user_id);
CREATE INDEX idx_search_history_created ON search_history(created_at);
CREATE INDEX idx_search_history_query ON search_history(query);

-- Search clicks for CTR tracking
CREATE TABLE search_clicks (
    id BIGSERIAL PRIMARY KEY,
    search_id BIGINT,
    user_id UUID,
    torrent_id UUID NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (search_id) REFERENCES search_history(id)
);

CREATE INDEX idx_search_clicks_search ON search_clicks(search_id);
CREATE INDEX idx_search_clicks_torrent ON search_clicks(torrent_id);
CREATE INDEX idx_search_clicks_created ON search_clicks(created_at);

-- A/B testing
CREATE TABLE search_ab_tests (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID,
    test_name VARCHAR(100) NOT NULL,
    variant VARCHAR(50) NOT NULL,
    query TEXT NOT NULL,
    results_count BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_ab_tests_test ON search_ab_tests(test_name, variant);
```

## Performance Considerations

### Indexing Performance

- Use batch operations for bulk indexing (default: 100 items per batch)
- Background job processes queue every 5 seconds (configurable)
- Use `queue_index_operation` for async indexing
- Consider rate limiting for large reindex operations

### Search Performance

- Meilisearch typically responds in 10-50ms
- Use pagination to limit result set size
- Filter before faceting for better performance
- Cache popular searches at application level

### Database Performance

- Ensure indexes exist on search_history and search_clicks tables
- Archive old analytics data periodically
- Consider partitioning search_history by date for large datasets

## Deployment

### Meilisearch Setup

```bash
# Using Docker
docker run -d \
  -p 7700:7700 \
  -v $(pwd)/meili_data:/meili_data \
  -e MEILI_MASTER_KEY=your_master_key \
  getmeili/meilisearch:latest

# Or install directly
curl -L https://install.meilisearch.com | sh
./meilisearch --master-key your_master_key
```

### Environment Variables

```bash
MEILISEARCH_HOST=http://localhost:7700
MEILISEARCH_API_KEY=your_master_key
```

### Background Indexer Service

```rust
// In your main application
let search_client = SearchClient::new(&config.meilisearch_host, &config.api_key).await?;
let indexer = SearchIndexer::with_settings(
    search_client,
    db.clone(),
    100,  // batch_size
    Duration::from_secs(5), // poll_interval
);

// Spawn background indexer
tokio::spawn(async move {
    if let Err(e) = indexer.start_background_job().await {
        error!("Indexer error: {}", e);
    }
});
```

## Testing

```bash
# Run tests (requires Meilisearch instance)
cargo test

# Run tests with ignored integration tests
cargo test -- --ignored

# Run specific test
cargo test test_search_query_builder
```

## Contributing

When adding new features:

1. Add appropriate filterable/sortable attributes to schema
2. Update tests
3. Document new functionality in README
4. Consider performance implications

## License

Part of the tracker-platform project.
