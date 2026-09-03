use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize)]
#[sqlx(rename_all = "PascalCase")]
pub enum ChatType {
    Direct,
    Group,
}

#[derive(Debug, FromRow)]
pub struct Chat {
    pub id: i64,
    pub chat_type: ChatType,
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct ChatMember {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_read_message_id: Option<i64>,
}
