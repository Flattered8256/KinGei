use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::message::MessageError;
use crate::event::{ClientEvent, EventHub};
use crate::model::chat::{Chat, ChatType};
use crate::model::file::File;
use crate::model::user::UserRole;
use crate::repository::chat::ChatRepository;
use crate::repository::file::FileRepository;
use crate::repository::friend::FriendRepository;
use crate::repository::instance::InstanceRepository;
use crate::repository::message::MessageRepository;
use crate::repository::user::UserRepository;
use crate::serializer::file::FileResponse;
use crate::serializer::message::{
    ListMessageQuery, MessageResponse, ReadCursorResponse, SendMessageRequest,
    UpdateReadCursorRequest,
};
use crate::service::file::require_ready_file;

const DEFAULT_MESSAGE_LIMIT: i64 = 50;
const MAX_MESSAGE_LIMIT: i64 = 100;
const MESSAGE_DELETE_WINDOW_SECS: i64 = 120;

pub struct MessageService {
    message_repo: Arc<MessageRepository>,
    chat_repo: Arc<ChatRepository>,
    friend_repo: Arc<FriendRepository>,
    file_repo: Arc<FileRepository>,
    user_repo: Arc<UserRepository>,
    instance_repo: Arc<InstanceRepository>,
    event_hub: EventHub,
}

impl MessageService {
    pub fn new(
        message_repo: Arc<MessageRepository>,
        chat_repo: Arc<ChatRepository>,
        friend_repo: Arc<FriendRepository>,
        file_repo: Arc<FileRepository>,
        user_repo: Arc<UserRepository>,
        instance_repo: Arc<InstanceRepository>,
        event_hub: EventHub,
    ) -> Self {
        Self {
            message_repo,
            chat_repo,
            friend_repo,
            file_repo,
            user_repo,
            instance_repo,
            event_hub,
        }
    }

    pub async fn send(
        &self,
        user_id: Uuid,
        chat_id: i64,
        request: SendMessageRequest,
        reply_to_id: Option<i64>,
    ) -> Result<MessageResponse, AppError> {
        let content = request.content.as_deref().unwrap_or_default().trim();
        if content.is_empty() && request.file_id.is_none() {
            return Err(AppError::Message(MessageError::EmptyContent));
        }

        let chat = self.require_member(user_id, chat_id).await?;
        self.require_direct_friends(user_id, &chat).await?;

        let parent = if let Some(id) = reply_to_id {
            let parent = self
                .message_repo
                .find_in_chat(chat.id, id)
                .await?
                .ok_or(AppError::Message(MessageError::MessageNotFound))?;
            if parent.deleted_at.is_some() {
                return Err(AppError::Message(MessageError::ReplyTargetDeleted));
            }
            Some(parent)
        } else {
            None
        };

        let file = match request.file_id {
            Some(file_id) => Some(require_ready_file(&self.file_repo, user_id, file_id).await?),
            None => None,
        };

        let created = self
            .message_repo
            .create(chat.id, user_id, content, request.file_id, reply_to_id)
            .await?;

        let mut files = HashMap::new();
        if let Some(file) = file {
            files.insert(file.id, self.file_response(file).await?);
        }
        if let Some(parent_file_id) = parent.as_ref().and_then(|p| p.file_id)
            && !files.contains_key(&parent_file_id)
        {
            files.extend(self.load_files(&[parent_file_id]).await?);
        }

        let reply_to = parent.map(|p| {
            let file = p.file_id.and_then(|id| files.get(&id).cloned());
            Box::new(MessageResponse::from_message(p, file, None))
        });
        let file = created.file_id.and_then(|id| files.get(&id).cloned());
        let response = MessageResponse::from_message(created, file, reply_to);
        self.publish_message_created(chat.id, response.clone())
            .await;
        Ok(response)
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

        let after = query.after;
        let mut messages = self
            .message_repo
            .list(chat.id, query.before, after, limit)
            .await?;
        if after.is_none() {
            messages.reverse();
        }

        let mut reply_ids = Vec::new();
        let mut seen_replies = HashSet::new();
        for message in &messages {
            if let Some(id) = message.reply_to_id
                && seen_replies.insert(id)
            {
                reply_ids.push(id);
            }
        }
        let parents = self.message_repo.find_by_ids(chat.id, &reply_ids).await?;

        let mut file_ids = Vec::new();
        let mut seen = HashSet::new();
        for message in messages.iter().chain(parents.iter()) {
            if let Some(file_id) = message.file_id
                && seen.insert(file_id)
            {
                file_ids.push(file_id);
            }
        }

        let files = self.load_files(&file_ids).await?;
        let replies: HashMap<i64, MessageResponse> = parents
            .into_iter()
            .map(|parent| {
                let file = parent.file_id.and_then(|id| files.get(&id).cloned());
                (parent.id, MessageResponse::from_message(parent, file, None))
            })
            .collect();

        let mut responses = Vec::with_capacity(messages.len());
        for message in messages {
            let file = message.file_id.and_then(|id| files.get(&id).cloned());
            let reply_to = message
                .reply_to_id
                .and_then(|id| replies.get(&id).cloned())
                .map(Box::new);
            responses.push(MessageResponse::from_message(message, file, reply_to));
        }
        Ok(responses)
    }

    pub async fn delete(
        &self,
        user_id: Uuid,
        chat_id: i64,
        message_id: i64,
    ) -> Result<MessageResponse, AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        let message = self
            .message_repo
            .find_in_chat(chat.id, message_id)
            .await?
            .ok_or(AppError::Message(MessageError::MessageNotFound))?;

        if message.deleted_at.is_some() {
            return Err(AppError::Message(MessageError::MessageAlreadyDeleted));
        }

        let actor = self.user_repo.find_by_id(user_id).await?;
        let privileged = chat.owner_id == Some(user_id)
            || actor
                .as_ref()
                .is_some_and(|user| user.role == UserRole::Admin);

        if message.sender_id != user_id && !privileged {
            return Err(AppError::Message(MessageError::CannotDeleteMessage));
        }

        if !privileged {
            let instance = self.instance_repo.get().await?;
            if instance.limit_message_delete {
                let age = Utc::now().signed_duration_since(message.created_at);
                if age.num_seconds() > MESSAGE_DELETE_WINDOW_SECS {
                    return Err(AppError::Message(MessageError::CannotDeleteOld));
                }
            }
        }

        let deleted = self
            .message_repo
            .soft_delete(chat.id, message.id)
            .await?
            .ok_or(AppError::Message(MessageError::MessageAlreadyDeleted))?;

        let response = MessageResponse::from_message(deleted, None, None);
        self.publish_message_deleted(chat.id, response.clone())
            .await;
        Ok(response)
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

        self.publish_message_read(chat.id, user_id, last_read_message_id)
            .await;

        Ok(ReadCursorResponse {
            last_read_message_id,
            unread_count,
        })
    }

    async fn load_files(&self, file_ids: &[i64]) -> Result<HashMap<i64, FileResponse>, AppError> {
        if file_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let files = self.file_repo.find_by_ids(file_ids).await?;
        let mut media: HashMap<i64, _> = self
            .file_repo
            .find_media_by_file_ids(file_ids)
            .await?
            .into_iter()
            .map(|row| (row.file_id, row))
            .collect();
        Ok(files
            .into_iter()
            .map(|file| {
                let media = media.remove(&file.id);
                (file.id, FileResponse::from(file, media))
            })
            .collect())
    }

    async fn file_response(&self, file: File) -> Result<FileResponse, AppError> {
        let media = self.file_repo.find_media(file.id).await?;
        Ok(FileResponse::from(file, media))
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

    async fn publish_message_created(&self, chat_id: i64, response: MessageResponse) {
        match self.chat_repo.list_members(chat_id).await {
            Ok(members) => {
                self.event_hub.publish_many(
                    members.into_iter().map(|member| member.user_id),
                    Arc::new(ClientEvent::MessageCreated(response)),
                );
            }
            Err(e) => tracing::error!(?e, chat_id, "failed to list members for message event"),
        }
    }

    async fn publish_message_deleted(&self, chat_id: i64, response: MessageResponse) {
        match self.chat_repo.list_members(chat_id).await {
            Ok(members) => {
                self.event_hub.publish_many(
                    members.into_iter().map(|member| member.user_id),
                    Arc::new(ClientEvent::MessageDeleted(response)),
                );
            }
            Err(e) => tracing::error!(?e, chat_id, "failed to list members for message event"),
        }
    }

    async fn publish_message_read(&self, chat_id: i64, user_id: Uuid, last_read_message_id: i64) {
        match self.chat_repo.list_members(chat_id).await {
            Ok(members) => {
                let event = Arc::new(ClientEvent::message_read(
                    chat_id,
                    user_id,
                    last_read_message_id,
                ));
                self.event_hub.publish_many(
                    members
                        .into_iter()
                        .filter(|member| member.user_id != user_id)
                        .map(|member| member.user_id),
                    event,
                );
            }
            Err(e) => tracing::error!(?e, chat_id, "failed to list members for read event"),
        }
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::database::{DbPool, run_migrations};
    use crate::model::user::User;
    use crate::repository::instance::UpdateInstance;
    use crate::repository::user::UserRepository;

    #[tokio::test]
    async fn sender_soft_deletes_own_message() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "hello").await;

        let deleted = fx
            .service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap();
        assert!(deleted.deleted);
        assert!(deleted.content.is_empty());
        assert!(deleted.file.is_none());

        let listed = fx
            .service
            .list(
                fx.owner_id,
                fx.chat_id,
                ListMessageQuery {
                    before: None,
                    after: None,
                    limit: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].deleted);
    }

    #[tokio::test]
    async fn member_cannot_delete_others() {
        let fx = setup().await;
        let created = send(&fx, fx.owner_id, "owner").await;

        let err = fx
            .service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::CannotDeleteMessage)
        ));
    }

    #[tokio::test]
    async fn owner_can_delete_others() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "member").await;

        let deleted = fx
            .service
            .delete(fx.owner_id, fx.chat_id, created.id)
            .await
            .unwrap();
        assert!(deleted.deleted);
    }

    #[tokio::test]
    async fn admin_can_delete_others() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "member").await;

        let deleted = fx
            .service
            .delete(fx.admin_id, fx.chat_id, created.id)
            .await
            .unwrap();
        assert!(deleted.deleted);
    }

    #[tokio::test]
    async fn window_blocks_member_from_deleting_old() {
        let fx = setup().await;
        enable_window(&fx).await;
        let created = send(&fx, fx.member_id, "old").await;
        backdate(&fx.pool, created.id).await;

        let err = fx
            .service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::CannotDeleteOld)
        ));
    }

    #[tokio::test]
    async fn window_allows_owner_and_admin_to_delete_old() {
        let fx = setup().await;
        enable_window(&fx).await;
        let member_msg = send(&fx, fx.member_id, "old-member").await;
        backdate(&fx.pool, member_msg.id).await;
        fx.service
            .delete(fx.owner_id, fx.chat_id, member_msg.id)
            .await
            .unwrap();

        let owner_msg = send(&fx, fx.owner_id, "old-owner").await;
        backdate(&fx.pool, owner_msg.id).await;
        fx.service
            .delete(fx.admin_id, fx.chat_id, owner_msg.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn window_off_allows_member_to_delete_old() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "old").await;
        backdate(&fx.pool, created.id).await;

        let deleted = fx
            .service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap();
        assert!(deleted.deleted);
    }

    #[tokio::test]
    async fn already_deleted_conflicts() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "hello").await;
        fx.service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap();

        let err = fx
            .service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::MessageAlreadyDeleted)
        ));
    }

    #[tokio::test]
    async fn unread_ignores_deleted() {
        let fx = setup().await;
        let created = send(&fx, fx.member_id, "hello").await;
        let before = fx
            .message_repo
            .unread_count(fx.chat_id, fx.owner_id, None)
            .await
            .unwrap();
        assert_eq!(before, 1);

        fx.service
            .delete(fx.member_id, fx.chat_id, created.id)
            .await
            .unwrap();
        let after = fx
            .message_repo
            .unread_count(fx.chat_id, fx.owner_id, None)
            .await
            .unwrap();
        assert_eq!(after, 0);
    }

    #[tokio::test]
    async fn reply_embeds_parent_in_response_and_list() {
        let fx = setup().await;
        let parent = send(&fx, fx.member_id, "hello").await;

        let created = reply(&fx, fx.owner_id, parent.id, "ok").await.unwrap();
        let nested = created.reply_to.expect("reply_to");
        assert_eq!(nested.id, parent.id);
        assert_eq!(nested.content, "hello");
        assert!(nested.reply_to.is_none());

        let listed = list_all(&fx, fx.owner_id).await;
        assert_eq!(listed.len(), 2);
        let listed_reply = listed.iter().find(|m| m.id == created.id).unwrap();
        let nested = listed_reply.reply_to.as_ref().expect("reply_to");
        assert_eq!(nested.id, parent.id);
        assert_eq!(nested.content, "hello");
    }

    #[tokio::test]
    async fn list_embeds_parent_outside_page() {
        let fx = setup().await;
        let parent = send(&fx, fx.member_id, "hello").await;
        send(&fx, fx.owner_id, "filler").await;
        let created = reply(&fx, fx.owner_id, parent.id, "ok").await.unwrap();

        let listed = fx
            .service
            .list(
                fx.owner_id,
                fx.chat_id,
                ListMessageQuery {
                    before: None,
                    after: None,
                    limit: Some(1),
                },
            )
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);
        let nested = listed[0].reply_to.as_ref().expect("reply_to");
        assert_eq!(nested.id, parent.id);
        assert_eq!(nested.content, "hello");
    }

    #[tokio::test]
    async fn reply_shows_deleted_parent() {
        let fx = setup().await;
        let parent = send(&fx, fx.member_id, "hello").await;
        let created = reply(&fx, fx.owner_id, parent.id, "ok").await.unwrap();

        fx.service
            .delete(fx.member_id, fx.chat_id, parent.id)
            .await
            .unwrap();

        let listed = list_all(&fx, fx.owner_id).await;
        let listed_reply = listed.iter().find(|m| m.id == created.id).unwrap();
        let nested = listed_reply.reply_to.as_ref().expect("reply_to");
        assert_eq!(nested.id, parent.id);
        assert!(nested.deleted);
        assert!(nested.content.is_empty());
        assert!(nested.file.is_none());
    }

    #[tokio::test]
    async fn cannot_reply_to_deleted() {
        let fx = setup().await;
        let parent = send(&fx, fx.member_id, "hello").await;
        fx.service
            .delete(fx.member_id, fx.chat_id, parent.id)
            .await
            .unwrap();

        let err = reply(&fx, fx.owner_id, parent.id, "ok").await.unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::ReplyTargetDeleted)
        ));
    }

    #[tokio::test]
    async fn cannot_reply_to_missing_or_other_chat() {
        let fx = setup().await;
        let err = reply(&fx, fx.owner_id, 999_999, "ok").await.unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::MessageNotFound)
        ));

        let other = fx
            .chat_repo
            .create_group(fx.owner_id, "other")
            .await
            .unwrap();
        fx.chat_repo
            .add_member(other.id, fx.member_id)
            .await
            .unwrap();
        let other_msg = fx
            .service
            .send(
                fx.member_id,
                other.id,
                SendMessageRequest {
                    content: Some("secret".to_string()),
                    file_id: None,
                },
                None,
            )
            .await
            .unwrap();

        let err = reply(&fx, fx.owner_id, other_msg.id, "ok")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::Message(MessageError::MessageNotFound)
        ));
    }

    #[tokio::test]
    async fn reply_to_reply_is_one_level() {
        let fx = setup().await;
        let first = send(&fx, fx.member_id, "one").await;
        let second = reply(&fx, fx.owner_id, first.id, "two").await.unwrap();
        let third = reply(&fx, fx.member_id, second.id, "three").await.unwrap();

        let nested = third.reply_to.expect("reply_to");
        assert_eq!(nested.id, second.id);
        assert_eq!(nested.content, "two");
        assert!(nested.reply_to.is_none());
    }

    struct Fixture {
        service: MessageService,
        pool: DbPool,
        instance_repo: Arc<InstanceRepository>,
        message_repo: Arc<MessageRepository>,
        chat_repo: Arc<ChatRepository>,
        admin_id: Uuid,
        owner_id: Uuid,
        member_id: Uuid,
        chat_id: i64,
    }

    async fn send(fx: &Fixture, sender_id: Uuid, content: &str) -> MessageResponse {
        fx.service
            .send(
                sender_id,
                fx.chat_id,
                SendMessageRequest {
                    content: Some(content.to_string()),
                    file_id: None,
                },
                None,
            )
            .await
            .unwrap()
    }

    async fn reply(
        fx: &Fixture,
        sender_id: Uuid,
        message_id: i64,
        content: &str,
    ) -> Result<MessageResponse, AppError> {
        fx.service
            .send(
                sender_id,
                fx.chat_id,
                SendMessageRequest {
                    content: Some(content.to_string()),
                    file_id: None,
                },
                Some(message_id),
            )
            .await
    }

    async fn list_all(fx: &Fixture, user_id: Uuid) -> Vec<MessageResponse> {
        fx.service
            .list(
                user_id,
                fx.chat_id,
                ListMessageQuery {
                    before: None,
                    after: None,
                    limit: None,
                },
            )
            .await
            .unwrap()
    }

    async fn enable_window(fx: &Fixture) {
        fx.instance_repo
            .update(UpdateInstance {
                allow_add_friend: None,
                allow_create_group: None,
                invite_only: None,
                limit_message_delete: Some(true),
            })
            .await
            .unwrap();
    }

    async fn backdate(pool: &DbPool, message_id: i64) {
        let created_at = Utc::now() - Duration::seconds(MESSAGE_DELETE_WINDOW_SECS + 1);
        sqlx::query("UPDATE message SET created_at = $1 WHERE id = $2")
            .bind(created_at)
            .bind(message_id)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn setup() -> Fixture {
        let pool = test_pool().await;
        run_migrations(&pool).await;

        let user_repo = Arc::new(UserRepository::new(pool.clone()));
        let chat_repo = Arc::new(ChatRepository::new(pool.clone()));
        let message_repo = Arc::new(MessageRepository::new(pool.clone()));
        let instance_repo = Arc::new(InstanceRepository::new(pool.clone()));

        let admin = user_repo
            .create(sample("alice", "alice@example.com"))
            .await
            .unwrap();
        let owner = user_repo
            .create(sample("bob", "bob@example.com"))
            .await
            .unwrap();
        let member = user_repo
            .create(sample("carol", "carol@example.com"))
            .await
            .unwrap();

        let chat = chat_repo.create_group(owner.id, "team").await.unwrap();
        chat_repo.add_member(chat.id, admin.id).await.unwrap();
        chat_repo.add_member(chat.id, member.id).await.unwrap();

        let service = MessageService::new(
            message_repo.clone(),
            chat_repo.clone(),
            Arc::new(FriendRepository::new(pool.clone())),
            Arc::new(FileRepository::new(pool.clone())),
            user_repo,
            instance_repo.clone(),
            EventHub::new(),
        );

        Fixture {
            service,
            pool,
            instance_repo,
            message_repo,
            chat_repo,
            admin_id: admin.id,
            owner_id: owner.id,
            member_id: member.id,
            chat_id: chat.id,
        }
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
