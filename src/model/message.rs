use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
