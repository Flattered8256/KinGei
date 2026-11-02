use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::user::UserError;
use crate::event::EventHub;
use crate::model::user::UserRole;
use crate::repository::chat::ChatRepository;
use crate::repository::file::FileRepository;
use crate::repository::user::UserRepository;
use crate::storage::Storage;

pub struct AdminService {
    user_repo: Arc<UserRepository>,
    chat_repo: Arc<ChatRepository>,
    file_repo: Arc<FileRepository>,
    storage: Storage,
    event_hub: EventHub,
}

impl AdminService {
    pub fn new(
        user_repo: Arc<UserRepository>,
        chat_repo: Arc<ChatRepository>,
        file_repo: Arc<FileRepository>,
        storage: Storage,
        event_hub: EventHub,
    ) -> Self {
        Self {
            user_repo,
            chat_repo,
            file_repo,
            storage,
            event_hub,
        }
    }

    pub async fn delete(&self, actor_id: Uuid, target_id: Uuid) -> Result<(), AppError> {
        let target = self
            .user_repo
            .find_by_id(target_id)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        if actor_id == target.id {
            return Err(AppError::User(UserError::CannotDeleteSelf));
        }
        if target.role == UserRole::Admin {
            return Err(AppError::User(UserError::CannotDeleteAdmin));
        }

        let keys = self.file_repo.list_object_keys_by_owner(target_id).await?;
        self.chat_repo.delete_directs_for_user(target_id).await?;
        self.user_repo
            .delete(target_id)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        for key in keys {
            if let Err(e) = self.storage.delete_object(&key).await {
                tracing::error!(?e, key, "delete object failed");
            }
        }
        self.event_hub.disconnect(target_id);
        Ok(())
    }
}
