use std::sync::Arc;
use uuid::Uuid;

use crate::error::chat::ChatError;
use crate::error::AppError;
use crate::model::chat::{Chat, ChatType};
use crate::repository::chat::ChatRepository;
use crate::repository::friend::FriendRepository;
use crate::repository::user::UserRepository;
use crate::serializer::chat::{
    ChatMemberResponse, ChatResponse, CreateDirectRequest, CreateGroupRequest, InviteMemberRequest,
    UpdateChatRequest,
};

pub struct ChatService {
    chat_repo: Arc<ChatRepository>,
    friend_repo: Arc<FriendRepository>,
    user_repo: Arc<UserRepository>,
}

impl ChatService {
    pub fn new(
        chat_repo: Arc<ChatRepository>,
        friend_repo: Arc<FriendRepository>,
        user_repo: Arc<UserRepository>,
    ) -> Self {
        Self {
            chat_repo,
            friend_repo,
            user_repo,
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
            return self.to_response(existing).await;
        }

        let created = self.chat_repo.create_direct(user_id, friend_id).await?;
        self.to_response(created).await
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


        let created = self
            .chat_repo
            .create_group(user_id, name)
            .await?;
        self.to_response(created).await
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<ChatResponse>, AppError> {
        let chats = self.chat_repo.list_by_user(user_id).await?;
        let mut responses = Vec::with_capacity(chats.len());
        for chat in chats {
            responses.push(self.to_response(chat).await?);
        }
        Ok(responses)
    }

    pub async fn get(&self, user_id: Uuid, chat_id: i64) -> Result<ChatResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.to_response(chat).await
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

        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::Chat(ChatError::EmptyName));
        }

        let updated = self
            .chat_repo
            .update_name(chat.id, name)
            .await?
            .ok_or(AppError::Chat(ChatError::ChatNotFound))?;

        self.to_response(updated).await
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

        if self.friend_repo.find(user_id, request.user_id).await?.is_none() {
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
        self.to_response(chat).await
    }

    pub async fn remove_member(
        &self,
        user_id: Uuid,
        chat_id: i64,
        target_id: Uuid,
    ) -> Result<(), AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.require_group(&chat)?;

        if target_id == user_id {
            let is_owner = chat.owner_id == Some(user_id);
            self.chat_repo
                .delete_member(chat.id, user_id)
                .await?
                .ok_or(AppError::Chat(ChatError::NotMember))?;

            if is_owner {
                    self.chat_repo.delete_chat(chat.id).await?;                
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

        Ok(())
    }

    async fn require_member(&self, user_id: Uuid, chat_id: i64) -> Result<Chat, AppError> {
        let chat = self
            .chat_repo
            .find_by_id(chat_id)
            .await?
            .ok_or(AppError::Chat(ChatError::ChatNotFound))?;

        if self.chat_repo.find_member(chat.id, user_id).await?.is_none() {
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

    async fn to_response(&self, chat: Chat) -> Result<ChatResponse, AppError> {
        let members = self.chat_repo.list_members(chat.id).await?;
        let mut member = Vec::with_capacity(members.len());

        for item in members {
            let Some(user) = self.user_repo.find_by_id(item.user_id).await? else {
                continue;
            };
            member.push(ChatMemberResponse {
                user: user.into(),
            });
        }

        Ok(ChatResponse::from(chat, member))
    }
}
