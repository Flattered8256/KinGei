use std::sync::Arc;

use crate::error::AppError;
use crate::repository::instance::{InstanceRepository, UpdateInstance};
use crate::serializer::instance::{InstanceResponse, UpdateInstanceRequest};

pub struct InstanceService {
    repo: Arc<InstanceRepository>,
    max_upload_size: u64,
}

impl InstanceService {
    pub fn new(repo: Arc<InstanceRepository>, max_upload_size: u64) -> Self {
        Self {
            repo,
            max_upload_size,
        }
    }

    pub async fn get(&self) -> Result<InstanceResponse, AppError> {
        let instance = self.repo.get().await?;
        Ok(InstanceResponse::from_instance(
            instance,
            self.max_upload_size,
        ))
    }

    pub async fn update(
        &self,
        request: UpdateInstanceRequest,
    ) -> Result<InstanceResponse, AppError> {
        if request.allow_add_friend.is_none()
            && request.allow_create_group.is_none()
            && request.invite_only.is_none()
            && request.limit_message_delete.is_none()
        {
            return self.get().await;
        }

        let updated = self
            .repo
            .update(UpdateInstance {
                allow_add_friend: request.allow_add_friend,
                allow_create_group: request.allow_create_group,
                invite_only: request.invite_only,
                limit_message_delete: request.limit_message_delete,
            })
            .await?;
        Ok(InstanceResponse::from_instance(
            updated,
            self.max_upload_size,
        ))
    }
}
