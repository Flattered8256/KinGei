use serde::{Deserialize, Serialize};

use crate::model::instance::Instance;

#[derive(Debug, Serialize)]
pub struct InstanceResponse {
    pub allow_add_friend: bool,
    pub allow_create_group: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInstanceRequest {
    pub allow_add_friend: Option<bool>,
    pub allow_create_group: Option<bool>,
}

impl From<Instance> for InstanceResponse {
    fn from(instance: Instance) -> Self {
        Self {
            allow_add_friend: instance.allow_add_friend,
            allow_create_group: instance.allow_create_group,
        }
    }
}
