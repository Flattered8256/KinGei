use chrono::{DateTime,Utc};
use uuid::Uuid;
use serde::{Serialize,Deserialize};

use crate::model::user::User;

#[derive(Debug,Serialize)]
pub struct UserResponse{
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}
 #[derive(Debug, Serialize)]
pub struct LoginResponse{
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug,Deserialize)]
pub struct CreateUserRequest{
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug,Deserialize)]
pub struct LoginRequest{
    pub email: String,
    pub password: String,
}

#[derive(Debug,Deserialize)]
pub struct UpdateUserRequest{
    pub username: Option<String>,
    pub email: Option<String>,
    
}

#[derive(Debug,Deserialize)]
pub struct ChangePasswordRequest{
    pub old_password: String,
    pub new_password: String,
}

impl From<User> for UserResponse {
    fn from(user:User) -> Self{
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}