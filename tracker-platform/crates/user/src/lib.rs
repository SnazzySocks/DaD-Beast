//! User Management and Social Features
//!
//! This crate provides comprehensive user management functionality for the unified
//! tracker platform, including:
//!
//! - **User Profiles**: Customizable profiles with avatars, signatures, and privacy controls
//! - **Statistics**: Upload/download tracking, ratio calculation, and activity metrics
//! - **Seedbonus System**: Rule-based bonus earning system (Unit3d pattern)
//! - **Freeleech System**: Three-tier freeleech with tokens and temporary windows
//! - **Achievements**: Badge/achievement system with progress tracking
//! - **Privacy Controls**: Granular privacy settings (Gazelle paranoia system)
//! - **Invitation System**: Invite tree tracking and quota management
//! - **Social Features**: Follow/unfollow users and activity feeds
//!
//! # Architecture
//!
//! The user crate follows a layered architecture:
//!
//! 1. **Core Layer** (`profile`, `statistics`, `privacy`)
//!    - User profile data and management
//!    - Statistics calculation and tracking
//!    - Privacy settings and enforcement
//!
//! 2. **Incentive Layer** (`bonus`, `freeleech`, `achievements`)
//!    - Seedbonus earning and spending
//!    - Freeleech token system
//!    - Achievement tracking and awards
//!
//! 3. **Social Layer** (`invites`, `follow`)
//!    - Invitation system with tree tracking
//!    - User following and activity feeds
//!    - Social interaction features
//!
//! # Quick Start
//!
//! ## Managing User Profiles
//!
//! ```rust,no_run
//! use user::profile::{ProfileService, UpdateProfileRequest};
//! use uuid::Uuid;
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let profile_service = ProfileService::new(db_pool);
//! let user_id = Uuid::new_v4();
//!
//! // Get user profile
//! let profile = profile_service.get_profile(user_id).await?;
//! println!("User: {} ({})", profile.username, profile.custom_title.unwrap_or_default());
//!
//! // Update profile
//! let update = UpdateProfileRequest {
//!     custom_title: Some("Power User".to_string()),
//!     about_me: Some("Passionate about sharing".to_string()),
//!     signature: None,
//! };
//! profile_service.update_profile(user_id, update).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Seedbonus
//!
//! ```rust,no_run
//! use user::bonus::{BonusService, BonusRule, BonusOperation};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let bonus_service = BonusService::new(db_pool);
//!
//! // Calculate bonus for a user's active torrents
//! let user_id = uuid::Uuid::new_v4();
//! let earned = bonus_service.calculate_user_bonus(user_id).await?;
//! println!("Earned {} bonus points", earned);
//!
//! // Exchange bonus for upload credit
//! bonus_service.exchange_bonus_for_upload(
//!     user_id,
//!     1000.0, // bonus points
//!     10 * 1024 * 1024 * 1024, // 10 GB upload credit
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Freeleech Tokens
//!
//! ```rust,no_run
//! use user::freeleech::{FreeleechService, TokenType};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let freeleech_service = FreeleechService::new(db_pool);
//!
//! let user_id = uuid::Uuid::new_v4();
//! let torrent_id = uuid::Uuid::new_v4();
//!
//! // Purchase a freeleech token with bonus points
//! let token = freeleech_service.purchase_token(
//!     user_id,
//!     1000.0, // cost in bonus points
//! ).await?;
//!
//! // Activate token on a torrent
//! freeleech_service.activate_token(token.id, user_id, torrent_id).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Achievement System
//!
//! ```rust,no_run
//! use user::achievements::{AchievementService, AchievementCategory, Rarity};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let achievement_service = AchievementService::new(db_pool);
//!
//! let user_id = uuid::Uuid::new_v4();
//!
//! // Check and award achievements based on user activity
//! achievement_service.check_upload_achievements(user_id, 100).await?;
//!
//! // Get user's achievements
//! let achievements = achievement_service.get_user_achievements(user_id).await?;
//! for achievement in achievements {
//!     println!("🏆 {} - {}", achievement.name, achievement.description);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Privacy Controls
//!
//! ```rust,no_run
//! use user::privacy::{PrivacyService, PrivacySettings, PrivacyLevel};
//!
//! # async fn example(db_pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! let privacy_service = PrivacyService::new(db_pool);
//!
//! let user_id = uuid::Uuid::new_v4();
//!
//! // Set privacy level to paranoid
//! let settings = PrivacySettings::from_level(PrivacyLevel::Paranoid);
//! privacy_service.update_privacy_settings(user_id, settings).await?;
//!
//! // Check if a field is visible to another user
//! let viewer_id = uuid::Uuid::new_v4();
//! let can_view = privacy_service.can_view_ratio(user_id, Some(viewer_id)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features Overview
//!
//! ## Seedbonus System (Unit3d Pattern)
//!
//! The seedbonus system rewards users for seeding torrents with points that can be:
//! - Exchanged for upload credit
//! - Used to purchase freeleech tokens
//! - Given as tips to other users
//!
//! Bonus calculation is rule-based with support for:
//! - Torrent age multipliers
//! - Size-based bonuses
//! - Seeder/leecher ratio considerations
//! - Personal upload bonuses
//! - Operation types (append, multiply)
//!
//! ## Freeleech System
//!
//! Three-tier freeleech system:
//! 1. **Global Freeleech**: Applied to specific torrents (set by staff)
//! 2. **Personal Freeleech Tokens**: One-time use tokens purchased with bonus points
//! 3. **Temporary Freeleech Windows**: Time-limited freeleech periods
//!
//! ## Achievement System
//!
//! Gamification through achievements with:
//! - Multiple categories (upload, download, seeding, community)
//! - Rarity levels (common, rare, epic, legendary)
//! - Progress tracking for incremental achievements
//! - Visual badges displayed on user profiles
//!
//! ## Privacy Controls (Gazelle Paranoia System)
//!
//! Granular privacy settings allow users to hide:
//! - Upload/download stats and ratio
//! - Last seen timestamp
//! - Snatched torrent list
//! - Active seeding/leeching torrents
//! - Profile visibility from users/guests
//!
//! Privacy presets: Public, Normal, Paranoid
//! Staff members can override privacy settings when necessary.
//!
//! # Database Schema
//!
//! This crate expects the following database tables:
//!
//! - `user_profiles`: Extended user profile information
//! - `user_statistics`: Upload/download statistics and tracking
//! - `bonus_transactions`: Seedbonus earning and spending history
//! - `bonus_rules`: Rules for calculating seedbonus
//! - `freeleech_tokens`: Personal freeleech token inventory
//! - `achievements`: Achievement definitions
//! - `user_achievements`: User achievement progress and awards
//! - `privacy_settings`: User privacy preferences
//! - `invitations`: Invitation codes and tracking
//! - `user_follows`: User follow relationships
//!
//! See the `migrations/` directory for SQL schema definitions.

// Re-export commonly used types
pub use uuid::Uuid;

// Module declarations
pub mod achievements;
pub mod bonus;
pub mod follow;
pub mod freeleech;
pub mod invites;
pub mod privacy;
pub mod profile;
pub mod statistics;

// Re-export key types for convenience
pub use achievements::{
    Achievement, AchievementCategory, AchievementError, AchievementService, Rarity,
    UserAchievement,
};
pub use bonus::{
    BonusError, BonusOperation, BonusRule, BonusService, BonusTransaction, BonusTransactionType,
};
pub use follow::{FollowError, FollowService, UserFollow};
pub use freeleech::{
    FreeleechError, FreeleechService, FreeleechToken, FreeleechType, TokenStatus,
};
pub use invites::{Invitation, InviteError, InviteService, InviteTree};
pub use privacy::{PrivacyError, PrivacyLevel, PrivacyService, PrivacySettings};
pub use profile::{ProfileError, ProfileService, UpdateProfileRequest, UserProfile};
pub use statistics::{
    PeerTime, StatisticsError, StatisticsService, UploadDownloadHistory, UserStatistics,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User module for complete user management functionality
///
/// This is a convenience re-export that groups all user-related
/// functionality in one place.
pub mod user {
    pub use crate::achievements::*;
    pub use crate::bonus::*;
    pub use crate::follow::*;
    pub use crate::freeleech::*;
    pub use crate::invites::*;
    pub use crate::privacy::*;
    pub use crate::profile::*;
    pub use crate::statistics::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_exports() {
        // Verify key types are exported
        let _: Result<(), ProfileError> = Ok(());
        let _: Result<(), StatisticsError> = Ok(());
        let _: Result<(), BonusError> = Ok(());
        let _: Result<(), FreeleechError> = Ok(());
        let _: Result<(), AchievementError> = Ok(());
        let _: Result<(), PrivacyError> = Ok(());
        let _: Result<(), InviteError> = Ok(());
        let _: Result<(), FollowError> = Ok(());
    }
}
