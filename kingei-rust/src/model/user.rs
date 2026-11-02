use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize)]
#[cfg_attr(feature = "sqlite", sqlx(rename_all = "PascalCase"))]
#[cfg_attr(
    feature = "postgres",
    sqlx(type_name = "text", rename_all = "PascalCase")
)]
pub enum UserRole {
    Admin,
    Member,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub password_hash: String,
    pub avatar: Option<i64>,
    pub role: UserRole,
}
