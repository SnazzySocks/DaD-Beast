//! Seedbonus system (Unit3d pattern)
//!
//! This module implements a rule-based bonus earning system inspired by Unit3d.
//! Users earn bonus points for seeding torrents, which can be:
//! - Exchanged for upload credit
//! - Used to purchase freeleech tokens
//! - Sent as tips to other users
//!
//! Bonus calculation is rule-based with support for:
//! - Torrent age multipliers
//! - Size-based bonuses
//! - Seeder/leecher ratio considerations
//! - Personal upload bonuses
//! - Operation types (append, multiply)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

/// Bonus-related errors
#[derive(Debug, Error)]
pub enum BonusError {
    #[error("Insufficient bonus points: required {required}, available {available}")]
    InsufficientBonus { required: f64, available: f64 },

    #[error("Invalid bonus amount: {0}")]
    InvalidAmount(f64),

    #[error("Bonus rule not found: {0}")]
    RuleNotFound(Uuid),

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Bonus rule operation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BonusOperation {
    /// Add to the bonus (e.g., +5 points per hour)
    Append,
    /// Multiply the bonus (e.g., 2x multiplier)
    Multiply,
}

/// Bonus rule for calculating seedbonus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusRule {
    /// Rule ID
    pub id: Uuid,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Whether this rule is active
    pub active: bool,

    /// Minimum torrent age in hours (None = no minimum)
    pub min_torrent_age_hours: Option<i32>,

    /// Maximum torrent age in hours (None = no maximum)
    pub max_torrent_age_hours: Option<i32>,

    /// Minimum torrent size in bytes (None = no minimum)
    pub min_torrent_size: Option<i64>,

    /// Maximum torrent size in bytes (None = no maximum)
    pub max_torrent_size: Option<i64>,

    /// Minimum number of seeders (None = no minimum)
    pub min_seeders: Option<i32>,

    /// Maximum number of seeders (None = no maximum)
    pub max_seeders: Option<i32>,

    /// Minimum number of leechers (None = no minimum)
    pub min_leechers: Option<i32>,

    /// Whether this is a personal release (uploaded by user)
    pub personal_release: bool,

    /// Bonus amount
    pub bonus_amount: f64,

    /// Operation type (append or multiply)
    pub operation: BonusOperation,

    /// Priority (higher priority rules are applied first)
    pub priority: i32,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,

    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,
}

impl BonusRule {
    /// Check if this rule applies to the given torrent conditions
    pub fn applies_to(
        &self,
        torrent_age_hours: i32,
        torrent_size: i64,
        seeders: i32,
        leechers: i32,
        is_personal_release: bool,
    ) -> bool {
        if !self.active {
            return false;
        }

        // Check torrent age
        if let Some(min_age) = self.min_torrent_age_hours {
            if torrent_age_hours < min_age {
                return false;
            }
        }
        if let Some(max_age) = self.max_torrent_age_hours {
            if torrent_age_hours > max_age {
                return false;
            }
        }

        // Check torrent size
        if let Some(min_size) = self.min_torrent_size {
            if torrent_size < min_size {
                return false;
            }
        }
        if let Some(max_size) = self.max_torrent_size {
            if torrent_size > max_size {
                return false;
            }
        }

        // Check seeders
        if let Some(min_seeders) = self.min_seeders {
            if seeders < min_seeders {
                return false;
            }
        }
        if let Some(max_seeders) = self.max_seeders {
            if seeders > max_seeders {
                return false;
            }
        }

        // Check leechers
        if let Some(min_leechers) = self.min_leechers {
            if leechers < min_leechers {
                return false;
            }
        }

        // Check personal release requirement
        if self.personal_release && !is_personal_release {
            return false;
        }

        true
    }
}

/// Bonus transaction type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BonusTransactionType {
    /// Earned from seeding
    Earned,
    /// Exchanged for upload credit
    ExchangedForUpload,
    /// Used to purchase freeleech token
    PurchasedFreeleech,
    /// Sent as tip to another user
    TipSent,
    /// Received as tip from another user
    TipReceived,
    /// Manual adjustment by admin
    ManualAdjustment,
}

/// Bonus transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusTransaction {
    /// Transaction ID
    pub id: Uuid,

    /// User ID
    pub user_id: Uuid,

    /// Transaction type
    pub transaction_type: BonusTransactionType,

    /// Bonus amount (positive for credit, negative for debit)
    pub amount: f64,

    /// Balance after transaction
    pub balance_after: f64,

    /// Related torrent ID (for earned bonuses)
    pub torrent_id: Option<Uuid>,

    /// Related user ID (for tips)
    pub related_user_id: Option<Uuid>,

    /// Description
    pub description: String,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

/// Bonus service for managing seedbonus
pub struct BonusService {
    db: PgPool,
}

impl BonusService {
    /// Create a new bonus service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get all active bonus rules
    pub async fn get_active_rules(&self) -> Result<Vec<BonusRule>, BonusError> {
        let rules = sqlx::query_as!(
            BonusRule,
            r#"
            SELECT
                id,
                name,
                description,
                active,
                min_torrent_age_hours,
                max_torrent_age_hours,
                min_torrent_size,
                max_torrent_size,
                min_seeders,
                max_seeders,
                min_leechers,
                personal_release,
                bonus_amount,
                operation as "operation: BonusOperation",
                priority,
                created_at,
                updated_at
            FROM bonus_rules
            WHERE active = true
            ORDER BY priority DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rules)
    }

    /// Calculate bonus for a specific torrent based on rules
    ///
    /// # Arguments
    ///
    /// * `torrent_id` - The torrent ID
    /// * `user_id` - The user ID (for personal release check)
    /// * `duration_hours` - Duration in hours that the user has been seeding
    ///
    /// # Returns
    ///
    /// Returns the calculated bonus points
    pub async fn calculate_torrent_bonus(
        &self,
        torrent_id: Uuid,
        user_id: Uuid,
        duration_hours: f64,
    ) -> Result<f64, BonusError> {
        // Get torrent info
        let torrent = sqlx::query!(
            r#"
            SELECT
                size,
                created_at,
                uploader_id,
                (SELECT COUNT(*) FROM peers WHERE torrent_id = $1 AND seeder = true) as "seeders!",
                (SELECT COUNT(*) FROM peers WHERE torrent_id = $1 AND seeder = false) as "leechers!"
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&self.db)
        .await?;

        let torrent_age_hours =
            (Utc::now() - torrent.created_at).num_hours() as i32;
        let is_personal_release = torrent.uploader_id == user_id;

        // Get applicable rules
        let rules = self.get_active_rules().await?;

        // Calculate bonus using rules
        let mut bonus = 0.0;
        let mut multiplier = 1.0;

        for rule in rules {
            if rule.applies_to(
                torrent_age_hours,
                torrent.size,
                torrent.seeders as i32,
                torrent.leechers as i32,
                is_personal_release,
            ) {
                match rule.operation {
                    BonusOperation::Append => {
                        bonus += rule.bonus_amount;
                    }
                    BonusOperation::Multiply => {
                        multiplier *= rule.bonus_amount;
                    }
                }
            }
        }

        // Apply multiplier and duration
        bonus *= multiplier * duration_hours;

        Ok(bonus)
    }

    /// Calculate total bonus for all user's active seeds
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    ///
    /// # Returns
    ///
    /// Returns the total bonus earned
    pub async fn calculate_user_bonus(&self, user_id: Uuid) -> Result<f64, BonusError> {
        // Get all active seeding torrents for user
        let active_torrents = sqlx::query!(
            r#"
            SELECT torrent_id, seed_time
            FROM peer_times
            WHERE user_id = $1
            ORDER BY torrent_id
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut total_bonus = 0.0;

        for torrent in active_torrents {
            // Calculate bonus for each torrent (1 hour worth of seeding)
            let bonus = self
                .calculate_torrent_bonus(torrent.torrent_id, user_id, 1.0)
                .await?;
            total_bonus += bonus;
        }

        Ok(total_bonus)
    }

    /// Award bonus to a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `amount` - The bonus amount to award
    /// * `transaction_type` - The transaction type
    /// * `torrent_id` - Optional torrent ID
    /// * `description` - Transaction description
    pub async fn award_bonus(
        &self,
        user_id: Uuid,
        amount: f64,
        transaction_type: BonusTransactionType,
        torrent_id: Option<Uuid>,
        description: String,
    ) -> Result<BonusTransaction, BonusError> {
        if amount <= 0.0 {
            return Err(BonusError::InvalidAmount(amount));
        }

        let mut tx = self.db.begin().await?;

        // Update user's bonus balance
        let new_balance = sqlx::query_scalar!(
            r#"
            UPDATE user_statistics
            SET seedbonus = seedbonus + $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING seedbonus
            "#,
            user_id,
            amount
        )
        .fetch_one(&mut *tx)
        .await?;

        // Record transaction
        let transaction = sqlx::query_as!(
            BonusTransaction,
            r#"
            INSERT INTO bonus_transactions
                (id, user_id, transaction_type, amount, balance_after, torrent_id, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING
                id,
                user_id,
                transaction_type as "transaction_type: BonusTransactionType",
                amount,
                balance_after,
                torrent_id,
                related_user_id,
                description,
                created_at
            "#,
            Uuid::new_v4(),
            user_id,
            transaction_type as BonusTransactionType,
            amount,
            new_balance,
            torrent_id,
            description
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(transaction)
    }

    /// Deduct bonus from a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `amount` - The bonus amount to deduct
    /// * `transaction_type` - The transaction type
    /// * `description` - Transaction description
    pub async fn deduct_bonus(
        &self,
        user_id: Uuid,
        amount: f64,
        transaction_type: BonusTransactionType,
        description: String,
    ) -> Result<BonusTransaction, BonusError> {
        if amount <= 0.0 {
            return Err(BonusError::InvalidAmount(amount));
        }

        // Check if user has enough bonus
        let current_balance = sqlx::query_scalar!(
            r#"
            SELECT seedbonus
            FROM user_statistics
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(BonusError::UserNotFound(user_id))?;

        if current_balance < amount {
            return Err(BonusError::InsufficientBonus {
                required: amount,
                available: current_balance,
            });
        }

        let mut tx = self.db.begin().await?;

        // Update user's bonus balance
        let new_balance = sqlx::query_scalar!(
            r#"
            UPDATE user_statistics
            SET seedbonus = seedbonus - $2, updated_at = NOW()
            WHERE user_id = $1
            RETURNING seedbonus
            "#,
            user_id,
            amount
        )
        .fetch_one(&mut *tx)
        .await?;

        // Record transaction
        let transaction = sqlx::query_as!(
            BonusTransaction,
            r#"
            INSERT INTO bonus_transactions
                (id, user_id, transaction_type, amount, balance_after, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING
                id,
                user_id,
                transaction_type as "transaction_type: BonusTransactionType",
                amount,
                balance_after,
                torrent_id,
                related_user_id,
                description,
                created_at
            "#,
            Uuid::new_v4(),
            user_id,
            transaction_type as BonusTransactionType,
            -amount, // Negative for deduction
            new_balance,
            description
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(transaction)
    }

    /// Exchange bonus for upload credit
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `bonus_amount` - The bonus amount to exchange
    /// * `upload_bytes` - The upload credit in bytes to receive
    pub async fn exchange_bonus_for_upload(
        &self,
        user_id: Uuid,
        bonus_amount: f64,
        upload_bytes: i64,
    ) -> Result<(), BonusError> {
        let mut tx = self.db.begin().await?;

        // Deduct bonus
        self.deduct_bonus(
            user_id,
            bonus_amount,
            BonusTransactionType::ExchangedForUpload,
            format!("Exchanged {} bonus for {} bytes upload credit", bonus_amount, upload_bytes),
        )
        .await?;

        // Add upload credit
        sqlx::query!(
            r#"
            UPDATE user_statistics
            SET uploaded = uploaded + $2, updated_at = NOW()
            WHERE user_id = $1
            "#,
            user_id,
            upload_bytes
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Send bonus tip to another user
    ///
    /// # Arguments
    ///
    /// * `from_user_id` - The sender user ID
    /// * `to_user_id` - The recipient user ID
    /// * `amount` - The bonus amount to send
    /// * `message` - Optional message
    pub async fn send_tip(
        &self,
        from_user_id: Uuid,
        to_user_id: Uuid,
        amount: f64,
        message: Option<String>,
    ) -> Result<(), BonusError> {
        let mut tx = self.db.begin().await?;

        // Deduct from sender
        let description = format!(
            "Tip sent to user {}{}",
            to_user_id,
            message
                .as_ref()
                .map(|m| format!(": {}", m))
                .unwrap_or_default()
        );

        self.deduct_bonus(
            from_user_id,
            amount,
            BonusTransactionType::TipSent,
            description,
        )
        .await?;

        // Award to recipient
        let description = format!(
            "Tip received from user {}{}",
            from_user_id,
            message
                .as_ref()
                .map(|m| format!(": {}", m))
                .unwrap_or_default()
        );

        self.award_bonus(
            to_user_id,
            amount,
            BonusTransactionType::TipReceived,
            None,
            description,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get bonus transaction history
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `limit` - Maximum number of results
    /// * `offset` - Offset for pagination
    pub async fn get_transaction_history(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BonusTransaction>, BonusError> {
        let transactions = sqlx::query_as!(
            BonusTransaction,
            r#"
            SELECT
                id,
                user_id,
                transaction_type as "transaction_type: BonusTransactionType",
                amount,
                balance_after,
                torrent_id,
                related_user_id,
                description,
                created_at
            FROM bonus_transactions
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bonus_rule_applies_to() {
        let rule = BonusRule {
            id: Uuid::new_v4(),
            name: "Test Rule".to_string(),
            description: "Test".to_string(),
            active: true,
            min_torrent_age_hours: Some(24),
            max_torrent_age_hours: Some(168),
            min_torrent_size: Some(1024 * 1024 * 1024), // 1 GB
            max_torrent_size: None,
            min_seeders: None,
            max_seeders: Some(10),
            min_leechers: None,
            personal_release: false,
            bonus_amount: 5.0,
            operation: BonusOperation::Append,
            priority: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Should apply
        assert!(rule.applies_to(
            48,                    // 48 hours old
            2 * 1024 * 1024 * 1024, // 2 GB
            5,                     // 5 seeders
            2,                     // 2 leechers
            false                  // not personal
        ));

        // Too young
        assert!(!rule.applies_to(
            12,                    // 12 hours old
            2 * 1024 * 1024 * 1024,
            5,
            2,
            false
        ));

        // Too many seeders
        assert!(!rule.applies_to(
            48,
            2 * 1024 * 1024 * 1024,
            15, // 15 seeders
            2,
            false
        ));

        // Too small
        assert!(!rule.applies_to(
            48,
            512 * 1024 * 1024, // 512 MB
            5,
            2,
            false
        ));
    }

    #[test]
    fn test_bonus_operations() {
        assert_eq!(BonusOperation::Append, BonusOperation::Append);
        assert_eq!(BonusOperation::Multiply, BonusOperation::Multiply);
        assert_ne!(BonusOperation::Append, BonusOperation::Multiply);
    }

    #[test]
    fn test_transaction_types() {
        let types = vec![
            BonusTransactionType::Earned,
            BonusTransactionType::ExchangedForUpload,
            BonusTransactionType::PurchasedFreeleech,
            BonusTransactionType::TipSent,
            BonusTransactionType::TipReceived,
            BonusTransactionType::ManualAdjustment,
        ];

        // Ensure all types are distinct
        for (i, t1) in types.iter().enumerate() {
            for (j, t2) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(t1, t2);
                } else {
                    assert_ne!(t1, t2);
                }
            }
        }
    }
}
