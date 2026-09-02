use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::serializer::user::UserResponse;
use crate::model::chat::{ChatType,Chat};

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
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ChatMemberResponse {
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub member: Vec<ChatMemberResponse>,
    pub created_at: DateTime<Utc>,
}

impl ChatResponse {
    pub fn from(
        chat: Chat,
        member: Vec<ChatMemberResponse>,
    ) -> Self {
        Self {
            id: chat.id,
            chat_type: chat.chat_type,
            name: chat.name,
            owner_id: chat.owner_id,
            member,
            created_at: chat.created_at,
        }
    }
}