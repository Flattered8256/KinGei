use serde::Serialize;

use crate::model::invite::Invite;

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub code: String,
}

impl From<Invite> for InviteResponse {
    fn from(invite: Invite) -> Self {
        Self { code: invite.code }
    }
}
