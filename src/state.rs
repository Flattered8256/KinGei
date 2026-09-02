use crate::service::user::UserService;
use crate::service::friend::FriendService;
use crate::service::chat::ChatService;
use crate::repository::user::UserRepository;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::chat::ChatRepository;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub user_service: UserService,
    pub friend_service: FriendService,
    pub chat_service: ChatService,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let friend_repo = Arc::new(FriendRepository::new(pool.clone()));
        let friend_request_repo = Arc::new(FriendRequestRepository::new(pool.clone()));
        let chat_repo = Arc::new(ChatRepository::new(pool));
        Self {
            user_service: UserService::new(user_repo.clone()),
            friend_service: FriendService::new(friend_repo.clone(), friend_request_repo, user_repo.clone()),
            chat_service: ChatService::new(chat_repo, friend_repo, user_repo),
        }
    }
}
