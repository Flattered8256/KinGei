use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::chat::ChatError;
use crate::event::{ClientEvent, EventHub};
use crate::model::chat::{Chat, ChatType};
use crate::repository::chat::ChatRepository;
use crate::repository::file::FileRepository;
use crate::repository::friend::FriendRepository;
use crate::repository::message::MessageRepository;
use crate::repository::user::UserRepository;
use crate::serializer::chat::{
    ChatMemberResponse, ChatResponse, CreateDirectRequest, CreateGroupRequest, InviteMemberRequest,
    UpdateChatRequest,
};
use crate::serializer::user::UserResponse;
use crate::service::file::require_image;

pub struct ChatService {
    chat_repo: Arc<ChatRepository>,
    friend_repo: Arc<FriendRepository>,
    user_repo: Arc<UserRepository>,
    file_repo: Arc<FileRepository>,
    message_repo: Arc<MessageRepository>,
    event_hub: EventHub,
}

impl ChatService {
    pub fn new(
        chat_repo: Arc<ChatRepository>,
        friend_repo: Arc<FriendRepository>,
        user_repo: Arc<UserRepository>,
        file_repo: Arc<FileRepository>,
        message_repo: Arc<MessageRepository>,
        event_hub: EventHub,
    ) -> Self {
        Self {
            chat_repo,
            friend_repo,
            user_repo,
            file_repo,
            message_repo,
            event_hub,
        }
    }

    pub async fn create_direct(
        &self,
        user_id: Uuid,
        request: CreateDirectRequest,
    ) -> Result<ChatResponse, AppError> {
        let friend_id = request.friend_id;

        if user_id == friend_id {
            return Err(AppError::Chat(ChatError::CannotAddSelf));
        }

        self.user_repo
            .find_by_id(friend_id)
            .await?
            .ok_or(AppError::Chat(ChatError::UserNotFound))?;

        if self.friend_repo.find(user_id, friend_id).await?.is_none() {
            return Err(AppError::Chat(ChatError::NotFriends));
        }

        if let Some(existing) = self.chat_repo.find_direct(user_id, friend_id).await? {
            return self.to_response(user_id, existing).await;
        }

        let created = self.chat_repo.create_direct(user_id, friend_id).await?;
        self.publish_chat_to_members(&created, ClientEvent::ChatCreated)
            .await;
        self.to_response(user_id, created).await
    }

    pub async fn create_group(
        &self,
        user_id: Uuid,
        request: CreateGroupRequest,
    ) -> Result<ChatResponse, AppError> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::Chat(ChatError::EmptyName));
        }

        let created = self.chat_repo.create_group(user_id, name).await?;
        self.publish_chat_to_members(&created, ClientEvent::ChatCreated)
            .await;
        self.to_response(user_id, created).await
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<ChatResponse>, AppError> {
        let chats = self.chat_repo.list_by_user(user_id).await?;
        self.to_responses(user_id, chats).await
    }

    pub async fn get(&self, user_id: Uuid, chat_id: i64) -> Result<ChatResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.to_response(user_id, chat).await
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        chat_id: i64,
        request: UpdateChatRequest,
    ) -> Result<ChatResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.require_group(&chat)?;

        if chat.owner_id != Some(user_id) {
            return Err(AppError::Chat(ChatError::NotOwner));
        }

        if request.name.is_none() && request.avatar.is_none() {
            return Err(AppError::Chat(ChatError::EmptyName));
        }

        let mut updated = chat;

        if let Some(name) = request.name {
            let name = name.trim();
            if name.is_empty() {
                return Err(AppError::Chat(ChatError::EmptyName));
            }
            updated = self
                .chat_repo
                .update_name(updated.id, name)
                .await?
                .ok_or(AppError::Chat(ChatError::ChatNotFound))?;
        }

        if let Some(file_id) = request.avatar {
            require_image(&self.file_repo, user_id, file_id).await?;
            updated = self
                .chat_repo
                .update_avatar(updated.id, file_id)
                .await?
                .ok_or(AppError::Chat(ChatError::ChatNotFound))?;
        }

        self.publish_chat_to_members(&updated, ClientEvent::ChatUpdated)
            .await;
        self.to_response(user_id, updated).await
    }

    pub async fn invite(
        &self,
        user_id: Uuid,
        chat_id: i64,
        request: InviteMemberRequest,
    ) -> Result<ChatResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.require_group(&chat)?;

        if request.user_id == user_id {
            return Err(AppError::Chat(ChatError::CannotAddSelf));
        }

        self.user_repo
            .find_by_id(request.user_id)
            .await?
            .ok_or(AppError::Chat(ChatError::UserNotFound))?;

        if self
            .friend_repo
            .find(user_id, request.user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Chat(ChatError::NotFriends));
        }

        if self
            .chat_repo
            .find_member(chat.id, request.user_id)
            .await?
            .is_some()
        {
            return Err(AppError::Chat(ChatError::AlreadyMember));
        }

        self.chat_repo.add_member(chat.id, request.user_id).await?;
        let chat = self
            .chat_repo
            .find_by_id(chat.id)
            .await?
            .ok_or(AppError::Chat(ChatError::ChatNotFound))?;
        self.publish_chat_to_members(&chat, ClientEvent::ChatMemberAdded)
            .await;
        self.to_response(user_id, chat).await
    }

    pub async fn remove_member(
        &self,
        user_id: Uuid,
        chat_id: i64,
        target_id: Uuid,
    ) -> Result<(), AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.require_group(&chat)?;

        let members = self.chat_repo.list_members(chat.id).await?;
        let member_ids: Vec<Uuid> = members.iter().map(|member| member.user_id).collect();

        if target_id == user_id {
            let is_owner = chat.owner_id == Some(user_id);
            self.chat_repo
                .delete_member(chat.id, user_id)
                .await?
                .ok_or(AppError::Chat(ChatError::NotMember))?;

            if is_owner {
                self.chat_repo.delete_chat(chat.id).await?;
                self.event_hub
                    .publish_many(member_ids, Arc::new(ClientEvent::chat_deleted(chat.id)));
            } else {
                self.event_hub.publish_many(
                    member_ids,
                    Arc::new(ClientEvent::chat_member_removed(chat.id, target_id)),
                );
            }

            return Ok(());
        }

        if chat.owner_id != Some(user_id) {
            return Err(AppError::Chat(ChatError::NotOwner));
        }

        self.chat_repo
            .delete_member(chat.id, target_id)
            .await?
            .ok_or(AppError::Chat(ChatError::NotMember))?;

        self.event_hub.publish_many(
            member_ids,
            Arc::new(ClientEvent::chat_member_removed(chat.id, target_id)),
        );

        Ok(())
    }

    async fn require_member(&self, user_id: Uuid, chat_id: i64) -> Result<Chat, AppError> {
        let chat = self
            .chat_repo
            .find_by_id(chat_id)
            .await?
            .ok_or(AppError::Chat(ChatError::ChatNotFound))?;

        if self
            .chat_repo
            .find_member(chat.id, user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Chat(ChatError::NotMember));
        }

        Ok(chat)
    }

    fn require_group(&self, chat: &Chat) -> Result<(), AppError> {
        if chat.chat_type != ChatType::Group {
            return Err(AppError::Chat(ChatError::CannotModifyDirect));
        }
        Ok(())
    }

    async fn to_response(&self, user_id: Uuid, chat: Chat) -> Result<ChatResponse, AppError> {
        self.to_responses(user_id, vec![chat])
            .await?
            .into_iter()
            .next()
            .ok_or(AppError::Chat(ChatError::ChatNotFound))
    }

    async fn to_responses(
        &self,
        viewer_id: Uuid,
        chats: Vec<Chat>,
    ) -> Result<Vec<ChatResponse>, AppError> {
        if chats.is_empty() {
            return Ok(Vec::new());
        }

        let chat_ids: Vec<i64> = chats.iter().map(|chat| chat.id).collect();
        let members = self.chat_repo.list_members_by_chat_ids(&chat_ids).await?;
        let user_ids: Vec<Uuid> = members.iter().map(|member| member.user_id).collect();
        let users = self.users_by_ids(&user_ids).await?;
        let unread: HashMap<i64, i64> = self
            .message_repo
            .unread_counts_for_user(viewer_id, &chat_ids)
            .await?
            .into_iter()
            .collect();

        let mut members_by_chat: HashMap<i64, Vec<_>> = HashMap::new();
        for member in members {
            members_by_chat
                .entry(member.chat_id)
                .or_default()
                .push(member);
        }

        let mut responses = Vec::with_capacity(chats.len());
        for chat in chats {
            let chat_members = members_by_chat.remove(&chat.id).unwrap_or_default();
            let mut member = Vec::with_capacity(chat_members.len());
            for item in chat_members {
                let Some(user) = users.get(&item.user_id) else {
                    continue;
                };
                member.push(ChatMemberResponse {
                    user: user.clone(),
                    last_read_message_id: item.last_read_message_id,
                });
            }
            let unread_count = unread.get(&chat.id).copied().unwrap_or(0);
            responses.push(ChatResponse::from(chat, member, unread_count));
        }
        Ok(responses)
    }

    async fn publish_chat_to_members(&self, chat: &Chat, wrap: fn(ChatResponse) -> ClientEvent) {
        let members = match self.chat_repo.list_members(chat.id).await {
            Ok(members) => members,
            Err(e) => {
                tracing::error!(
                    ?e,
                    chat_id = chat.id,
                    "failed to list members for chat event"
                );
                return;
            }
        };

        let user_ids: Vec<Uuid> = members.iter().map(|member| member.user_id).collect();
        let users = match self.users_by_ids(&user_ids).await {
            Ok(users) => users,
            Err(e) => {
                tracing::error!(
                    ?e,
                    chat_id = chat.id,
                    "failed to load members for chat event"
                );
                return;
            }
        };
        let unread = match self.message_repo.unread_counts_for_chat(chat.id).await {
            Ok(unread) => unread.into_iter().collect::<HashMap<_, _>>(),
            Err(e) => {
                tracing::error!(
                    ?e,
                    chat_id = chat.id,
                    "failed to load unread counts for chat event"
                );
                return;
            }
        };

        let mut member_responses = Vec::with_capacity(members.len());
        for item in &members {
            let Some(user) = users.get(&item.user_id) else {
                continue;
            };
            member_responses.push(ChatMemberResponse {
                user: user.clone(),
                last_read_message_id: item.last_read_message_id,
            });
        }

        for member in members {
            let unread_count = unread.get(&member.user_id).copied().unwrap_or(0);
            let response = ChatResponse::from(chat.clone(), member_responses.clone(), unread_count);
            self.event_hub
                .publish(member.user_id, Arc::new(wrap(response)));
        }
    }

    async fn users_by_ids(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, UserResponse>, sqlx::Error> {
        let users = self.user_repo.find_by_ids(ids).await?;
        Ok(users
            .into_iter()
            .map(|user| (user.id, UserResponse::from(user)))
            .collect())
    }
}
