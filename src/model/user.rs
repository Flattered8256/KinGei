use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime,Utc};

#[derive(Debug,FromRow)]
pub struct User{
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub password_hash: String
}