use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::friend::{FriendRequest, Status};
use crate::serializer::user::UserResponse;

#[derive(Debug, Deserialize)]
pub struct SendFriendRequest {
    pub receiver_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendRequestResponse {
    pub id: i64,
    pub sender: UserResponse,
    pub receiver: UserResponse,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FriendResponse {
    pub user: UserResponse,
    pub created_at: DateTime<Utc>,
}

impl FriendRequestResponse {
    pub fn from(request: FriendRequest, sender: UserResponse, receiver: UserResponse) -> Self {
        Self {
            id: request.id,
            sender,
            receiver,
            status: request.status,
            created_at: request.created_at,
            updated_at: request.updated_at,
        }
    }
}
