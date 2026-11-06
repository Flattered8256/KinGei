use crate::model::chat::{Chat, ChatType};
use crate::serializer::user::UserResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateDirectRequest {
    pub friend_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChatRequest {
    pub name: Option<String>,
    pub avatar: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMemberResponse {
    pub user: UserResponse,
    pub last_read_message_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub member: Vec<ChatMemberResponse>,
    pub created_at: DateTime<Utc>,
    pub unread_count: i64,
    pub avatar: Option<i64>,
}

impl ChatResponse {
    pub fn from(chat: Chat, member: Vec<ChatMemberResponse>, unread_count: i64) -> Self {
        Self {
            id: chat.id,
            chat_type: chat.chat_type,
            name: chat.name,
            owner_id: chat.owner_id,
            member,
            created_at: chat.created_at,
            unread_count,
            avatar: chat.avatar,
        }
    }
}
