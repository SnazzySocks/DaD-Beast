//! Achievement and badge system
//!
//! This module implements a gamification system with achievements/badges.
//! Users can earn achievements by:
//! - Uploading torrents
//! - Maintaining high ratios
//! - Long-term seeding
//! - Community participation
//! - Special milestones
//!
//! Achievements have rarity levels and are displayed on user profiles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Achievement-related errors
#[derive(Debug, Error)]
pub enum AchievementError {
    #[error("Achievement not found: {0}")]
    NotFound(Uuid),

    #[error("Achievement already awarded to user")]
    AlreadyAwarded,

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Invalid progress value: {0}")]
    InvalidProgress(i32),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Achievement category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AchievementCategory {
    /// Upload-related achievements
    Upload,
    /// Download-related achievements
    Download,
    /// Seeding-related achievements
    Seeding,
    /// Community participation achievements
    Community,
    /// Ratio achievements
    Ratio,
    /// Special/rare achievements
    Special,
}

/// Achievement rarity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    /// Common achievement (easy to get)
    Common,
    /// Uncommon achievement
    Uncommon,
    /// Rare achievement
    Rare,
    /// Epic achievement (very hard to get)
    Epic,
    /// Legendary achievement (extremely rare)
    Legendary,
}

/// Achievement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    /// Achievement ID
    pub id: Uuid,

    /// Achievement name
    pub name: String,

    /// Achievement description
    pub description: String,

    /// Category
    pub category: AchievementCategory,

    /// Rarity level
    pub rarity: Rarity,

    /// Badge icon URL or emoji
    pub icon: String,

    /// Whether this is a hidden achievement (not shown until earned)
    pub hidden: bool,

    /// Points value
    pub points: i32,

    /// Target value for progress-based achievements (None = not progress-based)
    pub target_value: Option<i32>,

    /// Whether this achievement is currently active
    pub active: bool,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,

    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

/// User achievement (awarded or in progress)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    /// User achievement ID
    pub id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// Achievement ID
    pub achievement_id: Uuid,

    /// Achievement details (joined)
    pub achievement: Option<Achievement>,

    /// Current progress (for progress-based achievements)
    pub progress: i32,

    /// Whether the achievement has been awarded
    pub awarded: bool,

    /// Awarded at timestamp
    pub awarded_at: Option<DateTime<Utc>>,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,

    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

impl UserAchievement {
    /// Calculate progress percentage
    pub fn progress_percentage(&self) -> Option<f64> {
        if let Some(achievement) = &self.achievement {
            if let Some(target) = achievement.target_value {
                return Some((self.progress as f64 / target as f64 * 100.0).min(100.0));
            }
        }
        None
    }

    /// Check if achievement is completed
    pub fn is_completed(&self) -> bool {
        if let Some(achievement) = &self.achievement {
            if let Some(target) = achievement.target_value {
                return self.progress >= target;
            }
        }
        self.awarded
    }
}

/// Achievement service for managing achievements
pub struct AchievementService {
    db: PgPool,
}

impl AchievementService {
    /// Create a new achievement service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get all active achievements
    pub async fn get_active_achievements(&self) -> Result<Vec<Achievement>, AchievementError> {
        let achievements = sqlx::query_as!(
            Achievement,
            r#"
            SELECT
                id,
                name,
                description,
                category as "category: AchievementCategory",
                rarity as "rarity: Rarity",
                icon,
                hidden,
                points,
                target_value,
                active,
                created_at,
                updated_at
            FROM achievements
            WHERE active = true
            ORDER BY rarity DESC, points DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(achievements)
    }

    /// Get user's achievements
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `include_hidden` - Whether to include hidden achievements not yet earned
    pub async fn get_user_achievements(
        &self,
        user_id: Uuid,
        include_hidden: bool,
    ) -> Result<Vec<UserAchievement>, AchievementError> {
        let achievements = if include_hidden {
            sqlx::query!(
                r#"
                SELECT
                    ua.id,
                    ua.user_id,
                    ua.achievement_id,
                    ua.progress,
                    ua.awarded,
                    ua.awarded_at,
                    ua.created_at,
                    ua.updated_at,
                    a.id as a_id,
                    a.name,
                    a.description,
                    a.category as "category: AchievementCategory",
                    a.rarity as "rarity: Rarity",
                    a.icon,
                    a.hidden,
                    a.points,
                    a.target_value,
                    a.active,
                    a.created_at as a_created_at,
                    a.updated_at as a_updated_at
                FROM user_achievements ua
                JOIN achievements a ON ua.achievement_id = a.id
                WHERE ua.user_id = $1
                ORDER BY ua.awarded_at DESC NULLS LAST, a.rarity DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT
                    ua.id,
                    ua.user_id,
                    ua.achievement_id,
                    ua.progress,
                    ua.awarded,
                    ua.awarded_at,
                    ua.created_at,
                    ua.updated_at,
                    a.id as a_id,
                    a.name,
                    a.description,
                    a.category as "category: AchievementCategory",
                    a.rarity as "rarity: Rarity",
                    a.icon,
                    a.hidden,
                    a.points,
                    a.target_value,
                    a.active,
                    a.created_at as a_created_at,
                    a.updated_at as a_updated_at
                FROM user_achievements ua
                JOIN achievements a ON ua.achievement_id = a.id
                WHERE ua.user_id = $1 AND (a.hidden = false OR ua.awarded = true)
                ORDER BY ua.awarded_at DESC NULLS LAST, a.rarity DESC
                "#,
                user_id
            )
            .fetch_all(&self.db)
            .await?
        };

        let user_achievements = achievements
            .into_iter()
            .map(|row| UserAchievement {
                id: row.id,
                user_id: row.user_id,
                achievement_id: row.achievement_id,
                achievement: Some(Achievement {
                    id: row.a_id,
                    name: row.name,
                    description: row.description,
                    category: row.category,
                    rarity: row.rarity,
                    icon: row.icon,
                    hidden: row.hidden,
                    points: row.points,
                    target_value: row.target_value,
                    active: row.active,
                    created_at: row.a_created_at,
                    updated_at: row.a_updated_at,
                }),
                progress: row.progress,
                awarded: row.awarded,
                awarded_at: row.awarded_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(user_achievements)
    }

    /// Award an achievement to a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `achievement_id` - The achievement ID
    pub async fn award_achievement(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
    ) -> Result<UserAchievement, AchievementError> {
        // Check if already awarded
        let existing = sqlx::query!(
            r#"
            SELECT id, awarded
            FROM user_achievements
            WHERE user_id = $1 AND achievement_id = $2
            "#,
            user_id,
            achievement_id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(existing) = existing {
            if existing.awarded {
                return Err(AchievementError::AlreadyAwarded);
            }

            // Update existing record to awarded
            sqlx::query!(
                r#"
                UPDATE user_achievements
                SET awarded = true, awarded_at = NOW(), updated_at = NOW()
                WHERE id = $1
                "#,
                existing.id
            )
            .execute(&self.db)
            .await?;
        } else {
            // Create new user achievement record
            sqlx::query!(
                r#"
                INSERT INTO user_achievements
                    (id, user_id, achievement_id, progress, awarded, awarded_at, created_at, updated_at)
                VALUES ($1, $2, $3, 0, true, NOW(), NOW(), NOW())
                "#,
                Uuid::new_v4(),
                user_id,
                achievement_id,
                0
            )
            .execute(&self.db)
            .await?;
        }

        // Fetch and return the awarded achievement
        let user_achievements = self.get_user_achievements(user_id, true).await?;
        user_achievements
            .into_iter()
            .find(|ua| ua.achievement_id == achievement_id)
            .ok_or(AchievementError::NotFound(achievement_id))
    }

    /// Update progress for a user achievement
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `achievement_id` - The achievement ID
    /// * `progress` - The new progress value
    pub async fn update_progress(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
        progress: i32,
    ) -> Result<UserAchievement, AchievementError> {
        if progress < 0 {
            return Err(AchievementError::InvalidProgress(progress));
        }

        // Get achievement to check target value
        let achievement = sqlx::query_as!(
            Achievement,
            r#"
            SELECT
                id,
                name,
                description,
                category as "category: AchievementCategory",
                rarity as "rarity: Rarity",
                icon,
                hidden,
                points,
                target_value,
                active,
                created_at,
                updated_at
            FROM achievements
            WHERE id = $1
            "#,
            achievement_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AchievementError::NotFound(achievement_id))?;

        // Check if progress meets target
        let should_award = achievement
            .target_value
            .map(|target| progress >= target)
            .unwrap_or(false);

        // Upsert user achievement
        sqlx::query!(
            r#"
            INSERT INTO user_achievements
                (id, user_id, achievement_id, progress, awarded, awarded_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (user_id, achievement_id)
            DO UPDATE SET
                progress = $4,
                awarded = CASE WHEN user_achievements.awarded THEN true ELSE $5 END,
                awarded_at = CASE WHEN $5 AND user_achievements.awarded_at IS NULL THEN NOW() ELSE user_achievements.awarded_at END,
                updated_at = NOW()
            "#,
            Uuid::new_v4(),
            user_id,
            achievement_id,
            progress,
            should_award,
            if should_award { Some(Utc::now()) } else { None }
        )
        .execute(&self.db)
        .await?;

        // Fetch and return updated achievement
        let user_achievements = self.get_user_achievements(user_id, true).await?;
        user_achievements
            .into_iter()
            .find(|ua| ua.achievement_id == achievement_id)
            .ok_or(AchievementError::NotFound(achievement_id))
    }

    /// Check and award upload achievements
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `upload_count` - Current upload count
    pub async fn check_upload_achievements(
        &self,
        user_id: Uuid,
        upload_count: i32,
    ) -> Result<Vec<UserAchievement>, AchievementError> {
        // Get upload achievements
        let achievements = sqlx::query_as!(
            Achievement,
            r#"
            SELECT
                id,
                name,
                description,
                category as "category: AchievementCategory",
                rarity as "rarity: Rarity",
                icon,
                hidden,
                points,
                target_value,
                active,
                created_at,
                updated_at
            FROM achievements
            WHERE category = 'upload' AND active = true
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let mut awarded = Vec::new();

        for achievement in achievements {
            if let Some(target) = achievement.target_value {
                if upload_count >= target {
                    // Try to award (will fail silently if already awarded)
                    if let Ok(user_achievement) =
                        self.award_achievement(user_id, achievement.id).await
                    {
                        awarded.push(user_achievement);
                    }
                } else {
                    // Update progress
                    self.update_progress(user_id, achievement.id, upload_count)
                        .await?;
                }
            }
        }

        Ok(awarded)
    }

    /// Check and award ratio achievements
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `ratio` - Current ratio
    pub async fn check_ratio_achievements(
        &self,
        user_id: Uuid,
        ratio: f64,
    ) -> Result<Vec<UserAchievement>, AchievementError> {
        // Get ratio achievements
        let achievements = sqlx::query_as!(
            Achievement,
            r#"
            SELECT
                id,
                name,
                description,
                category as "category: AchievementCategory",
                rarity as "rarity: Rarity",
                icon,
                hidden,
                points,
                target_value,
                active,
                created_at,
                updated_at
            FROM achievements
            WHERE category = 'ratio' AND active = true
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let mut awarded = Vec::new();

        for achievement in achievements {
            if let Some(target) = achievement.target_value {
                if ratio >= target as f64 {
                    if let Ok(user_achievement) =
                        self.award_achievement(user_id, achievement.id).await
                    {
                        awarded.push(user_achievement);
                    }
                }
            }
        }

        Ok(awarded)
    }

    /// Get achievement leaderboard (users with most achievement points)
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    pub async fn get_achievement_leaderboard(
        &self,
        limit: i64,
    ) -> Result<Vec<(Uuid, i32)>, AchievementError> {
        let leaderboard = sqlx::query!(
            r#"
            SELECT
                ua.user_id,
                SUM(a.points) as "total_points!"
            FROM user_achievements ua
            JOIN achievements a ON ua.achievement_id = a.id
            WHERE ua.awarded = true
            GROUP BY ua.user_id
            ORDER BY total_points DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(leaderboard
            .into_iter()
            .map(|row| (row.user_id, row.total_points))
            .collect())
    }

    /// Get user's total achievement points
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    pub async fn get_user_achievement_points(
        &self,
        user_id: Uuid,
    ) -> Result<i32, AchievementError> {
        let total = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(a.points), 0) as "total!"
            FROM user_achievements ua
            JOIN achievements a ON ua.achievement_id = a.id
            WHERE ua.user_id = $1 AND ua.awarded = true
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Common < Rarity::Uncommon);
        assert!(Rarity::Uncommon < Rarity::Rare);
        assert!(Rarity::Rare < Rarity::Epic);
        assert!(Rarity::Epic < Rarity::Legendary);
    }

    #[test]
    fn test_user_achievement_progress_percentage() {
        let achievement = Achievement {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            category: AchievementCategory::Upload,
            rarity: Rarity::Common,
            icon: "📤".to_string(),
            hidden: false,
            points: 10,
            target_value: Some(100),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_achievement = UserAchievement {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            achievement_id: achievement.id,
            achievement: Some(achievement),
            progress: 50,
            awarded: false,
            awarded_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(user_achievement.progress_percentage(), Some(50.0));
    }

    #[test]
    fn test_user_achievement_is_completed() {
        let achievement = Achievement {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            category: AchievementCategory::Upload,
            rarity: Rarity::Common,
            icon: "📤".to_string(),
            hidden: false,
            points: 10,
            target_value: Some(100),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut user_achievement = UserAchievement {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            achievement_id: achievement.id,
            achievement: Some(achievement.clone()),
            progress: 50,
            awarded: false,
            awarded_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(!user_achievement.is_completed());

        user_achievement.progress = 100;
        assert!(user_achievement.is_completed());

        user_achievement.progress = 150;
        assert!(user_achievement.is_completed());
    }

    #[test]
    fn test_achievement_categories() {
        let categories = vec![
            AchievementCategory::Upload,
            AchievementCategory::Download,
            AchievementCategory::Seeding,
            AchievementCategory::Community,
            AchievementCategory::Ratio,
            AchievementCategory::Special,
        ];

        // Ensure all categories are distinct
        for (i, c1) in categories.iter().enumerate() {
            for (j, c2) in categories.iter().enumerate() {
                if i == j {
                    assert_eq!(c1, c2);
                } else {
                    assert_ne!(c1, c2);
                }
            }
        }
    }
}
