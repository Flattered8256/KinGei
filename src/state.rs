use crate::service::user::{
    UserService,
};
use crate::repository::user::UserRepository;
use sqlx::SqlitePool;
use std::sync::Arc;
pub struct AppState {
    pub user_service: UserService,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let user_repo = Arc::new(UserRepository::new(pool));
        Self { user_service: UserService::new(user_repo) }
    }
}