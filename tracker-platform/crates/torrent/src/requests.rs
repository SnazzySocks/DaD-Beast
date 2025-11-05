//! Request/bounty system (Unit3d pattern)
//!
//! This module implements a comprehensive request system where users can:
//! - Create requests for missing content
//! - Add bounties to requests (multiple users can pool bounties)
//! - Fill requests by uploading matching content
//! - Automatically distribute bounties to uploader
//! - Vote on requests to show community interest

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::Validate;

/// Request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "request_status")]
pub enum RequestStatus {
    /// Open and accepting bounties
    Open,

    /// Filled by an upload
    Filled,

    /// Closed/cancelled by requester or admin
    Closed,

    /// Expired (no activity for too long)
    Expired,
}

/// Torrent request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TorrentRequest {
    /// Request ID
    pub id: Uuid,

    /// Requested by user
    pub requester_id: Uuid,

    /// Title/name of requested content
    #[validate(length(min = 3, max = 255))]
    pub title: String,

    /// Detailed description
    #[validate(length(max = 10000))]
    pub description: String,

    /// Category ID
    pub category_id: Uuid,

    /// Media type
    pub media_type: crate::metadata::MediaType,

    /// External IDs for identification
    pub external_ids: crate::metadata::ExternalIds,

    /// Year
    pub year: Option<i32>,

    /// Total bounty amount (sum of all contributions)
    pub total_bounty: i64,

    /// Number of bounty contributors
    pub bounty_contributors: i32,

    /// Number of votes
    pub vote_count: i32,

    /// Request status
    pub status: RequestStatus,

    /// Filled by torrent (if filled)
    pub filled_by_torrent_id: Option<Uuid>,

    /// Filled by user (if filled)
    pub filled_by_user_id: Option<Uuid>,

    /// When filled
    pub filled_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Expires at
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Bounty contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyContribution {
    /// Contribution ID
    pub id: Uuid,

    /// Request ID
    pub request_id: Uuid,

    /// User who contributed
    pub user_id: Uuid,

    /// Amount contributed (in bonus points)
    pub amount: i64,

    /// Refunded (if request cancelled)
    pub refunded: bool,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVote {
    /// Vote ID
    pub id: Uuid,

    /// Request ID
    pub request_id: Uuid,

    /// User who voted
    pub user_id: Uuid,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request creation input
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateRequestInput {
    /// Title
    #[validate(length(min = 3, max = 255))]
    pub title: String,

    /// Description
    #[validate(length(min = 10, max = 10000))]
    pub description: String,

    /// Category ID
    pub category_id: Uuid,

    /// Media type
    pub media_type: Option<crate::metadata::MediaType>,

    /// External IDs
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
    pub igdb_id: Option<i64>,

    /// Year
    pub year: Option<i32>,

    /// Initial bounty amount (optional)
    pub initial_bounty: Option<i64>,
}

/// Bounty addition input
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AddBountyInput {
    /// Request ID
    pub request_id: Uuid,

    /// Amount to add
    #[validate(range(min = 100, max = 1000000))]
    pub amount: i64,
}

/// Request fill input
#[derive(Debug, Clone, Deserialize)]
pub struct FillRequestInput {
    /// Request ID
    pub request_id: Uuid,

    /// Torrent ID that fills the request
    pub torrent_id: Uuid,
}

/// Request service
pub struct RequestService {
    pool: PgPool,
    min_bounty: i64,
    max_bounty_per_user: i64,
    request_expiry_days: i64,
}

impl RequestService {
    /// Create new request service
    pub fn new(
        pool: PgPool,
        min_bounty: i64,
        max_bounty_per_user: i64,
        request_expiry_days: i64,
    ) -> Self {
        Self {
            pool,
            min_bounty,
            max_bounty_per_user,
            request_expiry_days,
        }
    }

    /// Create a new request
    pub async fn create_request(
        &self,
        user_id: Uuid,
        input: CreateRequestInput,
    ) -> Result<TorrentRequest> {
        input.validate().context("Invalid request input")?;

        let mut tx = self.pool.begin().await?;

        let request_id = Uuid::new_v4();

        // Determine media type (use provided or default to Other)
        let media_type = input.media_type.unwrap_or(crate::metadata::MediaType::Other);

        // Build external IDs
        let external_ids = crate::metadata::ExternalIds {
            tmdb_id: input.tmdb_id,
            imdb_id: input.imdb_id,
            tvdb_id: input.tvdb_id,
            igdb_id: input.igdb_id,
            ..Default::default()
        };

        // Calculate expiry
        let expires_at = chrono::Utc::now()
            + chrono::Duration::days(self.request_expiry_days);

        // Insert request
        sqlx::query!(
            r#"
            INSERT INTO torrent_requests (
                id, requester_id, title, description, category_id,
                media_type, external_ids, year, total_bounty,
                bounty_contributors, vote_count, status, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, 0, 0, 'open', $9)
            "#,
            request_id,
            user_id,
            input.title,
            input.description,
            input.category_id,
            media_type as crate::metadata::MediaType,
            serde_json::to_value(&external_ids)?,
            input.year,
            expires_at,
        )
        .execute(&mut *tx)
        .await?;

        // Add initial bounty if provided
        let total_bounty = if let Some(bounty) = input.initial_bounty {
            if bounty < self.min_bounty {
                return Err(anyhow!(
                    "Initial bounty must be at least {} bonus points",
                    self.min_bounty
                ));
            }

            self.add_bounty_internal(&mut tx, request_id, user_id, bounty)
                .await?;

            bounty
        } else {
            0
        };

        tx.commit().await?;

        // Fetch and return the created request
        self.get_request(request_id)
            .await?
            .ok_or_else(|| anyhow!("Failed to fetch created request"))
    }

    /// Add bounty to a request
    pub async fn add_bounty(&self, user_id: Uuid, input: AddBountyInput) -> Result<()> {
        input.validate().context("Invalid bounty input")?;

        let mut tx = self.pool.begin().await?;

        // Check request exists and is open
        let request = sqlx::query!(
            r#"
            SELECT status as "status: RequestStatus"
            FROM torrent_requests
            WHERE id = $1
            "#,
            input.request_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Request not found"))?;

        if request.status != RequestStatus::Open {
            return Err(anyhow!("Cannot add bounty to a non-open request"));
        }

        // Check user's total contribution
        let user_total = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as "total!"
            FROM bounty_contributions
            WHERE request_id = $1 AND user_id = $2 AND refunded = false
            "#,
            input.request_id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?
        .total;

        if user_total + input.amount > self.max_bounty_per_user {
            return Err(anyhow!(
                "Total bounty per user cannot exceed {}",
                self.max_bounty_per_user
            ));
        }

        // Deduct bonus points from user
        self.deduct_bonus_points(&mut tx, user_id, input.amount)
            .await?;

        // Add bounty
        self.add_bounty_internal(&mut tx, input.request_id, user_id, input.amount)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Internal method to add bounty (within transaction)
    async fn add_bounty_internal(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        request_id: Uuid,
        user_id: Uuid,
        amount: i64,
    ) -> Result<()> {
        // Insert bounty contribution
        sqlx::query!(
            r#"
            INSERT INTO bounty_contributions (id, request_id, user_id, amount)
            VALUES ($1, $2, $3, $4)
            "#,
            Uuid::new_v4(),
            request_id,
            user_id,
            amount
        )
        .execute(&mut **tx)
        .await?;

        // Update request totals
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET
                total_bounty = total_bounty + $2,
                bounty_contributors = (
                    SELECT COUNT(DISTINCT user_id)
                    FROM bounty_contributions
                    WHERE request_id = $1 AND refunded = false
                )::int,
                updated_at = NOW()
            WHERE id = $1
            "#,
            request_id,
            amount
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Fill a request
    pub async fn fill_request(&self, input: FillRequestInput, filler_id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Check request exists and is open
        let request = sqlx::query!(
            r#"
            SELECT
                status as "status: RequestStatus",
                total_bounty
            FROM torrent_requests
            WHERE id = $1
            "#,
            input.request_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Request not found"))?;

        if request.status != RequestStatus::Open {
            return Err(anyhow!("Request is not open"));
        }

        // Check torrent exists and is approved
        let torrent = sqlx::query!(
            r#"
            SELECT
                uploader_id,
                moderation_status as "status: crate::moderation::ModerationStatus"
            FROM torrents
            WHERE id = $1
            "#,
            input.torrent_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Torrent not found"))?;

        if torrent.status != crate::moderation::ModerationStatus::Approved {
            return Err(anyhow!("Torrent must be approved"));
        }

        // Mark request as filled
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET
                status = 'filled',
                filled_by_torrent_id = $2,
                filled_by_user_id = $3,
                filled_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
            input.request_id,
            input.torrent_id,
            torrent.uploader_id
        )
        .execute(&mut *tx)
        .await?;

        // Award bounty to uploader
        if request.total_bounty > 0 {
            self.award_bonus_points(&mut tx, torrent.uploader_id, request.total_bounty)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    /// Vote on a request
    pub async fn vote_request(&self, request_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Insert vote (ignore if already exists)
        sqlx::query!(
            r#"
            INSERT INTO request_votes (id, request_id, user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (request_id, user_id) DO NOTHING
            "#,
            Uuid::new_v4(),
            request_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Update vote count
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET
                vote_count = (SELECT COUNT(*) FROM request_votes WHERE request_id = $1)::int,
                updated_at = NOW()
            WHERE id = $1
            "#,
            request_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Unvote a request
    pub async fn unvote_request(&self, request_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete vote
        sqlx::query!(
            r#"
            DELETE FROM request_votes
            WHERE request_id = $1 AND user_id = $2
            "#,
            request_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Update vote count
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET
                vote_count = (SELECT COUNT(*) FROM request_votes WHERE request_id = $1)::int,
                updated_at = NOW()
            WHERE id = $1
            "#,
            request_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Close/cancel a request
    pub async fn close_request(&self, request_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Check user is requester or admin
        let request = sqlx::query!(
            r#"
            SELECT requester_id, status as "status: RequestStatus"
            FROM torrent_requests
            WHERE id = $1
            "#,
            request_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Request not found"))?;

        // Only requester can close their own request
        // (admin check would be done in the handler)
        if request.requester_id != user_id {
            return Err(anyhow!("Only the requester can close this request"));
        }

        if request.status != RequestStatus::Open {
            return Err(anyhow!("Can only close open requests"));
        }

        // Close request
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET status = 'closed', updated_at = NOW()
            WHERE id = $1
            "#,
            request_id
        )
        .execute(&mut *tx)
        .await?;

        // Refund all bounties
        self.refund_bounties(&mut tx, request_id).await?;

        tx.commit().await?;

        Ok(())
    }

    /// Refund bounties for a request
    async fn refund_bounties(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        request_id: Uuid,
    ) -> Result<()> {
        // Get all contributions
        let contributions = sqlx::query!(
            r#"
            SELECT id, user_id, amount
            FROM bounty_contributions
            WHERE request_id = $1 AND refunded = false
            "#,
            request_id
        )
        .fetch_all(&mut **tx)
        .await?;

        // Refund each contribution
        for contribution in contributions {
            self.award_bonus_points(&mut **tx, contribution.user_id, contribution.amount)
                .await?;

            sqlx::query!(
                r#"
                UPDATE bounty_contributions
                SET refunded = true
                WHERE id = $1
                "#,
                contribution.id
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    /// Get a request by ID
    pub async fn get_request(&self, request_id: Uuid) -> Result<Option<TorrentRequest>> {
        let record = sqlx::query!(
            r#"
            SELECT
                id, requester_id, title, description, category_id,
                media_type as "media_type: crate::metadata::MediaType",
                external_ids, year, total_bounty, bounty_contributors,
                vote_count, status as "status: RequestStatus",
                filled_by_torrent_id, filled_by_user_id, filled_at,
                created_at, updated_at, expires_at
            FROM torrent_requests
            WHERE id = $1
            "#,
            request_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(r) => {
                let external_ids: crate::metadata::ExternalIds = r
                    .external_ids
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                Ok(Some(TorrentRequest {
                    id: r.id,
                    requester_id: r.requester_id,
                    title: r.title,
                    description: r.description,
                    category_id: r.category_id,
                    media_type: r.media_type,
                    external_ids,
                    year: r.year,
                    total_bounty: r.total_bounty,
                    bounty_contributors: r.bounty_contributors,
                    vote_count: r.vote_count,
                    status: r.status,
                    filled_by_torrent_id: r.filled_by_torrent_id,
                    filled_by_user_id: r.filled_by_user_id,
                    filled_at: r.filled_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    expires_at: r.expires_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get bounty contributions for a request
    pub async fn get_contributions(&self, request_id: Uuid) -> Result<Vec<BountyContribution>> {
        let records = sqlx::query!(
            r#"
            SELECT id, request_id, user_id, amount, refunded, created_at
            FROM bounty_contributions
            WHERE request_id = $1
            ORDER BY created_at DESC
            "#,
            request_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| BountyContribution {
                id: r.id,
                request_id: r.request_id,
                user_id: r.user_id,
                amount: r.amount,
                refunded: r.refunded,
                created_at: r.created_at,
            })
            .collect())
    }

    /// List open requests
    pub async fn list_requests(
        &self,
        limit: i64,
        offset: i64,
        status: Option<RequestStatus>,
    ) -> Result<Vec<TorrentRequest>> {
        let records = if let Some(status) = status {
            sqlx::query!(
                r#"
                SELECT
                    id, requester_id, title, description, category_id,
                    media_type as "media_type: crate::metadata::MediaType",
                    external_ids, year, total_bounty, bounty_contributors,
                    vote_count, status as "status: RequestStatus",
                    filled_by_torrent_id, filled_by_user_id, filled_at,
                    created_at, updated_at, expires_at
                FROM torrent_requests
                WHERE status = $1
                ORDER BY total_bounty DESC, vote_count DESC, created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                status as RequestStatus,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT
                    id, requester_id, title, description, category_id,
                    media_type as "media_type: crate::metadata::MediaType",
                    external_ids, year, total_bounty, bounty_contributors,
                    vote_count, status as "status: RequestStatus",
                    filled_by_torrent_id, filled_by_user_id, filled_at,
                    created_at, updated_at, expires_at
                FROM torrent_requests
                ORDER BY total_bounty DESC, vote_count DESC, created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(records
            .into_iter()
            .map(|r| {
                let external_ids: crate::metadata::ExternalIds = r
                    .external_ids
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                TorrentRequest {
                    id: r.id,
                    requester_id: r.requester_id,
                    title: r.title,
                    description: r.description,
                    category_id: r.category_id,
                    media_type: r.media_type,
                    external_ids,
                    year: r.year,
                    total_bounty: r.total_bounty,
                    bounty_contributors: r.bounty_contributors,
                    vote_count: r.vote_count,
                    status: r.status,
                    filled_by_torrent_id: r.filled_by_torrent_id,
                    filled_by_user_id: r.filled_by_user_id,
                    filled_at: r.filled_at,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    expires_at: r.expires_at,
                }
            })
            .collect())
    }

    /// Deduct bonus points from user
    async fn deduct_bonus_points(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        amount: i64,
    ) -> Result<()> {
        let result = sqlx::query!(
            r#"
            UPDATE user_stats
            SET bonus_points = bonus_points - $2
            WHERE user_id = $1 AND bonus_points >= $2
            RETURNING bonus_points
            "#,
            user_id,
            amount
        )
        .fetch_optional(&mut **tx)
        .await?;

        if result.is_none() {
            return Err(anyhow!("Insufficient bonus points"));
        }

        Ok(())
    }

    /// Award bonus points to user
    async fn award_bonus_points(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
        amount: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE user_stats
            SET bonus_points = bonus_points + $2
            WHERE user_id = $1
            "#,
            user_id,
            amount
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_status() {
        assert_eq!(RequestStatus::Open as i32, 0);
        assert!(RequestStatus::Open != RequestStatus::Filled);
    }
}
