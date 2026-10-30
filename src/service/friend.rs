use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::friend::FriendError;
use crate::event::{ClientEvent, EventHub};
use crate::model::friend::{FriendRequest, Status};
use crate::model::user::UserRole;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::instance::InstanceRepository;
use crate::repository::user::UserRepository;
use crate::serializer::friend::{FriendRequestResponse, FriendResponse, SendFriendRequest};
use crate::serializer::user::UserResponse;

pub struct FriendService {
    friend_repo: Arc<FriendRepository>,
    request_repo: Arc<FriendRequestRepository>,
    user_repo: Arc<UserRepository>,
    instance_repo: Arc<InstanceRepository>,
    event_hub: EventHub,
}

impl FriendService {
    pub fn new(
        friend_repo: Arc<FriendRepository>,
        request_repo: Arc<FriendRequestRepository>,
        user_repo: Arc<UserRepository>,
        instance_repo: Arc<InstanceRepository>,
        event_hub: EventHub,
    ) -> Self {
        Self {
            friend_repo,
            request_repo,
            user_repo,
            instance_repo,
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

        let sender = self
            .user_repo
            .find_by_id(sender_id)
            .await?
            .ok_or(AppError::Friend(FriendError::UserNotFound))?;
        if sender.role != UserRole::Admin {
            let instance = self.instance_repo.get().await?;
            if !instance.allow_add_friend {
                return Err(AppError::Friend(FriendError::AddFriendDisabled));
            }
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::database::{DbPool, run_migrations};
    use crate::model::user::User;
    use crate::repository::instance::UpdateInstance;
    use crate::repository::user::UserRepository;

    #[tokio::test]
    async fn member_cannot_send_request_when_disabled() {
        let (service, admin_id, member_id, instance_repo) = setup().await;
        instance_repo
            .update(UpdateInstance {
                allow_add_friend: Some(false),
                allow_create_group: None,
            })
            .await
            .unwrap();

        let err = service
            .send_request(
                member_id,
                SendFriendRequest {
                    receiver_id: admin_id,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Friend(FriendError::AddFriendDisabled)
        ));
    }

    #[tokio::test]
    async fn admin_can_send_request_when_disabled() {
        let (service, admin_id, member_id, instance_repo) = setup().await;
        instance_repo
            .update(UpdateInstance {
                allow_add_friend: Some(false),
                allow_create_group: None,
            })
            .await
            .unwrap();

        let created = service
            .send_request(
                admin_id,
                SendFriendRequest {
                    receiver_id: member_id,
                },
            )
            .await
            .unwrap();
        assert_eq!(created.sender.id, admin_id);
        assert_eq!(created.receiver.id, member_id);
    }

    async fn setup() -> (FriendService, Uuid, Uuid, Arc<InstanceRepository>) {
        let pool = test_pool().await;
        run_migrations(&pool).await;

        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let instance_repo = Arc::new(InstanceRepository::new(pool.clone()));
        let admin = user_repo
            .create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        let member = user_repo
            .create(sample("bob", "bob@example.com"))
            .await
            .unwrap();

        let service = FriendService::new(
            Arc::new(FriendRepository::new(pool.clone())),
            Arc::new(FriendRequestRepository::new(pool)),
            user_repo,
            instance_repo.clone(),
            EventHub::new(),
        );
        (service, admin.id, member.id, instance_repo)
    }

    fn sample(username: &str, email: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            created_at: Utc::now(),
            password_hash: "hash".to_string(),
            avatar: None,
            role: UserRole::Member,
        }
    }

    async fn test_pool() -> DbPool {
        use std::str::FromStr;

        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap()
    }
}
