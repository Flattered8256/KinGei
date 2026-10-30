use std::sync::Arc;

use crate::error::AppError;
use crate::repository::instance::{InstanceRepository, UpdateInstance};
use crate::serializer::instance::{InstanceResponse, UpdateInstanceRequest};

pub struct InstanceService {
    repo: Arc<InstanceRepository>,
}

impl InstanceService {
    pub fn new(repo: Arc<InstanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self) -> Result<InstanceResponse, AppError> {
        let instance = self.repo.get().await?;
        Ok(InstanceResponse::from(instance))
    }

    pub async fn update(
        &self,
        request: UpdateInstanceRequest,
    ) -> Result<InstanceResponse, AppError> {
        if request.allow_add_friend.is_none()
            && request.allow_create_group.is_none()
            && request.invite_only.is_none()
        {
            return self.get().await;
        }

        let updated = self
            .repo
            .update(UpdateInstance {
                allow_add_friend: request.allow_add_friend,
                allow_create_group: request.allow_create_group,
                invite_only: request.invite_only,
            })
            .await?;
        Ok(InstanceResponse::from(updated))
    }
}
