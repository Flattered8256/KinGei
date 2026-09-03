use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::friend::FriendError;
use crate::model::friend::Status;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::user::UserRepository;
use crate::serializer::friend::{FriendRequestResponse, FriendResponse, SendFriendRequest};

pub struct FriendService {
    friend_repo: Arc<FriendRepository>,
    request_repo: Arc<FriendRequestRepository>,
    user_repo: Arc<UserRepository>,
}

impl FriendService {
    pub fn new(
        friend_repo: Arc<FriendRepository>,
        request_repo: Arc<FriendRequestRepository>,
        user_repo: Arc<UserRepository>,
    ) -> Self {
        Self {
            friend_repo,
            request_repo,
            user_repo,
        }
    }

    pub async fn send_request(
        &self,
        sender_id: Uuid,
        request: SendFriendRequest,
    ) -> Result<FriendRequestResponse, AppError> {
        let receiver_id = request.receiver_id;

        if sender_id == receiver_id {
            return Err(AppError::Friend(FriendError::CannotAddSelf));
        }

        self.user_repo
            .find_by_id(receiver_id)
            .await?
            .ok_or(AppError::Friend(FriendError::UserNotFound))?;

        if self
            .friend_repo
            .find(sender_id, receiver_id)
            .await?
            .is_some()
        {
            return Err(AppError::Friend(FriendError::AlreadyFriends));
        }

        if self
            .request_repo
            .find_by_status(sender_id, receiver_id, Status::Pending)
            .await?
            .is_some()
            || self
                .request_repo
                .find_by_status(receiver_id, sender_id, Status::Pending)
                .await?
                .is_some()
        {
            return Err(AppError::Friend(FriendError::RequestAlreadyPending));
        }

        if let Some(existing) = self
            .request_repo
            .find_by_pair(sender_id, receiver_id)
            .await?
        {
            let updated = self
                .request_repo
                .update_status(existing.id, Status::Pending)
                .await?
                .ok_or(AppError::Friend(FriendError::RequestNotFound))?;
            return Ok(FriendRequestResponse::from(updated));
        }

        let created = self.request_repo.create(sender_id, receiver_id).await?;
        Ok(FriendRequestResponse::from(created))
    }

    pub async fn list_received(
        &self,
        receiver_id: Uuid,
    ) -> Result<Vec<FriendRequestResponse>, AppError> {
        let requests = self.request_repo.list_received(receiver_id).await?;
        Ok(requests
            .into_iter()
            .map(FriendRequestResponse::from)
            .collect())
    }

    pub async fn list_sent(&self, sender_id: Uuid) -> Result<Vec<FriendRequestResponse>, AppError> {
        let requests = self.request_repo.list_sent(sender_id).await?;
        Ok(requests
            .into_iter()
            .map(FriendRequestResponse::from)
            .collect())
    }

    pub async fn accept(
        &self,
        user_id: Uuid,
        request_id: i64,
    ) -> Result<FriendRequestResponse, AppError> {
        let request = self
            .request_repo
            .find_by_id(request_id)
            .await?
            .ok_or(AppError::Friend(FriendError::RequestNotFound))?;

        if request.receiver_id != user_id {
            return Err(AppError::Friend(FriendError::NotRequestReceiver));
        }

        if request.status != Status::Pending {
            return Err(AppError::Friend(FriendError::RequestNotPending));
        }

        let (accepted, _) = self
            .request_repo
            .accept_and_create_friend(request.id, request.sender_id, request.receiver_id)
            .await?;

        Ok(FriendRequestResponse::from(accepted))
    }

    pub async fn reject(
        &self,
        user_id: Uuid,
        request_id: i64,
    ) -> Result<FriendRequestResponse, AppError> {
        let request = self
            .request_repo
            .find_by_id(request_id)
            .await?
            .ok_or(AppError::Friend(FriendError::RequestNotFound))?;

        if request.receiver_id != user_id {
            return Err(AppError::Friend(FriendError::NotRequestReceiver));
        }

        if request.status != Status::Pending {
            return Err(AppError::Friend(FriendError::RequestNotPending));
        }

        let rejected = self
            .request_repo
            .update_status(request.id, Status::Reject)
            .await?
            .ok_or(AppError::Friend(FriendError::RequestNotFound))?;

        Ok(FriendRequestResponse::from(rejected))
    }

    pub async fn cancel_request(
        &self,
        user_id: Uuid,
        request_id: i64,
    ) -> Result<FriendRequestResponse, AppError> {
        let request = self
            .request_repo
            .find_by_id(request_id)
            .await?
            .ok_or(AppError::Friend(FriendError::RequestNotFound))?;

        if request.sender_id != user_id {
            return Err(AppError::Friend(FriendError::NotRequestSender));
        }

        if request.status != Status::Pending {
            return Err(AppError::Friend(FriendError::RequestNotPending));
        }

        let deleted = self
            .request_repo
            .delete(request.id)
            .await?
            .ok_or(AppError::Friend(FriendError::RequestNotFound))?;

        Ok(FriendRequestResponse::from(deleted))
    }

    pub async fn list_friends(&self, user_id: Uuid) -> Result<Vec<FriendResponse>, AppError> {
        let friends = self.friend_repo.list(user_id).await?;
        let mut responses = Vec::with_capacity(friends.len());

        for friend in friends {
            let Some(user) = self.user_repo.find_by_id(friend.friend_id).await? else {
                continue;
            };
            responses.push(FriendResponse {
                friend_id: friend.friend_id,
                username: user.username,
                email: user.email,
                created_at: friend.created_at,
            });
        }

        Ok(responses)
    }

    pub async fn delete_friend(&self, user_id: Uuid, friend_id: Uuid) -> Result<(), AppError> {
        if self.friend_repo.find(user_id, friend_id).await?.is_none() {
            return Err(AppError::Friend(FriendError::NotFriends));
        }

        self.friend_repo.delete(user_id, friend_id).await?;
        Ok(())
    }
}
