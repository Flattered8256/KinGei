use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ReadReceiptEvent {
    pub chat_id: i64,
    pub user_id: Uuid,
    pub last_read_message_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMemberRemovedEvent {
    pub chat_id: i64,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatDeletedEvent {
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendDeletedEvent {
    pub friend_id: Uuid,
}
