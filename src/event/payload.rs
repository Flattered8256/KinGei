use axum::response::sse::Event;
use uuid::Uuid;

use crate::serializer::chat::ChatResponse;
use crate::serializer::event::{
    ChatDeletedEvent, ChatMemberRemovedEvent, FriendDeletedEvent, ReadReceiptEvent,
};
use crate::serializer::friend::FriendRequestResponse;
use crate::serializer::message::MessageResponse;

#[derive(Debug, Clone)]
pub enum ClientEvent {
    MessageCreated(MessageResponse),
    MessageRead(ReadReceiptEvent),
    ChatCreated(ChatResponse),
    ChatUpdated(ChatResponse),
    ChatMemberAdded(ChatResponse),
    ChatMemberRemoved(ChatMemberRemovedEvent),
    ChatDeleted(ChatDeletedEvent),
    FriendRequestCreated(FriendRequestResponse),
    FriendRequestAccepted(FriendRequestResponse),
    FriendRequestRejected(FriendRequestResponse),
    FriendRequestCancelled(FriendRequestResponse),
    FriendDeleted(FriendDeletedEvent),
}

impl ClientEvent {
    fn event_name(&self) -> &'static str {
        match self {
            Self::MessageCreated(_) => "message.created",
            Self::MessageRead(_) => "message.read",
            Self::ChatCreated(_) => "chat.created",
            Self::ChatUpdated(_) => "chat.updated",
            Self::ChatMemberAdded(_) => "chat.member.added",
            Self::ChatMemberRemoved(_) => "chat.member.removed",
            Self::ChatDeleted(_) => "chat.deleted",
            Self::FriendRequestCreated(_) => "friend.request.created",
            Self::FriendRequestAccepted(_) => "friend.request.accepted",
            Self::FriendRequestRejected(_) => "friend.request.rejected",
            Self::FriendRequestCancelled(_) => "friend.request.cancelled",
            Self::FriendDeleted(_) => "friend.deleted",
        }
    }

    pub fn to_sse_event(&self) -> Result<Event, axum::Error> {
        let event = Event::default().event(self.event_name());
        match self {
            Self::MessageCreated(data) => event.json_data(data),
            Self::MessageRead(data) => event.json_data(data),
            Self::ChatCreated(data) => event.json_data(data),
            Self::ChatUpdated(data) => event.json_data(data),
            Self::ChatMemberAdded(data) => event.json_data(data),
            Self::ChatMemberRemoved(data) => event.json_data(data),
            Self::ChatDeleted(data) => event.json_data(data),
            Self::FriendRequestCreated(data) => event.json_data(data),
            Self::FriendRequestAccepted(data) => event.json_data(data),
            Self::FriendRequestRejected(data) => event.json_data(data),
            Self::FriendRequestCancelled(data) => event.json_data(data),
            Self::FriendDeleted(data) => event.json_data(data),
        }
    }

    pub fn chat_member_removed(chat_id: i64, user_id: Uuid) -> Self {
        Self::ChatMemberRemoved(ChatMemberRemovedEvent { chat_id, user_id })
    }

    pub fn chat_deleted(chat_id: i64) -> Self {
        Self::ChatDeleted(ChatDeletedEvent { chat_id })
    }

    pub fn friend_deleted(friend_id: Uuid) -> Self {
        Self::FriendDeleted(FriendDeletedEvent { friend_id })
    }

    pub fn message_read(chat_id: i64, user_id: Uuid, last_read_message_id: i64) -> Self {
        Self::MessageRead(ReadReceiptEvent {
            chat_id,
            user_id,
            last_read_message_id,
        })
    }
}
