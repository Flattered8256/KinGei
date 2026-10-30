use crate::database::DbPool;
use crate::event::EventHub;
use crate::repository::chat::ChatRepository;
use crate::repository::file::FileRepository;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::instance::InstanceRepository;
use crate::repository::message::MessageRepository;
use crate::repository::user::UserRepository;
use crate::service::admin::AdminService;
use crate::service::chat::ChatService;
use crate::service::file::FileService;
use crate::service::friend::FriendService;
use crate::service::instance::InstanceService;
use crate::service::message::MessageService;
use crate::service::user::UserService;
use crate::storage::Storage;
use std::sync::Arc;

pub struct AppState {
    pub user_service: UserService,
    pub admin_service: AdminService,
    pub friend_service: FriendService,
    pub chat_service: ChatService,
    pub message_service: MessageService,
    pub file_service: FileService,
    pub instance_service: InstanceService,
    pub event_hub: EventHub,
}

impl AppState {
    pub fn new(pool: DbPool, storage: Storage) -> Self {
        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let friend_repo = Arc::new(FriendRepository::new(pool.clone()));
        let friend_request_repo = Arc::new(FriendRequestRepository::new(pool.clone()));
        let chat_repo = Arc::new(ChatRepository::new(pool.clone()));
        let message_repo = Arc::new(MessageRepository::new(pool.clone()));
        let file_repo = Arc::new(FileRepository::new(pool.clone()));
        let instance_repo = Arc::new(InstanceRepository::new(pool));
        let event_hub = EventHub::new();
        Self {
            user_service: UserService::new(user_repo.clone(), file_repo.clone()),
            admin_service: AdminService::new(
                user_repo.clone(),
                chat_repo.clone(),
                file_repo.clone(),
                storage.clone(),
                event_hub.clone(),
            ),
            friend_service: FriendService::new(
                friend_repo.clone(),
                friend_request_repo,
                user_repo.clone(),
                instance_repo.clone(),
                event_hub.clone(),
            ),
            chat_service: ChatService::new(
                chat_repo.clone(),
                friend_repo.clone(),
                user_repo,
                file_repo.clone(),
                message_repo.clone(),
                instance_repo.clone(),
                event_hub.clone(),
            ),
            message_service: MessageService::new(
                message_repo,
                chat_repo,
                friend_repo,
                file_repo.clone(),
                event_hub.clone(),
            ),
            file_service: FileService::new(file_repo, storage),
            instance_service: InstanceService::new(instance_repo),
            event_hub,
        }
    }
}
