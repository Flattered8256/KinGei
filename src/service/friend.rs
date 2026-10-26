use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::friend::FriendError;
use crate::event::{ClientEvent, EventHub};
use crate::model::friend::{FriendRequest, Status};
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::user::UserRepository;
use crate::serializer::friend::{FriendRequestResponse, FriendResponse, SendFriendRequest};
use crate::serializer::user::UserResponse;

pub struct FriendService {
    friend_repo: Arc<FriendRepository>,
    request_repo: Arc<FriendRequestRepository>,
    user_repo: Arc<UserRepository>,
    event_hub: EventHub,
}

impl FriendService {
    pub fn new(
        friend_repo: Arc<FriendRepository>,
        request_repo: Arc<FriendRequestRepository>,
        user_repo: Arc<UserRepository>,
        event_hub: EventHub,
    ) -> Self {
        Self {
            friend_repo,
            request_repo,
            user_repo,
            event_hub,
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
            let response = self.to_response(updated).await?;
            self.emit(
                receiver_id,
                ClientEvent::FriendRequestCreated(response.clone()),
            );
            return Ok(response);
        }

        let created = self.request_repo.create(sender_id, receiver_id).await?;
        let response = self.to_response(created).await?;
        self.emit(
            receiver_id,
            ClientEvent::FriendRequestCreated(response.clone()),
        );
        Ok(response)
    }

    pub async fn list_received(
        &self,
        receiver_id: Uuid,
    ) -> Result<Vec<FriendRequestResponse>, AppError> {
        let requests = self.request_repo.list_received(receiver_id).await?;
        self.to_response_list(requests).await
    }

    pub async fn list_sent(&self, sender_id: Uuid) -> Result<Vec<FriendRequestResponse>, AppError> {
        let requests = self.request_repo.list_sent(sender_id).await?;
        self.to_response_list(requests).await
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

        let response = self.to_response(accepted).await?;
        self.emit(
            request.sender_id,
            ClientEvent::FriendRequestAccepted(response.clone()),
        );
        Ok(response)
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

        let response = self.to_response(rejected).await?;
        self.emit(
            request.sender_id,
            ClientEvent::FriendRequestRejected(response.clone()),
        );
        Ok(response)
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

        let response = self.to_response(deleted).await?;
        self.emit(
            request.receiver_id,
            ClientEvent::FriendRequestCancelled(response.clone()),
        );
        Ok(response)
    }

    pub async fn list_friends(&self, user_id: Uuid) -> Result<Vec<FriendResponse>, AppError> {
        let friends = self.friend_repo.list(user_id).await?;
        let user_ids: Vec<Uuid> = friends.iter().map(|friend| friend.friend_id).collect();
        let users = self.users_by_ids(&user_ids).await?;
        let mut responses = Vec::with_capacity(friends.len());

        for friend in friends {
            let Some(user) = users.get(&friend.friend_id) else {
                continue;
            };
            responses.push(FriendResponse {
                user: user.clone(),
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
        self.emit(friend_id, ClientEvent::friend_deleted(user_id));
        Ok(())
    }

    fn emit(&self, user_id: Uuid, event: ClientEvent) {
        self.event_hub.publish(user_id, Arc::new(event));
    }

    async fn to_response(&self, request: FriendRequest) -> Result<FriendRequestResponse, AppError> {
        let users = self
            .users_by_ids(&[request.sender_id, request.receiver_id])
            .await?;
        let sender = users
            .get(&request.sender_id)
            .cloned()
            .ok_or(AppError::Friend(FriendError::UserNotFound))?;
        let receiver = users
            .get(&request.receiver_id)
            .cloned()
            .ok_or(AppError::Friend(FriendError::UserNotFound))?;
        Ok(FriendRequestResponse::from(request, sender, receiver))
    }

    async fn to_response_list(
        &self,
        requests: Vec<FriendRequest>,
    ) -> Result<Vec<FriendRequestResponse>, AppError> {
        let mut user_ids = Vec::with_capacity(requests.len() * 2);
        for request in &requests {
            user_ids.push(request.sender_id);
            user_ids.push(request.receiver_id);
        }
        let users = self.users_by_ids(&user_ids).await?;

        let mut responses = Vec::with_capacity(requests.len());
        for request in requests {
            let Some(sender) = users.get(&request.sender_id) else {
                continue;
            };
            let Some(receiver) = users.get(&request.receiver_id) else {
                continue;
            };
            responses.push(FriendRequestResponse::from(
                request,
                sender.clone(),
                receiver.clone(),
            ));
        }
        Ok(responses)
    }

    async fn users_by_ids(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, UserResponse>, sqlx::Error> {
        let users = self.user_repo.find_by_ids(ids).await?;
        Ok(users
            .into_iter()
            .map(|user| (user.id, UserResponse::from(user)))
            .collect())
    }
}
