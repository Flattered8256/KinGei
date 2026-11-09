use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::user::UserError;
use crate::event::{ClientEvent, EventHub, PresenceSignal};
use crate::model::user::UserRole;
use crate::repository::chat::ChatRepository;
use crate::repository::friend::{FriendRepository, FriendRequestRepository};
use crate::repository::user::UserRepository;
use crate::serializer::event::PresenceChangedEvent;

pub const GRACE: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct PresenceService {
    friend_repo: Arc<FriendRepository>,
    request_repo: Arc<FriendRequestRepository>,
    chat_repo: Arc<ChatRepository>,
    user_repo: Arc<UserRepository>,
    event_hub: EventHub,
    tracker: Arc<PresenceTracker>,
    grace: Duration,
}

struct PresenceTracker {
    inner: RwLock<TrackerInner>,
}

struct TrackerInner {
    connected: HashSet<Uuid>,
    online: HashSet<Uuid>,
    generation: HashMap<Uuid, u64>,
}

impl PresenceService {
    pub fn new(
        friend_repo: Arc<FriendRepository>,
        request_repo: Arc<FriendRequestRepository>,
        chat_repo: Arc<ChatRepository>,
        user_repo: Arc<UserRepository>,
        event_hub: EventHub,
        grace: Duration,
    ) -> Self {
        Self {
            friend_repo,
            request_repo,
            chat_repo,
            user_repo,
            event_hub,
            tracker: Arc::new(PresenceTracker::new()),
            grace,
        }
    }

    pub fn spawn(self, rx: mpsc::UnboundedReceiver<PresenceSignal>) {
        tokio::spawn(async move {
            self.run(rx).await;
        });
    }

    pub async fn snapshot(&self, viewer: Uuid) -> Result<Vec<PresenceChangedEvent>, AppError> {
        let user = self
            .user_repo
            .find_by_id(viewer)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        let online = self.tracker.online_ids();
        let visible = if user.role == UserRole::Admin {
            online
        } else {
            let allowed = self.visible_ids(viewer).await?;
            online.intersection(&allowed).copied().collect()
        };
        Ok(visible
            .into_iter()
            .map(|user_id| PresenceChangedEvent {
                user_id,
                online: true,
            })
            .collect())
    }

    async fn run(self, mut rx: mpsc::UnboundedReceiver<PresenceSignal>) {
        while let Some(signal) = rx.recv().await {
            match signal {
                PresenceSignal::Connected(user_id) => {
                    if self.tracker.on_connected(user_id) {
                        self.broadcast(user_id, true).await;
                    }
                }
                PresenceSignal::Disconnected(user_id) => {
                    let generation = self.tracker.on_disconnected(user_id);
                    let service = self.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(service.grace).await;
                        if service.tracker.expire_grace(user_id, generation) {
                            service.broadcast(user_id, false).await;
                        }
                    });
                }
            }
        }
    }

    async fn broadcast(&self, user_id: Uuid, online: bool) {
        match self.audience(user_id).await {
            Ok(audience) => {
                let event = Arc::new(ClientEvent::presence_changed(user_id, online));
                self.event_hub
                    .publish_many(audience.into_iter().filter(|id| *id != user_id), event);
            }
            Err(e) => {
                tracing::error!(?e, %user_id, "presence audience failed");
            }
        }
    }

    async fn visible_ids(&self, user_id: Uuid) -> Result<HashSet<Uuid>, AppError> {
        let mut ids = self.audience(user_id).await?;
        ids.insert(user_id);
        Ok(ids)
    }

    async fn audience(&self, user_id: Uuid) -> Result<HashSet<Uuid>, AppError> {
        let friends = self.friend_repo.list(user_id).await?;
        let pending = self
            .request_repo
            .list_pending_counterpart_ids(user_id)
            .await?;
        let peers = self.chat_repo.list_peer_ids(user_id).await?;
        let admins = self.user_repo.list_ids_by_role(UserRole::Admin).await?;

        let mut ids =
            HashSet::with_capacity(friends.len() + pending.len() + peers.len() + admins.len());
        ids.extend(friends.into_iter().map(|friend| friend.friend_id));
        ids.extend(pending);
        ids.extend(peers);
        ids.extend(admins);
        Ok(ids)
    }
}

impl PresenceTracker {
    fn new() -> Self {
        Self {
            inner: RwLock::new(TrackerInner {
                connected: HashSet::new(),
                online: HashSet::new(),
                generation: HashMap::new(),
            }),
        }
    }

    fn on_connected(&self, user_id: Uuid) -> bool {
        let mut inner = write_lock(&self.inner);
        inner.connected.insert(user_id);
        bump_generation(&mut inner.generation, user_id);
        inner.online.insert(user_id)
    }

    fn on_disconnected(&self, user_id: Uuid) -> u64 {
        let mut inner = write_lock(&self.inner);
        inner.connected.remove(&user_id);
        bump_generation(&mut inner.generation, user_id)
    }

    fn expire_grace(&self, user_id: Uuid, generation: u64) -> bool {
        let mut inner = write_lock(&self.inner);
        if inner.generation.get(&user_id) != Some(&generation) {
            return false;
        }
        if inner.connected.contains(&user_id) {
            return false;
        }
        inner.online.remove(&user_id);
        inner.generation.remove(&user_id);
        true
    }

    fn online_ids(&self) -> HashSet<Uuid> {
        read_lock(&self.inner).online.clone()
    }
}

fn bump_generation(generation: &mut HashMap<Uuid, u64>, user_id: Uuid) -> u64 {
    let slot = generation.entry(user_id).or_insert(0);
    *slot += 1;
    *slot
}

fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_marks_online() {
        let tracker = PresenceTracker::new();
        let user_id = Uuid::new_v4();
        assert!(tracker.on_connected(user_id));
        assert!(tracker.online_ids().contains(&user_id));
        assert!(!tracker.on_connected(user_id));
    }

    #[test]
    fn reconnect_during_grace_stays_online() {
        let tracker = PresenceTracker::new();
        let user_id = Uuid::new_v4();
        tracker.on_connected(user_id);
        let generation = tracker.on_disconnected(user_id);
        assert!(tracker.online_ids().contains(&user_id));

        assert!(!tracker.on_connected(user_id));
        assert!(!tracker.expire_grace(user_id, generation));
        assert!(tracker.online_ids().contains(&user_id));
    }

    #[test]
    fn stale_generation_does_not_expire() {
        let tracker = PresenceTracker::new();
        let user_id = Uuid::new_v4();
        tracker.on_connected(user_id);
        let generation = tracker.on_disconnected(user_id);
        assert!(!tracker.expire_grace(user_id, generation.wrapping_sub(1)));
        assert!(tracker.online_ids().contains(&user_id));
        assert!(tracker.expire_grace(user_id, generation));
        assert!(!tracker.online_ids().contains(&user_id));
    }
}
