use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::message::MessageError;
use crate::event::{ClientEvent, EventHub};
use crate::model::chat::{Chat, ChatType};
use crate::repository::chat::ChatRepository;
use crate::repository::friend::FriendRepository;

pub const TTL: Duration = Duration::from_secs(8);

#[derive(Clone)]
pub struct TypingService {
    chat_repo: Arc<ChatRepository>,
    friend_repo: Arc<FriendRepository>,
    event_hub: EventHub,
    tracker: Arc<TypingTracker>,
    ttl: Duration,
}

struct TypingTracker {
    inner: RwLock<TrackerInner>,
}

struct TrackerInner {
    active: HashSet<(i64, Uuid)>,
    generation: HashMap<(i64, Uuid), u64>,
}

impl TypingService {
    pub fn new(
        chat_repo: Arc<ChatRepository>,
        friend_repo: Arc<FriendRepository>,
        event_hub: EventHub,
        ttl: Duration,
    ) -> Self {
        Self {
            chat_repo,
            friend_repo,
            event_hub,
            tracker: Arc::new(TypingTracker::new()),
            ttl,
        }
    }

    pub async fn set(&self, user_id: Uuid, chat_id: i64, typing: bool) -> Result<(), AppError> {
        let chat = self.require_member(user_id, chat_id).await?;
        self.require_direct_friends(user_id, &chat).await?;

        if typing {
            let (generation, _) = self.tracker.start(chat_id, user_id);
            self.schedule_expire(chat_id, user_id, generation);
            self.broadcast(chat_id, user_id, true).await;
        } else if self.tracker.stop(chat_id, user_id).is_some() {
            self.broadcast(chat_id, user_id, false).await;
        }

        Ok(())
    }

    pub async fn clear(&self, user_id: Uuid, chat_id: i64) {
        if self.tracker.stop(chat_id, user_id).is_some() {
            self.broadcast(chat_id, user_id, false).await;
        }
    }

    fn schedule_expire(&self, chat_id: i64, user_id: Uuid, generation: u64) {
        let service = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(service.ttl).await;
            if service.tracker.expire(chat_id, user_id, generation) {
                service.broadcast(chat_id, user_id, false).await;
            }
        });
    }

    async fn broadcast(&self, chat_id: i64, user_id: Uuid, typing: bool) {
        match self.chat_repo.list_members(chat_id).await {
            Ok(members) => {
                let event = Arc::new(ClientEvent::typing_changed(chat_id, user_id, typing));
                self.event_hub.publish_many(
                    members
                        .into_iter()
                        .filter(|member| member.user_id != user_id)
                        .map(|member| member.user_id),
                    event,
                );
            }
            Err(e) => tracing::error!(?e, chat_id, "failed to list members for typing event"),
        }
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

impl TypingTracker {
    fn new() -> Self {
        Self {
            inner: RwLock::new(TrackerInner {
                active: HashSet::new(),
                generation: HashMap::new(),
            }),
        }
    }

    fn start(&self, chat_id: i64, user_id: Uuid) -> (u64, bool) {
        let mut inner = write_lock(&self.inner);
        let key = (chat_id, user_id);
        let generation = bump_generation(&mut inner.generation, key);
        let newly = inner.active.insert(key);
        (generation, newly)
    }

    fn stop(&self, chat_id: i64, user_id: Uuid) -> Option<u64> {
        let mut inner = write_lock(&self.inner);
        let key = (chat_id, user_id);
        if !inner.active.remove(&key) {
            return None;
        }
        let generation = bump_generation(&mut inner.generation, key);
        inner.generation.remove(&key);
        Some(generation)
    }

    fn expire(&self, chat_id: i64, user_id: Uuid, generation: u64) -> bool {
        let mut inner = write_lock(&self.inner);
        let key = (chat_id, user_id);
        if inner.generation.get(&key) != Some(&generation) {
            return false;
        }
        inner.active.remove(&key);
        inner.generation.remove(&key);
        true
    }
}

fn bump_generation(generation: &mut HashMap<(i64, Uuid), u64>, key: (i64, Uuid)) -> u64 {
    let slot = generation.entry(key).or_insert(0);
    *slot += 1;
    *slot
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_marks_active_once() {
        let tracker = TypingTracker::new();
        let user_id = Uuid::new_v4();
        let (generation, newly) = tracker.start(1, user_id);
        assert!(newly);
        assert!(generation > 0);

        let (refresh, newly_again) = tracker.start(1, user_id);
        assert!(!newly_again);
        assert!(refresh > generation);
        assert!(tracker.stop(1, user_id).is_some());
    }

    #[test]
    fn stop_clears_active() {
        let tracker = TypingTracker::new();
        let user_id = Uuid::new_v4();
        tracker.start(1, user_id);
        assert!(tracker.stop(1, user_id).is_some());
        assert!(tracker.stop(1, user_id).is_none());
    }

    #[test]
    fn stale_generation_does_not_expire() {
        let tracker = TypingTracker::new();
        let user_id = Uuid::new_v4();
        let (generation, _) = tracker.start(1, user_id);
        let (refresh, _) = tracker.start(1, user_id);
        assert!(!tracker.expire(1, user_id, generation));
        assert!(tracker.expire(1, user_id, refresh));
        assert!(tracker.stop(1, user_id).is_none());
    }

    #[test]
    fn stop_invalidates_pending_expire() {
        let tracker = TypingTracker::new();
        let user_id = Uuid::new_v4();
        let (generation, _) = tracker.start(1, user_id);
        assert!(tracker.stop(1, user_id).is_some());
        assert!(!tracker.expire(1, user_id, generation));
        assert!(tracker.stop(1, user_id).is_none());
    }
}
