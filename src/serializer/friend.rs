use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::friend::{FriendRequest, Status};

#[derive(Debug, Deserialize)]
pub struct SendFriendRequest {
    pub receiver_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct FriendRequestResponse {
    pub id: i64,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FriendResponse {
    pub friend_id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<FriendRequest> for FriendRequestResponse {
    fn from(request: FriendRequest) -> Self {
        Self {
            id: request.id,
            sender_id: request.sender_id,
            receiver_id: request.receiver_id,
            status: request.status,
            created_at: request.created_at,
            updated_at: request.updated_at,
        }
    }
}
