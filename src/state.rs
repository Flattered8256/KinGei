use crate::event::EventHub;
use crate::repository::chat::ChatRepository;
use crate::repository::file::FileRepository;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::message::MessageRepository;
use crate::repository::user::UserRepository;
use crate::service::chat::ChatService;
use crate::service::file::FileService;
use crate::service::friend::FriendService;
use crate::service::message::MessageService;
use crate::service::user::UserService;
use crate::storage::Storage;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub user_service: UserService,
    pub friend_service: FriendService,
    pub chat_service: ChatService,
    pub message_service: MessageService,
    pub file_service: FileService,
    pub event_hub: EventHub,
}

impl AppState {
    pub fn new(pool: SqlitePool, storage: Storage) -> Self {
        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let friend_repo = Arc::new(FriendRepository::new(pool.clone()));
        let friend_request_repo = Arc::new(FriendRequestRepository::new(pool.clone()));
        let chat_repo = Arc::new(ChatRepository::new(pool.clone()));
        let message_repo = Arc::new(MessageRepository::new(pool.clone()));
        let file_repo = Arc::new(FileRepository::new(pool));
        let event_hub = EventHub::new();
        Self {
            user_service: UserService::new(user_repo.clone()),
            friend_service: FriendService::new(
                friend_repo.clone(),
                friend_request_repo,
                user_repo.clone(),
                event_hub.clone(),
            ),
            chat_service: ChatService::new(
                chat_repo.clone(),
                friend_repo.clone(),
                user_repo,
                message_repo.clone(),
                event_hub.clone(),
            ),
            message_service: MessageService::new(
                message_repo,
                chat_repo,
                friend_repo,
                event_hub.clone(),
            ),
            file_service: FileService::new(file_repo, storage),
            event_hub,
        }
    }
}
