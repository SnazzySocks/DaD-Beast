# Search Crate - Quick Reference

## Installation & Setup

```bash
# Start Meilisearch
docker run -d -p 7700:7700 \
  -e MEILI_MASTER_KEY=your_master_key \
  getmeili/meilisearch:latest

# Run database migrations
psql -d tracker -f migrations/001_search_tables.sql
```

## Common Operations

### 1. Initialize Client

```rust
use search::SearchClient;

let client = SearchClient::new(
    "http://localhost:7700",
    "master_key"
).await?;
```

### 2. Basic Search

```rust
use search::{SearchQuery, SortBy};

// Simple search
let results = client.search(
    SearchQuery::new("ubuntu")
).await?;

// With sorting and pagination
let results = client.search(
    SearchQuery::new("movies")
        .with_sort(SortBy::MostSeeders)
        .with_page(2)
        .with_limit(20)
).await?;
```

### 3. Filtering

```rust
use search::filters::*;

// Single filter
let query = SearchQuery::new("action")
    .with_filter(CategoryFilter::single("movies"));

// Multiple filters
let query = SearchQuery::new("movies")
    .with_filter(SeedsFilter::well_seeded())
    .with_filter(SizeRangeFilter::gb(Some(1), Some(10)))
    .with_filter(DateRangeFilter::last_days(30))
    .with_filter(PromotionFilter::freeleech_only());

// Filter builder
let filters = FilterBuilder::new()
    .add(CategoryFilter::single("movies"))
    .add(TagFilter::any(vec!["1080p".to_string(), "4K".to_string()]))
    .add(ResolutionFilter::hd())
    .build();
```

### 4. Faceted Search

```rust
use search::facets::{FacetedSearch, FacetBuilder};

let faceted = FacetedSearch::new("movies")
    .add_facets(FacetBuilder::browse_facets());

let results = faceted.execute(&client.index()).await?;

// Access facets
if let Some(categories) = results.category_facets() {
    for value in categories.values {
        println!("{}: {}", value.value, value.count);
    }
}
```

### 5. Autocomplete

```rust
use search::suggest::AutocompleteService;

let autocomplete = AutocompleteService::new(client, db);

// Get suggestions
let suggestions = autocomplete.suggest("ubun", 10).await?;

// Recent searches for user
let recent = autocomplete.get_recent_searches(user_id, 10).await?;

// Popular searches
let popular = autocomplete.get_popular_searches(20).await?;
```

### 6. Indexing

```rust
use search::indexer::{SearchIndexer, IndexOperation, queue_index_operation};

let indexer = SearchIndexer::new(client, db);

// Index single torrent
indexer.index_torrent(torrent_id).await?;

// Index batch
indexer.index_torrents_batch(vec![id1, id2, id3]).await?;

// Queue for background processing
queue_index_operation(&db, torrent_id, IndexOperation::Upsert).await?;

// Start background worker
tokio::spawn(async move {
    indexer.start_background_job().await
});

// Full reindex
indexer.reindex_all().await?;
```

### 7. Analytics

```rust
use search::analytics::SearchAnalytics;

let analytics = SearchAnalytics::new(db);

// Track search
analytics.track_search(
    Some(user_id),
    "ubuntu",
    None,
    results_count,
    processing_time_ms
).await?;

// Track click
analytics.track_click(
    Some(search_id),
    Some(user_id),
    torrent_id,
    position
).await?;

// Get statistics
let popular = analytics.get_popular_searches(20, 7).await?;
let no_results = analytics.get_no_result_searches(20, 7).await?;
let stats = analytics.get_performance_stats(30).await?;
let ctr = analytics.get_click_through_rate(7).await?;
```

## Available Filters

| Filter | Usage | Example |
|--------|-------|---------|
| `CategoryFilter` | Filter by category | `CategoryFilter::single("movies")` |
| `TagFilter` | Filter by tags (AND/OR) | `TagFilter::any(vec!["1080p", "x264"])` |
| `DateRangeFilter` | Filter by date | `DateRangeFilter::last_days(30)` |
| `SizeRangeFilter` | Filter by size | `SizeRangeFilter::gb(Some(1), Some(10))` |
| `SeedsFilter` | Filter by seeders/leechers | `SeedsFilter::well_seeded()` |
| `UploaderFilter` | Filter by uploader | `UploaderFilter::by_name("user123")` |
| `MediaTypeFilter` | Filter by media type | `MediaTypeFilter::single("movie")` |
| `ResolutionFilter` | Filter by resolution | `ResolutionFilter::hd()` |
| `PromotionFilter` | Filter freeleech/double | `PromotionFilter::freeleech_only()` |

## Sort Options

- `SortBy::NewestFirst` / `SortBy::OldestFirst`
- `SortBy::LargestFirst` / `SortBy::SmallestFirst`
- `SortBy::MostSeeders` / `SortBy::MostLeechers`
- `SortBy::MostSnatches`
- `SortBy::HighestRated`
- `SortBy::MostCommented`
- `SortBy::FeaturedFirst`

## Background Jobs

### Indexer Worker

```rust
// Start in your main application
let indexer = SearchIndexer::with_settings(
    client,
    db.clone(),
    100,                          // batch_size
    Duration::from_secs(5),       // poll_interval
);

tokio::spawn(async move {
    indexer.start_background_job().await
});
```

### Popular Searches Refresh

```sql
-- Run periodically (e.g., via cron)
SELECT refresh_popular_searches();
```

### Cleanup Old History

```sql
-- Run periodically to clean old data
SELECT cleanup_old_search_history(90); -- Keep 90 days
```

## Error Handling

```rust
use search::error::{SearchError, SearchResult};

match client.search(query).await {
    Ok(results) => println!("Found {} results", results.hits.len()),
    Err(SearchError::Meilisearch(e)) => eprintln!("Meilisearch error: {}", e),
    Err(SearchError::Database(e)) => eprintln!("Database error: {}", e),
    Err(SearchError::InvalidQuery(msg)) => eprintln!("Invalid query: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Environment Variables

```bash
MEILISEARCH_HOST=http://localhost:7700
MEILISEARCH_API_KEY=your_master_key
DATABASE_URL=postgres://user:pass@localhost/tracker
```

## Performance Tips

1. **Use batch operations** for bulk indexing
2. **Enable pagination** for large result sets
3. **Filter before faceting** for better performance
4. **Cache popular searches** at application level
5. **Index cleanup** - Archive old analytics data
6. **Monitor queue size** - Ensure indexer keeps up
7. **Use connection pooling** for database

## Monitoring

```rust
// Check index health
let stats = client.get_stats().await?;
println!("Documents: {}", stats.number_of_documents);
println!("Is indexing: {}", stats.is_indexing);

// Check queue size
let queue_size = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM search_index_queue"
).fetch_one(&db).await?;

// Performance metrics
let perf = analytics.get_performance_stats(1).await?;
println!("Avg: {}ms, P95: {}ms", perf.avg_time_ms, perf.p95_time_ms);
```

## Troubleshooting

### Queue backing up
- Increase `batch_size` in indexer
- Reduce `poll_interval`
- Check Meilisearch performance

### Slow searches
- Review filter complexity
- Check index configuration
- Monitor Meilisearch logs
- Consider caching

### High memory usage
- Reduce batch sizes
- Archive old analytics data
- Check Meilisearch settings

## See Also

- [README.md](./README.md) - Full documentation
- [examples/basic_usage.rs](./examples/basic_usage.rs) - Complete examples
- [Meilisearch Documentation](https://docs.meilisearch.com/)
