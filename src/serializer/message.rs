use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::message::Message;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessageQuery {
    pub before: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReadCursorRequest {
    pub last_read_message_id: i64,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReadCursorResponse {
    pub last_read_message_id: i64,
    pub unread_count: i64,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            chat_id: message.chat_id,
            sender_id: message.sender_id,
            content: message.content,
            created_at: message.created_at,
        }
    }
}
