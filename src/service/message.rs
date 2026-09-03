use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::message::MessageError;
use crate::model::chat::{Chat, ChatType};
use crate::repository::chat::ChatRepository;
use crate::repository::friend::FriendRepository;
use crate::repository::message::MessageRepository;
use crate::serializer::message::{
    ListMessageQuery, MessageResponse, ReadCursorResponse, SendMessageRequest,
    UpdateReadCursorRequest,
};

const DEFAULT_MESSAGE_LIMIT: i64 = 50;
const MAX_MESSAGE_LIMIT: i64 = 100;

pub struct MessageService {
    message_repo: Arc<MessageRepository>,
    chat_repo: Arc<ChatRepository>,
    friend_repo: Arc<FriendRepository>,
}

impl MessageService {
    pub fn new(
        message_repo: Arc<MessageRepository>,
        chat_repo: Arc<ChatRepository>,
        friend_repo: Arc<FriendRepository>,
    ) -> Self {
        Self {
            message_repo,
            chat_repo,
            friend_repo,
        }
    }

    pub async fn send(
        &self,
        user_id: Uuid,
        chat_id: i64,
        request: SendMessageRequest,
    ) -> Result<MessageResponse, AppError> {
        let content = request.content.trim();
        if content.is_empty() {
            return Err(AppError::Message(MessageError::EmptyContent));
        }

        let chat = self.require_member(user_id, chat_id).await?;
        self.require_direct_friends(user_id, &chat).await?;

        let created = self.message_repo.create(chat.id, user_id, content).await?;
        Ok(MessageResponse::from(created))
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        chat_id: i64,
        query: ListMessageQuery,
    ) -> Result<Vec<MessageResponse>, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        let limit = query
            .limit
            .unwrap_or(DEFAULT_MESSAGE_LIMIT)
            .clamp(1, MAX_MESSAGE_LIMIT);

        let mut messages = self.message_repo.list(chat.id, query.before, limit).await?;
        messages.reverse();
        Ok(messages.into_iter().map(MessageResponse::from).collect())
    }

    pub async fn update_read_cursor(
        &self,
        user_id: Uuid,
        chat_id: i64,
        request: UpdateReadCursorRequest,
    ) -> Result<ReadCursorResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;

        self.message_repo
            .find_in_chat(chat.id, request.last_read_message_id)
            .await?
            .ok_or(AppError::Message(MessageError::MessageNotFound))?;

        let last_read_message_id = self
            .message_repo
            .update_read_cursor(chat.id, user_id, request.last_read_message_id)
            .await?
            .ok_or(AppError::Message(MessageError::NotMember))?;

        let unread_count = self
            .message_repo
            .unread_count(chat.id, user_id, Some(last_read_message_id))
            .await?;

        Ok(ReadCursorResponse {
            last_read_message_id,
            unread_count,
        })
    }

    async fn require_member(&self, user_id: Uuid, chat_id: i64) -> Result<Chat, AppError> {
        let chat = self
            .chat_repo
            .find_by_id(chat_id)
            .await?
            .ok_or(AppError::Message(MessageError::ChatNotFound))?;

        if self
            .chat_repo
            .find_member(chat.id, user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Message(MessageError::NotMember));
        }

        Ok(chat)
    }

    async fn require_direct_friends(&self, user_id: Uuid, chat: &Chat) -> Result<(), AppError> {
        if chat.chat_type != ChatType::Direct {
            return Ok(());
        }

        let members = self.chat_repo.list_members(chat.id).await?;
        let Some(other) = members.iter().find(|member| member.user_id != user_id) else {
            return Err(AppError::Message(MessageError::NotFriends));
        };

        if self
            .friend_repo
            .find(user_id, other.user_id)
            .await?
            .is_none()
        {
            return Err(AppError::Message(MessageError::NotFriends));
        }

        Ok(())
    }
}
