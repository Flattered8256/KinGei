use serde::{Deserialize, Serialize};

use crate::model::instance::Instance;

#[derive(Debug, Serialize)]
pub struct InstanceResponse {
    pub allow_add_friend: bool,
    pub allow_create_group: bool,
    pub initializing: bool,
    pub invite_only: bool,
    pub limit_message_delete: bool,
    pub max_upload_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInstanceRequest {
    pub allow_add_friend: Option<bool>,
    pub allow_create_group: Option<bool>,
    pub invite_only: Option<bool>,
    pub limit_message_delete: Option<bool>,
}

impl InstanceResponse {
    pub fn from_instance(instance: Instance, max_upload_size: u64) -> Self {
        Self {
            allow_add_friend: instance.allow_add_friend,
            allow_create_group: instance.allow_create_group,
            initializing: instance.initializing,
            invite_only: instance.invite_only,
            limit_message_delete: instance.limit_message_delete,
            max_upload_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_carries_flags_and_max_upload_size() {
        let response = InstanceResponse::from_instance(
            Instance {
                id: 1,
                allow_add_friend: false,
                allow_create_group: true,
                initializing: false,
                invite_only: true,
                limit_message_delete: true,
            },
            100 * 1024 * 1024,
        );

        assert!(!response.allow_add_friend);
        assert!(response.allow_create_group);
        assert!(!response.initializing);
        assert!(response.invite_only);
        assert!(response.limit_message_delete);
        assert_eq!(response.max_upload_size, 100 * 1024 * 1024);
    }
}
