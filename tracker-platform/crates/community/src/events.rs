//! Events Calendar
//!
//! Provides event calendar functionality with RSVP system,
//! reminders, recurring events, and iCalendar export.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;
use validator::Validate;

/// Event-related errors
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Event not found: {0}")]
    NotFound(Uuid),

    #[error("RSVP not found")]
    RsvpNotFound,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Event is full")]
    EventFull,

    #[error("Event has already started")]
    EventStarted,

    #[error("Invalid recurrence pattern: {0}")]
    InvalidRecurrence(String),

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
}

pub type Result<T> = std::result::Result<T, EventError>;

/// Event
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub created_by: Uuid,

    // Timing
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub timezone: String,

    // Recurrence
    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
    pub recurrence_end: Option<DateTime<Utc>>,

    // Settings
    pub max_attendees: Option<i32>,
    pub requires_approval: bool,
    pub is_public: bool,
    pub allow_guests: bool,

    // Category
    pub category: Option<String>,

    // Statistics
    pub attendee_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Event with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWithDetails {
    #[serde(flatten)]
    pub event: Event,
    pub organizer_username: String,
    pub user_rsvp: Option<RsvpStatus>,
    pub is_full: bool,
}

/// Event RSVP
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRsvp {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: RsvpStatus,
    pub guests_count: i32,
    pub comment: Option<String>,
    pub reminder_sent: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RSVP status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rsvp_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Going,
    Maybe,
    NotGoing,
    Pending,
}

/// Event recurrence pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventRecurrence {
    Daily,
    Weekly { days: Vec<chrono::Weekday> },
    Monthly { day_of_month: u32 },
    Yearly { month: u32, day: u32 },
    Custom { rule: String },
}

/// Request to create an event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEventRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(length(max = 5000))]
    pub description: Option<String>,

    #[validate(length(max = 200))]
    pub location: Option<String>,

    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,

    #[validate(length(max = 50))]
    pub timezone: String,

    pub is_recurring: bool,
    pub recurrence_rule: Option<String>,
    pub recurrence_end: Option<DateTime<Utc>>,

    pub max_attendees: Option<i32>,
    pub requires_approval: bool,
    pub is_public: bool,
    pub allow_guests: bool,

    #[validate(length(max = 50))]
    pub category: Option<String>,
}

/// Request to update an event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateEventRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: Option<String>,

    #[validate(length(max = 5000))]
    pub description: Option<String>,

    #[validate(length(max = 200))]
    pub location: Option<String>,

    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub max_attendees: Option<i32>,
    pub is_public: Option<bool>,
}

/// Request to RSVP to an event
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RsvpRequest {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: RsvpStatus,

    #[validate(range(min = 0, max = 10))]
    pub guests_count: i32,

    #[validate(length(max = 500))]
    pub comment: Option<String>,
}

/// Event list parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventListParams {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub category: Option<String>,
    pub page: i32,
    pub per_page: i32,
}

impl Default for EventListParams {
    fn default() -> Self {
        Self {
            start_date: Some(Utc::now()),
            end_date: None,
            category: None,
            page: 1,
            per_page: 25,
        }
    }
}

/// Event service for managing events
pub struct EventService {
    db: PgPool,
}

impl EventService {
    /// Creates a new event service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Creates a new event
    pub async fn create_event(
        &self,
        created_by: Uuid,
        request: CreateEventRequest,
    ) -> Result<Event> {
        request.validate()?;

        // Validate times
        if request.ends_at <= request.starts_at {
            return Err(EventError::Validation(
                validator::ValidationErrors::new(),
            ));
        }

        let event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (
                id, title, description, location, created_by,
                starts_at, ends_at, timezone,
                is_recurring, recurrence_rule, recurrence_end,
                max_attendees, requires_approval, is_public, allow_guests,
                category, attendee_count,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, 0, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&request.title)
        .bind(&request.description)
        .bind(&request.location)
        .bind(created_by)
        .bind(request.starts_at)
        .bind(request.ends_at)
        .bind(&request.timezone)
        .bind(request.is_recurring)
        .bind(&request.recurrence_rule)
        .bind(request.recurrence_end)
        .bind(request.max_attendees)
        .bind(request.requires_approval)
        .bind(request.is_public)
        .bind(request.allow_guests)
        .bind(&request.category)
        .fetch_one(&self.db)
        .await?;

        Ok(event)
    }

    /// Gets an event by ID
    pub async fn get_event(&self, event_id: Uuid) -> Result<Event> {
        let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
            .bind(event_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or(EventError::NotFound(event_id))?;

        Ok(event)
    }

    /// Gets an event with details
    pub async fn get_event_with_details(
        &self,
        event_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<EventWithDetails> {
        let event = self.get_event(event_id).await?;

        let organizer_username = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = $1",
        )
        .bind(event.created_by)
        .fetch_one(&self.db)
        .await?;

        let user_rsvp = if let Some(uid) = user_id {
            sqlx::query_scalar::<_, RsvpStatus>(
                "SELECT status FROM event_rsvps WHERE event_id = $1 AND user_id = $2",
            )
            .bind(event_id)
            .bind(uid)
            .fetch_optional(&self.db)
            .await?
        } else {
            None
        };

        let is_full = if let Some(max) = event.max_attendees {
            event.attendee_count >= max
        } else {
            false
        };

        Ok(EventWithDetails {
            event,
            organizer_username,
            user_rsvp,
            is_full,
        })
    }

    /// Lists events with filters
    pub async fn list_events(
        &self,
        params: EventListParams,
    ) -> Result<(Vec<EventWithDetails>, i64)> {
        let offset = (params.page - 1) * params.per_page;

        let mut query = String::from("SELECT * FROM events WHERE is_public = true");

        if let Some(start) = params.start_date {
            query.push_str(&format!(" AND starts_at >= '{}'", start));
        }

        if let Some(end) = params.end_date {
            query.push_str(&format!(" AND starts_at <= '{}'", end));
        }

        if let Some(cat) = &params.category {
            query.push_str(&format!(" AND category = '{}'", cat));
        }

        query.push_str(&format!(" ORDER BY starts_at LIMIT {} OFFSET {}", params.per_page, offset));

        let events = sqlx::query_as::<_, Event>(&query)
            .fetch_all(&self.db)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM events WHERE is_public = true",
        )
        .fetch_one(&self.db)
        .await?;

        let mut events_with_details = Vec::new();
        for event in events {
            let details = self.get_event_with_details(event.id, None).await?;
            events_with_details.push(details);
        }

        Ok((events_with_details, total))
    }

    /// Updates an event
    pub async fn update_event(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        is_moderator: bool,
        request: UpdateEventRequest,
    ) -> Result<Event> {
        request.validate()?;

        let event = self.get_event(event_id).await?;

        // Check permissions
        if event.created_by != user_id && !is_moderator {
            return Err(EventError::PermissionDenied);
        }

        // Validate times if both are provided
        if let (Some(starts), Some(ends)) = (request.starts_at, request.ends_at) {
            if ends <= starts {
                return Err(EventError::Validation(
                    validator::ValidationErrors::new(),
                ));
            }
        }

        let updated_event = sqlx::query_as::<_, Event>(
            r#"
            UPDATE events
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                location = COALESCE($4, location),
                starts_at = COALESCE($5, starts_at),
                ends_at = COALESCE($6, ends_at),
                max_attendees = COALESCE($7, max_attendees),
                is_public = COALESCE($8, is_public),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(event_id)
        .bind(request.title)
        .bind(request.description.or(Some(event.description)))
        .bind(request.location.or(Some(event.location)))
        .bind(request.starts_at)
        .bind(request.ends_at)
        .bind(request.max_attendees)
        .bind(request.is_public)
        .fetch_one(&self.db)
        .await?;

        Ok(updated_event)
    }

    /// Deletes an event
    pub async fn delete_event(&self, event_id: Uuid, user_id: Uuid, is_moderator: bool) -> Result<()> {
        let event = self.get_event(event_id).await?;

        if event.created_by != user_id && !is_moderator {
            return Err(EventError::PermissionDenied);
        }

        let mut tx = self.db.begin().await?;

        // Delete RSVPs
        sqlx::query("DELETE FROM event_rsvps WHERE event_id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await?;

        // Delete event
        sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// RSVPs to an event
    pub async fn rsvp(&self, request: RsvpRequest) -> Result<EventRsvp> {
        request.validate()?;

        let event = self.get_event(request.event_id).await?;

        // Check if event is full
        if let Some(max) = event.max_attendees {
            if event.attendee_count >= max && request.status == RsvpStatus::Going {
                return Err(EventError::EventFull);
            }
        }

        let mut tx = self.db.begin().await?;

        // Check if user has already RSVP'd
        let existing = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM event_rsvps WHERE event_id = $1 AND user_id = $2",
        )
        .bind(request.event_id)
        .bind(request.user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let rsvp = if let Some(existing_id) = existing {
            // Update existing RSVP
            sqlx::query_as::<_, EventRsvp>(
                r#"
                UPDATE event_rsvps
                SET status = $2,
                    guests_count = $3,
                    comment = $4,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING *
                "#,
            )
            .bind(existing_id)
            .bind(request.status)
            .bind(request.guests_count)
            .bind(&request.comment)
            .fetch_one(&mut *tx)
            .await?
        } else {
            // Create new RSVP
            let status = if event.requires_approval {
                RsvpStatus::Pending
            } else {
                request.status
            };

            sqlx::query_as::<_, EventRsvp>(
                r#"
                INSERT INTO event_rsvps (
                    id, event_id, user_id, status, guests_count, comment,
                    reminder_sent, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, false, NOW(), NOW())
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(request.event_id)
            .bind(request.user_id)
            .bind(status)
            .bind(request.guests_count)
            .bind(&request.comment)
            .fetch_one(&mut *tx)
            .await?
        };

        // Update attendee count
        self.update_attendee_count(&mut tx, request.event_id).await?;

        tx.commit().await?;

        Ok(rsvp)
    }

    /// Removes RSVP
    pub async fn remove_rsvp(&self, event_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.db.begin().await?;

        sqlx::query("DELETE FROM event_rsvps WHERE event_id = $1 AND user_id = $2")
            .bind(event_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        self.update_attendee_count(&mut *tx, event_id).await?;

        tx.commit().await?;

        Ok(())
    }

    /// Gets RSVPs for an event
    pub async fn get_rsvps(&self, event_id: Uuid, status: Option<RsvpStatus>) -> Result<Vec<(EventRsvp, String)>> {
        let rsvps = if let Some(status) = status {
            sqlx::query_as::<_, (EventRsvp, String)>(
                r#"
                SELECT er.*, u.username
                FROM event_rsvps er
                JOIN users u ON er.user_id = u.id
                WHERE er.event_id = $1 AND er.status = $2
                ORDER BY er.created_at
                "#,
            )
            .bind(event_id)
            .bind(status)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, (EventRsvp, String)>(
                r#"
                SELECT er.*, u.username
                FROM event_rsvps er
                JOIN users u ON er.user_id = u.id
                WHERE er.event_id = $1
                ORDER BY er.created_at
                "#,
            )
            .bind(event_id)
            .fetch_all(&self.db)
            .await?
        };

        Ok(rsvps)
    }

    /// Approves an RSVP (for events requiring approval)
    pub async fn approve_rsvp(&self, event_id: Uuid, user_id: Uuid) -> Result<()> {
        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
            UPDATE event_rsvps
            SET status = 'going', updated_at = NOW()
            WHERE event_id = $1 AND user_id = $2 AND status = 'pending'
            "#,
        )
        .bind(event_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        self.update_attendee_count(&mut *tx, event_id).await?;

        tx.commit().await?;

        Ok(())
    }

    /// Exports event to iCalendar format
    pub fn export_ical(&self, event: &Event) -> Result<String> {
        use icalendar::{Calendar, Component, Event as ICalEvent};

        let mut calendar = Calendar::new();

        let mut ical_event = ICalEvent::new();
        ical_event
            .summary(&event.title)
            .starts(event.starts_at)
            .ends(event.ends_at);

        if let Some(desc) = &event.description {
            ical_event.description(desc);
        }

        if let Some(loc) = &event.location {
            ical_event.location(loc);
        }

        calendar.push(ical_event);

        Ok(calendar.to_string())
    }

    /// Gets upcoming events for a user
    pub async fn get_user_events(&self, user_id: Uuid) -> Result<Vec<EventWithDetails>> {
        let events = sqlx::query_as::<_, Event>(
            r#"
            SELECT DISTINCT e.* FROM events e
            JOIN event_rsvps er ON e.id = er.event_id
            WHERE er.user_id = $1
              AND er.status IN ('going', 'maybe')
              AND e.starts_at > NOW()
            ORDER BY e.starts_at
            LIMIT 50
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let mut events_with_details = Vec::new();
        for event in events {
            let details = self.get_event_with_details(event.id, Some(user_id)).await?;
            events_with_details.push(details);
        }

        Ok(events_with_details)
    }

    /// Updates attendee count for an event
    async fn update_attendee_count(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, event_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE events
            SET attendee_count = (
                SELECT COUNT(*) FROM event_rsvps
                WHERE event_id = $1 AND status = 'going'
            )
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsvp_status() {
        assert_eq!(RsvpStatus::Going, RsvpStatus::Going);
        assert_ne!(RsvpStatus::Going, RsvpStatus::Maybe);
    }

    #[test]
    fn test_create_event_validation() {
        let request = CreateEventRequest {
            title: "Community Meetup".to_string(),
            description: Some("Monthly community gathering".to_string()),
            location: Some("Discord".to_string()),
            starts_at: Utc::now() + chrono::Duration::days(7),
            ends_at: Utc::now() + chrono::Duration::days(7) + chrono::Duration::hours(2),
            timezone: "UTC".to_string(),
            is_recurring: false,
            recurrence_rule: None,
            recurrence_end: None,
            max_attendees: Some(50),
            requires_approval: false,
            is_public: true,
            allow_guests: true,
            category: Some("Community".to_string()),
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_event_list_params_default() {
        let params = EventListParams::default();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 25);
    }
}
