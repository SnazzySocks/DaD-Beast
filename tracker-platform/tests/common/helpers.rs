/// Test helper functions
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// HTTP test helpers
pub struct HttpTestHelper;

impl HttpTestHelper {
    /// Make a GET request to the router
    pub async fn get(
        router: Router,
        uri: &str,
        auth_token: Option<&str>,
    ) -> (StatusCode, String) {
        let mut request = Request::builder().uri(uri).method("GET");

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let request = request.body(Body::empty()).unwrap();
        let response = router.oneshot(request).await.unwrap();

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Make a POST request to the router
    pub async fn post<T: Serialize>(
        router: Router,
        uri: &str,
        body: &T,
        auth_token: Option<&str>,
    ) -> (StatusCode, String) {
        let json_body = serde_json::to_string(body).unwrap();

        let mut request = Request::builder()
            .uri(uri)
            .method("POST")
            .header("Content-Type", "application/json");

        if let Some(token) = auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let request = request.body(Body::from(json_body)).unwrap();
        let response = router.oneshot(request).await.unwrap();

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Parse JSON response
    pub fn parse_json<T: DeserializeOwned>(body: &str) -> Result<T, serde_json::Error> {
        serde_json::from_str(body)
    }
}

/// Database test helpers
pub struct DbTestHelper;

impl DbTestHelper {
    /// Insert test user into database
    pub async fn insert_user(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Uuid, sqlx::Error> {
        let user_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_active, is_verified)
            VALUES ($1, $2, $3, $4, true, true)
            "#,
            user_id,
            username,
            email,
            password_hash
        )
        .execute(pool)
        .await?;

        Ok(user_id)
    }

    /// Insert test torrent into database
    pub async fn insert_torrent(
        pool: &PgPool,
        info_hash: &str,
        name: &str,
        uploader_id: Uuid,
    ) -> Result<Uuid, sqlx::Error> {
        let torrent_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO torrents (id, info_hash, name, description, size, uploader_id, category_id, is_approved)
            VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            "#,
            torrent_id,
            info_hash,
            name,
            "Test description",
            1000000i64,
            uploader_id,
            1i32
        )
        .execute(pool)
        .await?;

        Ok(torrent_id)
    }

    /// Clean up test user
    pub async fn cleanup_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Clean up test torrent
    pub async fn cleanup_torrent(pool: &PgPool, torrent_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM torrents WHERE id = $1", torrent_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

/// JWT token helper
pub struct JwtHelper;

impl JwtHelper {
    /// Generate a test JWT token
    pub fn generate_token(user_id: Uuid, secret: &str) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde::{Deserialize, Serialize};
        use chrono::{Duration, Utc};

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: i64,
            iat: i64,
        }

        let claims = Claims {
            sub: user_id.to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }
}

/// Redis test helpers
pub struct RedisTestHelper;

impl RedisTestHelper {
    /// Set a test key in Redis
    pub async fn set_key(
        conn: &mut redis::aio::ConnectionManager,
        key: &str,
        value: &str,
    ) -> Result<(), redis::RedisError> {
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(conn)
            .await
    }

    /// Get a test key from Redis
    pub async fn get_key(
        conn: &mut redis::aio::ConnectionManager,
        key: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        redis::cmd("GET")
            .arg(key)
            .query_async(conn)
            .await
    }

    /// Delete a test key from Redis
    pub async fn delete_key(
        conn: &mut redis::aio::ConnectionManager,
        key: &str,
    ) -> Result<(), redis::RedisError> {
        redis::cmd("DEL")
            .arg(key)
            .query_async(conn)
            .await
    }

    /// Clean up test keys by pattern
    pub async fn cleanup_pattern(
        conn: &mut redis::aio::ConnectionManager,
        pattern: &str,
    ) -> Result<(), redis::RedisError> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(conn)
            .await?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async(conn)
                .await?;
        }

        Ok(())
    }
}

/// Assert helpers
#[macro_export]
macro_rules! assert_json_include {
    ($actual:expr, $expected:expr) => {
        let actual: serde_json::Value = serde_json::from_str($actual).unwrap();
        let expected: serde_json::Value = serde_json::from_str($expected).unwrap();
        assert!(
            json_includes(&actual, &expected),
            "JSON does not include expected values.\nActual: {}\nExpected: {}",
            serde_json::to_string_pretty(&actual).unwrap(),
            serde_json::to_string_pretty(&expected).unwrap()
        );
    };
}

fn json_includes(actual: &serde_json::Value, expected: &serde_json::Value) -> bool {
    use serde_json::Value;

    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => {
            e.iter().all(|(k, v)| {
                a.get(k).map_or(false, |av| json_includes(av, v))
            })
        }
        (Value::Array(a), Value::Array(e)) => {
            e.iter().all(|ev| a.iter().any(|av| json_includes(av, ev)))
        }
        (a, e) => a == e,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_includes() {
        let actual = serde_json::json!({
            "name": "test",
            "value": 123,
            "nested": {
                "key": "value"
            }
        });

        let expected = serde_json::json!({
            "name": "test"
        });

        assert!(json_includes(&actual, &expected));
    }
}
