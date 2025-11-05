//! Basic usage example for the search crate
//!
//! Run with: cargo run --example basic_usage

use search::{
    analytics::SearchAnalytics,
    client::SearchClient,
    facets::{FacetBuilder, FacetedSearch},
    filters::*,
    indexer::{IndexOperation, SearchIndexer, queue_index_operation},
    query::{SearchQuery, SortBy},
    suggest::AutocompleteService,
};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configuration
    let meilisearch_host = std::env::var("MEILISEARCH_HOST")
        .unwrap_or_else(|_| "http://localhost:7700".to_string());
    let meilisearch_key = std::env::var("MEILISEARCH_API_KEY")
        .unwrap_or_else(|_| "master_key".to_string());
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Connect to database
    let db = PgPool::connect(&database_url).await?;
    
    // Initialize search client
    println!("Connecting to Meilisearch at {}", meilisearch_host);
    let client = SearchClient::new(&meilisearch_host, &meilisearch_key).await?;
    
    // Check health
    let is_healthy = client.health_check().await?;
    println!("Meilisearch health: {}", if is_healthy { "OK" } else { "UNHEALTHY" });

    // Example 1: Basic search
    println!("\n=== Example 1: Basic Search ===");
    let results = client.search(
        SearchQuery::new("ubuntu")
            .with_limit(5)
    ).await?;
    
    println!("Found {} results in {}ms", 
        results.hits.len(), 
        results.processing_time_ms
    );
    
    for (i, hit) in results.hits.iter().enumerate() {
        println!("{}. {} ({} seeders)", 
            i + 1, 
            hit.result.name, 
            hit.result.seeders
        );
    }

    // Example 2: Advanced filtering
    println!("\n=== Example 2: Advanced Filtering ===");
    let filtered_results = client.search(
        SearchQuery::new("movies")
            .with_filter(CategoryFilter::single("movies"))
            .with_filter(SeedsFilter::well_seeded())
            .with_filter(SizeRangeFilter::gb(Some(1), Some(10)))
            .with_sort(SortBy::MostSeeders)
            .with_limit(5)
    ).await?;
    
    println!("Found {} well-seeded movies (1-10 GB)", filtered_results.hits.len());

    // Example 3: Faceted search
    println!("\n=== Example 3: Faceted Search ===");
    let faceted = FacetedSearch::new("")
        .add_facets(FacetBuilder::browse_facets());
    
    let facet_results = faceted.execute(&client.index()).await?;
    
    if let Some(categories) = facet_results.category_facets() {
        println!("Categories:");
        for (i, value) in categories.values.iter().take(5).enumerate() {
            println!("  {}. {}: {} torrents", i + 1, value.value, value.count);
        }
    }

    // Example 4: Autocomplete
    println!("\n=== Example 4: Autocomplete ===");
    let autocomplete = AutocompleteService::new(client.clone(), db.clone());
    
    let suggestions = autocomplete.suggest("ubun", 5).await?;
    println!("Suggestions for 'ubun':");
    for suggestion in suggestions {
        println!("  - {} ({:?})", suggestion.text, suggestion.suggestion_type);
    }

    // Example 5: Indexing
    println!("\n=== Example 5: Indexing Operations ===");
    let indexer = SearchIndexer::new(client.clone(), db.clone());
    
    // Get current stats
    let stats = client.get_stats().await?;
    println!("Current index stats:");
    println!("  Documents: {}", stats.number_of_documents);
    println!("  Is indexing: {}", stats.is_indexing);
    
    // Queue an index operation (example - you'll need a real torrent ID)
    // let torrent_id = Uuid::new_v4();
    // queue_index_operation(&db, torrent_id, IndexOperation::Upsert).await?;
    // println!("Queued torrent for indexing: {}", torrent_id);
    
    // Process queue
    let processed = indexer.process_queue().await?;
    println!("Processed {} items from queue", processed);

    // Example 6: Search Analytics
    println!("\n=== Example 6: Search Analytics ===");
    let analytics = SearchAnalytics::new(db.clone());
    
    // Get popular searches
    let popular = analytics.get_popular_searches(5, 7).await?;
    println!("Popular searches (last 7 days):");
    for search in popular {
        println!("  - '{}': {} searches", search.query, search.search_count);
    }
    
    // Get performance stats
    let perf_stats = analytics.get_performance_stats(7).await?;
    println!("Search performance (last 7 days):");
    println!("  Total searches: {}", perf_stats.total_searches);
    println!("  Avg time: {}ms", perf_stats.avg_time_ms);
    println!("  P95 time: {}ms", perf_stats.p95_time_ms);
    
    // Get CTR
    let ctr = analytics.get_click_through_rate(7).await?;
    println!("  Click-through rate: {:.2}%", ctr);

    // Example 7: Filter Builder
    println!("\n=== Example 7: Complex Filter Builder ===");
    let complex_filter = FilterBuilder::new()
        .add(CategoryFilter::single("movies"))
        .add(TagFilter::any(vec!["1080p".to_string(), "4K".to_string()]))
        .add(ResolutionFilter::hd())
        .add(DateRangeFilter::last_days(30))
        .add(PromotionFilter::freeleech_only())
        .build();
    
    println!("Built filter: {}", complex_filter);
    
    let complex_results = client.search(
        SearchQuery::new("")
            .with_raw_filter(complex_filter)
            .with_limit(3)
    ).await?;
    
    println!("Found {} freeleech HD movies", complex_results.hits.len());

    println!("\n=== All Examples Complete ===");
    
    Ok(())
}
