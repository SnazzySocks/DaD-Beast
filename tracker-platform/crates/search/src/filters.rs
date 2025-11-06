//! Advanced filtering for search queries

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Filter trait for building Meilisearch filter strings
pub trait Filter {
    /// Convert filter to Meilisearch filter string
    fn to_filter_string(&self) -> String;
}

/// Category filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFilter {
    pub categories: Vec<String>,
}

impl CategoryFilter {
    pub fn new(categories: Vec<String>) -> Self {
        Self { categories }
    }

    pub fn single(category: impl Into<String>) -> Self {
        Self {
            categories: vec![category.into()],
        }
    }
}

impl Filter for CategoryFilter {
    fn to_filter_string(&self) -> String {
        if self.categories.is_empty() {
            return String::new();
        }
        
        if self.categories.len() == 1 {
            format!("category = \"{}\"", self.categories[0])
        } else {
            let filters: Vec<String> = self
                .categories
                .iter()
                .map(|cat| format!("category = \"{}\"", cat))
                .collect();
            format!("({})", filters.join(" OR "))
        }
    }
}

/// Tag filter with AND/OR logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilter {
    pub tags: Vec<String>,
    pub logic: TagLogic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TagLogic {
    /// Match all tags (AND)
    All,
    /// Match any tag (OR)
    Any,
}

impl TagFilter {
    pub fn all(tags: Vec<String>) -> Self {
        Self {
            tags,
            logic: TagLogic::All,
        }
    }

    pub fn any(tags: Vec<String>) -> Self {
        Self {
            tags,
            logic: TagLogic::Any,
        }
    }
}

impl Filter for TagFilter {
    fn to_filter_string(&self) -> String {
        if self.tags.is_empty() {
            return String::new();
        }

        let filters: Vec<String> = self
            .tags
            .iter()
            .map(|tag| format!("tags = \"{}\"", tag))
            .collect();

        match self.logic {
            TagLogic::All => filters.join(" AND "),
            TagLogic::Any => format!("({})", filters.join(" OR ")),
        }
    }
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeFilter {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl DateRangeFilter {
    pub fn new(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        Self { start, end }
    }

    pub fn from(start: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: None,
        }
    }

    pub fn until(end: DateTime<Utc>) -> Self {
        Self {
            start: None,
            end: Some(end),
        }
    }

    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    /// Last N days
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self::between(start, end)
    }

    /// Last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self::between(start, end)
    }
}

impl Filter for DateRangeFilter {
    fn to_filter_string(&self) -> String {
        match (&self.start, &self.end) {
            (Some(start), Some(end)) => {
                format!(
                    "uploaded_at >= {} AND uploaded_at <= {}",
                    start.timestamp(),
                    end.timestamp()
                )
            }
            (Some(start), None) => {
                format!("uploaded_at >= {}", start.timestamp())
            }
            (None, Some(end)) => {
                format!("uploaded_at <= {}", end.timestamp())
            }
            (None, None) => String::new(),
        }
    }
}

/// Size range filter (in bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeRangeFilter {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl SizeRangeFilter {
    pub fn new(min: Option<i64>, max: Option<i64>) -> Self {
        Self { min, max }
    }

    pub fn min(min: i64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn max(max: i64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    pub fn between(min: i64, max: i64) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    /// Size range in megabytes
    pub fn mb(min: Option<i64>, max: Option<i64>) -> Self {
        const MB: i64 = 1024 * 1024;
        Self {
            min: min.map(|v| v * MB),
            max: max.map(|v| v * MB),
        }
    }

    /// Size range in gigabytes
    pub fn gb(min: Option<i64>, max: Option<i64>) -> Self {
        const GB: i64 = 1024 * 1024 * 1024;
        Self {
            min: min.map(|v| v * GB),
            max: max.map(|v| v * GB),
        }
    }
}

impl Filter for SizeRangeFilter {
    fn to_filter_string(&self) -> String {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => {
                format!("size >= {} AND size <= {}", min, max)
            }
            (Some(min), None) => {
                format!("size >= {}", min)
            }
            (None, Some(max)) => {
                format!("size <= {}", max)
            }
            (None, None) => String::new(),
        }
    }
}

/// Seeder/leecher count filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedsFilter {
    pub min_seeders: Option<i32>,
    pub max_seeders: Option<i32>,
    pub min_leechers: Option<i32>,
    pub max_leechers: Option<i32>,
}

impl SeedsFilter {
    pub fn new() -> Self {
        Self {
            min_seeders: None,
            max_seeders: None,
            min_leechers: None,
            max_leechers: None,
        }
    }

    pub fn min_seeders(mut self, min: i32) -> Self {
        self.min_seeders = Some(min);
        self
    }

    pub fn max_seeders(mut self, max: i32) -> Self {
        self.max_seeders = Some(max);
        self
    }

    pub fn min_leechers(mut self, min: i32) -> Self {
        self.min_leechers = Some(min);
        self
    }

    pub fn max_leechers(mut self, max: i32) -> Self {
        self.max_leechers = Some(max);
        self
    }

    /// Only show well-seeded torrents (5+ seeders)
    pub fn well_seeded() -> Self {
        Self::new().min_seeders(5)
    }

    /// Only show active torrents (1+ seeder)
    pub fn active() -> Self {
        Self::new().min_seeders(1)
    }
}

impl Default for SeedsFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for SeedsFilter {
    fn to_filter_string(&self) -> String {
        let mut filters = Vec::new();

        if let Some(min) = self.min_seeders {
            filters.push(format!("seeders >= {}", min));
        }
        if let Some(max) = self.max_seeders {
            filters.push(format!("seeders <= {}", max));
        }
        if let Some(min) = self.min_leechers {
            filters.push(format!("leechers >= {}", min));
        }
        if let Some(max) = self.max_leechers {
            filters.push(format!("leechers <= {}", max));
        }

        filters.join(" AND ")
    }
}

/// Uploader filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploaderFilter {
    pub uploader_id: Option<Uuid>,
    pub uploader_name: Option<String>,
}

impl UploaderFilter {
    pub fn by_id(uploader_id: Uuid) -> Self {
        Self {
            uploader_id: Some(uploader_id),
            uploader_name: None,
        }
    }

    pub fn by_name(uploader_name: impl Into<String>) -> Self {
        Self {
            uploader_id: None,
            uploader_name: Some(uploader_name.into()),
        }
    }
}

impl Filter for UploaderFilter {
    fn to_filter_string(&self) -> String {
        match (&self.uploader_id, &self.uploader_name) {
            (Some(id), _) => format!("uploader_id = \"{}\"", id),
            (None, Some(name)) => format!("uploader = \"{}\"", name),
            (None, None) => String::new(),
        }
    }
}

/// Media type filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTypeFilter {
    pub media_types: Vec<String>,
}

impl MediaTypeFilter {
    pub fn new(media_types: Vec<String>) -> Self {
        Self { media_types }
    }

    pub fn single(media_type: impl Into<String>) -> Self {
        Self {
            media_types: vec![media_type.into()],
        }
    }
}

impl Filter for MediaTypeFilter {
    fn to_filter_string(&self) -> String {
        if self.media_types.is_empty() {
            return String::new();
        }

        if self.media_types.len() == 1 {
            format!("media_type = \"{}\"", self.media_types[0])
        } else {
            let filters: Vec<String> = self
                .media_types
                .iter()
                .map(|mt| format!("media_type = \"{}\"", mt))
                .collect();
            format!("({})", filters.join(" OR "))
        }
    }
}

/// Resolution filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionFilter {
    pub resolutions: Vec<String>,
}

impl ResolutionFilter {
    pub fn new(resolutions: Vec<String>) -> Self {
        Self { resolutions }
    }

    pub fn single(resolution: impl Into<String>) -> Self {
        Self {
            resolutions: vec![resolution.into()],
        }
    }

    pub fn hd() -> Self {
        Self::new(vec!["720p".to_string(), "1080p".to_string()])
    }

    pub fn full_hd() -> Self {
        Self::single("1080p")
    }

    pub fn ultra_hd() -> Self {
        Self::new(vec!["4K".to_string(), "2160p".to_string()])
    }
}

impl Filter for ResolutionFilter {
    fn to_filter_string(&self) -> String {
        if self.resolutions.is_empty() {
            return String::new();
        }

        if self.resolutions.len() == 1 {
            format!("resolution = \"{}\"", self.resolutions[0])
        } else {
            let filters: Vec<String> = self
                .resolutions
                .iter()
                .map(|res| format!("resolution = \"{}\"", res))
                .collect();
            format!("({})", filters.join(" OR "))
        }
    }
}

/// Freeleech/Double upload filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionFilter {
    pub freeleech: Option<bool>,
    pub double_upload: Option<bool>,
}

impl PromotionFilter {
    pub fn new() -> Self {
        Self {
            freeleech: None,
            double_upload: None,
        }
    }

    pub fn freeleech_only() -> Self {
        Self {
            freeleech: Some(true),
            double_upload: None,
        }
    }

    pub fn double_upload_only() -> Self {
        Self {
            freeleech: None,
            double_upload: Some(true),
        }
    }

    pub fn any_promotion() -> Self {
        Self {
            freeleech: Some(true),
            double_upload: Some(true),
        }
    }
}

impl Default for PromotionFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for PromotionFilter {
    fn to_filter_string(&self) -> String {
        let mut filters = Vec::new();

        match (self.freeleech, self.double_upload) {
            (Some(true), Some(true)) => {
                // Either freeleech OR double upload
                return "(is_freeleech = true OR is_double_upload = true)".to_string();
            }
            (Some(true), _) => filters.push("is_freeleech = true".to_string()),
            (_, Some(true)) => filters.push("is_double_upload = true".to_string()),
            _ => {}
        }

        filters.join(" AND ")
    }
}

/// Combined filter builder
#[derive(Debug, Clone, Default)]
pub struct FilterBuilder {
    filters: Vec<String>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<F: Filter>(mut self, filter: F) -> Self {
        let filter_str = filter.to_filter_string();
        if !filter_str.is_empty() {
            self.filters.push(filter_str);
        }
        self
    }

    pub fn build(self) -> String {
        if self.filters.is_empty() {
            return String::new();
        }
        self.filters.join(" AND ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_filter() {
        let filter = CategoryFilter::single("movies");
        assert_eq!(filter.to_filter_string(), "category = \"movies\"");

        let filter = CategoryFilter::new(vec!["movies".to_string(), "tv".to_string()]);
        assert_eq!(
            filter.to_filter_string(),
            "(category = \"movies\" OR category = \"tv\")"
        );
    }

    #[test]
    fn test_tag_filter() {
        let filter = TagFilter::all(vec!["1080p".to_string(), "x264".to_string()]);
        assert_eq!(
            filter.to_filter_string(),
            "tags = \"1080p\" AND tags = \"x264\""
        );

        let filter = TagFilter::any(vec!["action".to_string(), "comedy".to_string()]);
        assert_eq!(
            filter.to_filter_string(),
            "(tags = \"action\" OR tags = \"comedy\")"
        );
    }

    #[test]
    fn test_size_filter() {
        let filter = SizeRangeFilter::gb(Some(1), Some(5));
        let filter_str = filter.to_filter_string();
        assert!(filter_str.contains("size >="));
        assert!(filter_str.contains("AND"));
        assert!(filter_str.contains("size <="));
    }

    #[test]
    fn test_seeds_filter() {
        let filter = SeedsFilter::new().min_seeders(5);
        assert_eq!(filter.to_filter_string(), "seeders >= 5");

        let filter = SeedsFilter::well_seeded();
        assert_eq!(filter.to_filter_string(), "seeders >= 5");
    }

    #[test]
    fn test_filter_builder() {
        let filter = FilterBuilder::new()
            .add(CategoryFilter::single("movies"))
            .add(SeedsFilter::well_seeded())
            .build();

        assert!(filter.contains("category = \"movies\""));
        assert!(filter.contains("seeders >= 5"));
        assert!(filter.contains(" AND "));
    }
}
