use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime,Utc};

#[derive(Debug,sqlx::Type)]
pub enum Status{
    Accept,
    Pending,
    Reject,
}

#[derive(Debug,FromRow)]
pub struct Friend{
    pub id: i64,
    pub user_id: Uuid,
    pub friend_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug,FromRow)]
pub struct FriendRequest{
    pub id: i64,
    pub sender_id: Uuid, 
    pub receiver_id: Uuid,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}