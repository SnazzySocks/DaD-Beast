//! Polls and Voting
//!
//! Provides polling functionality with multiple choice support,
//! vote tracking, and configurable poll settings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Poll-related errors
#[derive(Debug, Error)]
pub enum PollError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Poll not found: {0}")]
    NotFound(Uuid),

    #[error("Option not found: {0}")]
    OptionNotFound(Uuid),

    #[error("Poll has expired")]
    PollExpired,

    #[error("Already voted")]
    AlreadyVoted,

    #[error("Cannot change vote")]
    CannotChangeVote,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Invalid poll options: {0}")]
    InvalidOptions(String),
}

pub type Result<T> = std::result::Result<T, PollError>;

/// Poll
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Poll {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,

    // Settings
    pub allow_multiple_choice: bool,
    pub max_choices: Option<i32>,
    pub allow_vote_change: bool,
    pub show_results_before_vote: bool,
    pub is_anonymous: bool,

    // Status
    pub is_closed: bool,
    pub expires_at: Option<DateTime<Utc>>,

    // Statistics
    pub total_votes: i32,
    pub total_voters: i32,

    // Association
    pub topic_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Poll with options and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollWithResults {
    #[serde(flatten)]
    pub poll: Poll,
    pub options: Vec<PollOptionWithVotes>,
    pub user_votes: Vec<Uuid>, // Option IDs user voted for
}

/// Poll option
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PollOption {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_text: String,
    pub sort_order: i32,
    pub vote_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Poll option with vote percentage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOptionWithVotes {
    #[serde(flatten)]
    pub option: PollOption,
    pub percentage: f64,
}

/// Poll vote
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PollVote {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Vote change policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChangePolicy {
    Never,
    Always,
    BeforeExpiry,
}

/// Request to create a poll
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePollRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    #[validate(length(min = 2))]
    pub options: Vec<String>,

    pub allow_multiple_choice: bool,
    pub max_choices: Option<i32>,
    pub allow_vote_change: bool,
    pub show_results_before_vote: bool,
    pub is_anonymous: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub topic_id: Option<Uuid>,
}

/// Request to vote on a poll
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VoteRequest {
    pub poll_id: Uuid,
    pub user_id: Uuid,

    #[validate(length(min = 1))]
    pub option_ids: Vec<Uuid>,
}

/// Request to update a poll
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePollRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    pub is_closed: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Poll statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollStatistics {
    pub total_votes: i32,
    pub total_voters: i32,
    pub options_count: i32,
    pub most_popular_option: Option<String>,
    pub most_popular_votes: i32,
}

/// Poll service for managing polls
pub struct PollService {
    db: PgPool,
}

impl PollService {
    /// Creates a new poll service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new poll
    pub async fn create_poll(
        &self,
        created_by: Uuid,
        request: CreatePollRequest,
    ) -> Result<Poll> {
        request.validate()?;

        // Validate options
        if request.options.len() < 2 {
            return Err(PollError::InvalidOptions(
                "Poll must have at least 2 options".to_string(),
            ));
        }

        if request.allow_multiple_choice {
            if let Some(max) = request.max_choices {
                if max < 1 || max > request.options.len() as i32 {
                    return Err(PollError::InvalidOptions(
                        "Invalid max_choices value".to_string(),
                    ));
                }
            }
        }

        let mut tx = self.db.begin().await?;

        // Create poll
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            INSERT INTO polls (
                id, title, description, created_by,
                allow_multiple_choice, max_choices, allow_vote_change,
                show_results_before_vote, is_anonymous, is_closed,
                expires_at, total_votes, total_voters, topic_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, $10, 0, 0, $11, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.title)
        .bind(&request.description)
        .bind(created_by)
        .bind(request.allow_multiple_choice)
        .bind(request.max_choices)
        .bind(request.allow_vote_change)
        .bind(request.show_results_before_vote)
        .bind(request.is_anonymous)
        .bind(request.expires_at)
        .bind(request.topic_id)
        .fetch_one(&mut *tx)
        .await?;

        // Create options
        for (index, option_text) in request.options.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO poll_options (id, poll_id, option_text, sort_order, vote_count, created_at)
                VALUES ($1, $2, $3, $4, 0, NOW())
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(poll.id)
            .bind(option_text)
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;
        }

        // If associated with topic, update topic
        if let Some(topic_id) = request.topic_id {
            sqlx::query(
                "UPDATE topics SET has_poll = true, poll_id = $2 WHERE id = $1",
            )
            .bind(topic_id)
            .bind(poll.id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(poll)
    }

    /// Gets a poll by ID
    pub async fn get_poll(&self, poll_id: Uuid) -> Result<Poll> {
        let poll = sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = $1")
            .bind(poll_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(PollError::NotFound(poll_id))?;

        Ok(poll)
    }

    /// Gets a poll with results
    pub async fn get_poll_with_results(
        &self,
        poll_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<PollWithResults> {
        let poll = self.get_poll(poll_id).await?;

        // Get options
        let options = sqlx::query_as::<_, PollOption>(
            "SELECT * FROM poll_options WHERE poll_id = $1 ORDER BY sort_order",
        )
        .bind(poll_id)
        .fetch_all(&self.db)
        .await?;

        // Calculate percentages
        let mut options_with_votes = Vec::new();
        for option in options {
            let percentage = if poll.total_votes > 0 {
                (option.vote_count as f64 / poll.total_votes as f64) * 100.0
            } else {
                0.0
            };

            options_with_votes.push(PollOptionWithVotes {
                option,
                percentage,
            });
        }

        // Get user's votes if logged in
        let user_votes = if let Some(uid) = user_id {
            sqlx::query_scalar::<_, Uuid>(
                "SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2",
            )
            .bind(poll_id)
            .bind(uid)
            .fetch_all(&self.db)
            .await?
        } else {
            Vec::new()
        };

        Ok(PollWithResults {
            poll,
            options: options_with_votes,
            user_votes,
        })
    }

    /// Votes on a poll
    pub async fn vote(&self, request: VoteRequest) -> Result<()> {
        request.validate()?;

        let poll = self.get_poll(request.poll_id).await?;

        // Check if poll is closed
        if poll.is_closed {
            return Err(PollError::PollExpired);
        }

        // Check if poll has expired
        if let Some(expires_at) = poll.expires_at {
            if expires_at < Utc::now() {
                return Err(PollError::PollExpired);
            }
        }

        // Check if user has already voted
        let has_voted = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM poll_votes WHERE poll_id = $1 AND user_id = $2)",
        )
        .bind(request.poll_id)
        .bind(request.user_id)
        .fetch_one(&self.db)
        .await?;

        if has_voted && !poll.allow_vote_change {
            return Err(PollError::AlreadyVoted);
        }

        // Validate number of choices
        if !poll.allow_multiple_choice && request.option_ids.len() > 1 {
            return Err(PollError::InvalidOptions(
                "Multiple choices not allowed".to_string(),
            ));
        }

        if let Some(max) = poll.max_choices {
            if request.option_ids.len() > max as usize {
                return Err(PollError::InvalidOptions(format!(
                    "Maximum {} choices allowed",
                    max
                )));
            }
        }

        // Verify all options belong to this poll
        for option_id in &request.option_ids {
            let valid = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM poll_options WHERE id = $1 AND poll_id = $2)",
            )
            .bind(option_id)
            .bind(request.poll_id)
            .fetch_one(&self.db)
            .await?;

            if !valid {
                return Err(PollError::OptionNotFound(*option_id));
            }
        }

        let mut tx = self.db.begin().await?;

        // Remove old votes if changing vote
        if has_voted {
            // Decrement old option counts
            sqlx::query(
                r#"
                UPDATE poll_options
                SET vote_count = vote_count - 1
                WHERE id IN (SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2)
                "#,
            )
            .bind(request.poll_id)
            .bind(request.user_id)
            .execute(&mut *tx)
            .await?;

            // Delete old votes
            sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2")
                .bind(request.poll_id)
                .bind(request.user_id)
                .execute(&mut *tx)
                .await?;
        }

        // Add new votes
        for option_id in &request.option_ids {
            sqlx::query(
                r#"
                INSERT INTO poll_votes (id, poll_id, option_id, user_id, created_at)
                VALUES ($1, $2, $3, $4, NOW())
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(request.poll_id)
            .bind(option_id)
            .bind(request.user_id)
            .execute(&mut *tx)
            .await?;

            // Increment option count
            sqlx::query("UPDATE poll_options SET vote_count = vote_count + 1 WHERE id = $1")
                .bind(option_id)
                .execute(&mut *tx)
                .await?;
        }

        // Update poll statistics
        let (total_votes, total_voters) = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*) as total_votes,
                COUNT(DISTINCT user_id) as total_voters
            FROM poll_votes
            WHERE poll_id = $1
            "#,
        )
        .bind(request.poll_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE polls SET total_votes = $2, total_voters = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(request.poll_id)
        .bind(total_votes as i32)
        .bind(total_voters as i32)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Removes a vote
    pub async fn remove_vote(&self, poll_id: Uuid, user_id: Uuid) -> Result<()> {
        let poll = self.get_poll(poll_id).await?;

        if !poll.allow_vote_change {
            return Err(PollError::CannotChangeVote);
        }

        let mut tx = self.db.begin().await?;

        // Decrement option counts
        sqlx::query(
            r#"
            UPDATE poll_options
            SET vote_count = vote_count - 1
            WHERE id IN (SELECT option_id FROM poll_votes WHERE poll_id = $1 AND user_id = $2)
            "#,
        )
        .bind(poll_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Delete votes
        sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1 AND user_id = $2")
            .bind(poll_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Update poll statistics
        let (total_votes, total_voters) = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COUNT(*) as total_votes,
                COUNT(DISTINCT user_id) as total_voters
            FROM poll_votes
            WHERE poll_id = $1
            "#,
        )
        .bind(poll_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE polls SET total_votes = $2, total_voters = $3, updated_at = NOW() WHERE id = $1",
        )
        .bind(poll_id)
        .bind(total_votes as i32)
        .bind(total_voters as i32)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Updates a poll
    pub async fn update_poll(
        &self,
        poll_id: Uuid,
        user_id: Uuid,
        is_moderator: bool,
        request: UpdatePollRequest,
    ) -> Result<Poll> {
        request.validate()?;

        let poll = self.get_poll(poll_id).await?;

        // Check permissions
        if poll.created_by != user_id && !is_moderator {
            return Err(PollError::PermissionDenied);
        }

        let updated_poll = sqlx::query_as::<_, Poll>(
            r#"
            UPDATE polls
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                is_closed = COALESCE($4, is_closed),
                expires_at = COALESCE($5, expires_at),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(poll_id)
        .bind(request.title)
        .bind(request.description.or(Some(poll.description)))
        .bind(request.is_closed)
        .bind(request.expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(updated_poll)
    }

    /// Closes a poll
    pub async fn close_poll(&self, poll_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE polls SET is_closed = true, updated_at = NOW() WHERE id = $1")
            .bind(poll_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Gets poll statistics
    pub async fn get_statistics(&self, poll_id: Uuid) -> Result<PollStatistics> {
        let poll = self.get_poll(poll_id).await?;

        let options = sqlx::query_as::<_, PollOption>(
            "SELECT * FROM poll_options WHERE poll_id = $1 ORDER BY vote_count DESC",
        )
        .bind(poll_id)
        .fetch_all(&self.db)
        .await?;

        let (most_popular_option, most_popular_votes) = options
            .first()
            .map(|opt| (Some(opt.option_text.clone()), opt.vote_count))
            .unwrap_or((None, 0));

        Ok(PollStatistics {
            total_votes: poll.total_votes,
            total_voters: poll.total_voters,
            options_count: options.len() as i32,
            most_popular_option,
            most_popular_votes,
        })
    }

    /// Gets voters for a poll (if not anonymous)
    pub async fn get_voters(&self, poll_id: Uuid) -> Result<Vec<(Uuid, String, Vec<String>)>> {
        let poll = self.get_poll(poll_id).await?;

        if poll.is_anonymous {
            return Err(PollError::PermissionDenied);
        }

        let votes = sqlx::query_as::<_, (Uuid, Uuid, String)>(
            r#"
            SELECT DISTINCT pv.user_id, pv.option_id, po.option_text
            FROM poll_votes pv
            JOIN poll_options po ON pv.option_id = po.id
            WHERE pv.poll_id = $1
            ORDER BY pv.user_id
            "#,
        )
        .bind(poll_id)
        .fetch_all(&self.db)
        .await?;

        // Group by user
        let mut voters: std::collections::HashMap<Uuid, Vec<String>> = std::collections::HashMap::new();
        for (user_id, _option_id, option_text) in votes {
            voters.entry(user_id).or_insert_with(Vec::new).push(option_text);
        }

        // Get usernames
        let mut result = Vec::new();
        for (user_id, options) in voters {
            let username = sqlx::query_scalar::<_, String>(
                "SELECT username FROM users WHERE id = $1",
            )
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

            result.push((user_id, username, options));
        }

        Ok(result)
    }

    /// Deletes a poll
    pub async fn delete_poll(&self, poll_id: Uuid, user_id: Uuid, is_moderator: bool) -> Result<()> {
        let poll = self.get_poll(poll_id).await?;

        if poll.created_by != user_id && !is_moderator {
            return Err(PollError::PermissionDenied);
        }

        let mut tx = self.db.begin().await?;

        // Delete votes
        sqlx::query("DELETE FROM poll_votes WHERE poll_id = $1")
            .bind(poll_id)
            .execute(&mut *tx)
            .await?;

        // Delete options
        sqlx::query("DELETE FROM poll_options WHERE poll_id = $1")
            .bind(poll_id)
            .execute(&mut *tx)
            .await?;

        // Delete poll
        sqlx::query("DELETE FROM polls WHERE id = $1")
            .bind(poll_id)
            .execute(&mut *tx)
            .await?;

        // Update topic if associated
        if let Some(topic_id) = poll.topic_id {
            sqlx::query("UPDATE topics SET has_poll = false, poll_id = NULL WHERE id = $1")
                .bind(topic_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_change_policy() {
        assert_eq!(VoteChangePolicy::Always, VoteChangePolicy::Always);
        assert_ne!(VoteChangePolicy::Always, VoteChangePolicy::Never);
    }

    #[test]
    fn test_create_poll_validation() {
        let request = CreatePollRequest {
            title: "Favorite Feature?".to_string(),
            description: Some("Vote for your favorite feature".to_string()),
            options: vec![
                "Forums".to_string(),
                "Chat".to_string(),
                "Wiki".to_string(),
            ],
            allow_multiple_choice: false,
            max_choices: None,
            allow_vote_change: true,
            show_results_before_vote: true,
            is_anonymous: false,
            expires_at: None,
            topic_id: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_vote_request_validation() {
        let request = VoteRequest {
            poll_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            option_ids: vec![Uuid::new_v4()],
        };

        assert!(request.validate().is_ok());
    }
}
