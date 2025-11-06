/// Mock implementations for testing
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Mock email service
#[derive(Clone)]
pub struct MockEmailService {
    sent_emails: Arc<Mutex<Vec<MockEmail>>>,
}

#[derive(Debug, Clone)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self {
            sent_emails: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send(&self, to: String, subject: String, body: String) {
        let email = MockEmail { to, subject, body };
        self.sent_emails.lock().unwrap().push(email);
    }

    pub fn get_sent_emails(&self) -> Vec<MockEmail> {
        self.sent_emails.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.sent_emails.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.sent_emails.lock().unwrap().len()
    }
}

impl Default for MockEmailService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock search service
#[derive(Clone)]
pub struct MockSearchService {
    indexed_documents: Arc<Mutex<Vec<MockDocument>>>,
}

#[derive(Debug, Clone)]
pub struct MockDocument {
    pub id: String,
    pub content: String,
}

impl MockSearchService {
    pub fn new() -> Self {
        Self {
            indexed_documents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn index(&self, id: String, content: String) -> Result<(), String> {
        let doc = MockDocument { id, content };
        self.indexed_documents.lock().unwrap().push(doc);
        Ok(())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MockDocument>, String> {
        let docs = self.indexed_documents.lock().unwrap();
        let results: Vec<MockDocument> = docs
            .iter()
            .filter(|doc| doc.content.contains(query))
            .cloned()
            .collect();
        Ok(results)
    }

    pub fn clear(&self) {
        self.indexed_documents.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.indexed_documents.lock().unwrap().len()
    }
}

impl Default for MockSearchService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock storage service
#[derive(Clone)]
pub struct MockStorageService {
    stored_files: Arc<Mutex<Vec<MockFile>>>,
}

#[derive(Debug, Clone)]
pub struct MockFile {
    pub path: String,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl MockStorageService {
    pub fn new() -> Self {
        Self {
            stored_files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn upload(&self, path: String, content: Vec<u8>, content_type: String) -> Result<String, String> {
        let file = MockFile {
            path: path.clone(),
            content,
            content_type,
        };
        self.stored_files.lock().unwrap().push(file);
        Ok(format!("https://storage.example.com/{}", path))
    }

    pub async fn download(&self, path: &str) -> Result<Vec<u8>, String> {
        let files = self.stored_files.lock().unwrap();
        files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.content.clone())
            .ok_or_else(|| "File not found".to_string())
    }

    pub async fn delete(&self, path: &str) -> Result<(), String> {
        let mut files = self.stored_files.lock().unwrap();
        files.retain(|f| f.path != path);
        Ok(())
    }

    pub fn clear(&self) {
        self.stored_files.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.stored_files.lock().unwrap().len()
    }
}

impl Default for MockStorageService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock notification service
#[derive(Clone)]
pub struct MockNotificationService {
    notifications: Arc<Mutex<Vec<MockNotification>>>,
}

#[derive(Debug, Clone)]
pub struct MockNotification {
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
}

impl MockNotificationService {
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn send(
        &self,
        user_id: Uuid,
        title: String,
        message: String,
        notification_type: String,
    ) -> Result<(), String> {
        let notification = MockNotification {
            user_id,
            title,
            message,
            notification_type,
        };
        self.notifications.lock().unwrap().push(notification);
        Ok(())
    }

    pub fn get_notifications_for_user(&self, user_id: Uuid) -> Vec<MockNotification> {
        self.notifications
            .lock()
            .unwrap()
            .iter()
            .filter(|n| n.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.notifications.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.notifications.lock().unwrap().len()
    }
}

impl Default for MockNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock metrics service
#[derive(Clone)]
pub struct MockMetricsService {
    metrics: Arc<Mutex<Vec<MockMetric>>>,
}

#[derive(Debug, Clone)]
pub struct MockMetric {
    pub name: String,
    pub value: f64,
    pub labels: Vec<(String, String)>,
}

impl MockMetricsService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record(&self, name: String, value: f64, labels: Vec<(String, String)>) {
        let metric = MockMetric {
            name,
            value,
            labels,
        };
        self.metrics.lock().unwrap().push(metric);
    }

    pub fn get_metrics(&self, name: &str) -> Vec<MockMetric> {
        self.metrics
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.name == name)
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.metrics.lock().unwrap().clear();
    }

    pub fn count(&self) -> usize {
        self.metrics.lock().unwrap().len()
    }
}

impl Default for MockMetricsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_email_service() {
        let service = MockEmailService::new();
        service.send(
            "test@example.com".to_string(),
            "Test".to_string(),
            "Body".to_string(),
        );
        assert_eq!(service.count(), 1);
        let emails = service.get_sent_emails();
        assert_eq!(emails[0].to, "test@example.com");
    }

    #[tokio::test]
    async fn test_mock_search_service() {
        let service = MockSearchService::new();
        service
            .index("doc1".to_string(), "test content".to_string())
            .await
            .unwrap();
        assert_eq!(service.count(), 1);

        let results = service.search("test").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_storage_service() {
        let service = MockStorageService::new();
        let url = service
            .upload(
                "test.txt".to_string(),
                b"test content".to_vec(),
                "text/plain".to_string(),
            )
            .await
            .unwrap();
        assert!(url.contains("test.txt"));

        let content = service.download("test.txt").await.unwrap();
        assert_eq!(content, b"test content");
    }

    #[tokio::test]
    async fn test_mock_notification_service() {
        let service = MockNotificationService::new();
        let user_id = Uuid::new_v4();
        service
            .send(
                user_id,
                "Test".to_string(),
                "Message".to_string(),
                "info".to_string(),
            )
            .await
            .unwrap();

        let notifications = service.get_notifications_for_user(user_id);
        assert_eq!(notifications.len(), 1);
    }
}
