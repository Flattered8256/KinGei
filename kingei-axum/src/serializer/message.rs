use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::message::Message;
use crate::serializer::file::FileResponse;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: Option<String>,
    pub file_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListMessageQuery {
    pub before: Option<i64>,
    pub after: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReadCursorRequest {
    pub last_read_message_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct SetTypingRequest {
    pub typing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: Uuid,
    pub content: String,
    pub file: Option<FileResponse>,
    pub created_at: DateTime<Utc>,
    pub deleted: bool,
    pub reply_to: Option<Box<MessageResponse>>,
}

#[derive(Debug, Serialize)]
pub struct ReadCursorResponse {
    pub last_read_message_id: i64,
    pub unread_count: i64,
}

impl MessageResponse {
    pub fn from_message(
        message: Message,
        file: Option<FileResponse>,
        reply_to: Option<Box<MessageResponse>>,
    ) -> Self {
        let deleted = message.deleted_at.is_some();
        Self {
            id: message.id,
            chat_id: message.chat_id,
            sender_id: message.sender_id,
            content: if deleted {
                String::new()
            } else {
                message.content
            },
            file: if deleted { None } else { file },
            created_at: message.created_at,
            deleted,
            reply_to: if deleted { None } else { reply_to },
        }
    }
}
