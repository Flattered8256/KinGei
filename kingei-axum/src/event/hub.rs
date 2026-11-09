use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use axum::response::sse::Event;
use futures::stream::Stream;
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

use crate::event::ClientEvent;

const CHANNEL_CAPACITY: usize = 64;
const MAX_CONNECTIONS_PER_USER: usize = 4;

struct ConnSlot {
    id: u64,
    cancel: oneshot::Sender<()>,
}

struct UserMailbox {
    sender: broadcast::Sender<Arc<ClientEvent>>,
    conns: VecDeque<ConnSlot>,
}

struct Inner {
    mailboxes: RwLock<HashMap<Uuid, UserMailbox>>,
    next_conn_id: AtomicU64,
    presence: mpsc::UnboundedSender<PresenceSignal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceSignal {
    Connected(Uuid),
    Disconnected(Uuid),
}

#[derive(Clone)]
pub struct EventHub {
    inner: Arc<Inner>,
}

pub struct EventSubscription {
    rx: broadcast::Receiver<Arc<ClientEvent>>,
    cancel: oneshot::Receiver<()>,
    guard: ConnectionGuard,
}

struct ConnectionGuard {
    inner: Arc<Inner>,
    user_id: Uuid,
    conn_id: u64,
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHub {
    pub fn pair() -> (Self, mpsc::UnboundedReceiver<PresenceSignal>) {
        let (presence, rx) = mpsc::unbounded_channel();
        let hub = Self {
            inner: Arc::new(Inner {
                mailboxes: RwLock::new(HashMap::new()),
                next_conn_id: AtomicU64::new(1),
                presence,
            }),
        };
        (hub, rx)
    }

    pub fn new() -> Self {
        Self::pair().0
    }

    pub fn subscribe(&self, user_id: Uuid) -> EventSubscription {
        let conn_id = self.inner.next_conn_id.fetch_add(1, Ordering::Relaxed);
        let (cancel_tx, cancel_rx) = oneshot::channel();

        let mut mailboxes = write_lock(&self.inner.mailboxes);
        let mailbox = mailboxes.entry(user_id).or_insert_with(|| {
            let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
            UserMailbox {
                sender,
                conns: VecDeque::new(),
            }
        });
        let first = mailbox.conns.is_empty();

        // 同一用户最多 4 条 SSE，超出关掉最老的，避免刷新页面堆连接。
        while mailbox.conns.len() >= MAX_CONNECTIONS_PER_USER {
            if let Some(old) = mailbox.conns.pop_front() {
                let _ = old.cancel.send(());
            }
        }

        mailbox.conns.push_back(ConnSlot {
            id: conn_id,
            cancel: cancel_tx,
        });
        let rx = mailbox.sender.subscribe();
        drop(mailboxes);
        if first {
            let _ = self.inner.presence.send(PresenceSignal::Connected(user_id));
        }

        EventSubscription {
            rx,
            cancel: cancel_rx,
            guard: ConnectionGuard {
                inner: self.inner.clone(),
                user_id,
                conn_id,
            },
        }
    }

    pub fn publish(&self, user_id: Uuid, event: Arc<ClientEvent>) {
        let mailboxes = read_lock(&self.inner.mailboxes);
        if let Some(mailbox) = mailboxes.get(&user_id) {
            let _ = mailbox.sender.send(event);
        }
    }

    pub fn disconnect(&self, user_id: Uuid) {
        let mut mailboxes = write_lock(&self.inner.mailboxes);
        let Some(mailbox) = mailboxes.remove(&user_id) else {
            return;
        };
        for conn in mailbox.conns {
            let _ = conn.cancel.send(());
        }
        drop(mailboxes);
        let _ = self
            .inner
            .presence
            .send(PresenceSignal::Disconnected(user_id));
    }

    pub fn publish_many(&self, user_ids: impl IntoIterator<Item = Uuid>, event: Arc<ClientEvent>) {
        let mailboxes = read_lock(&self.inner.mailboxes);
        for user_id in user_ids {
            if let Some(mailbox) = mailboxes.get(&user_id) {
                let _ = mailbox.sender.send(event.clone());
            }
        }
    }
}

impl EventSubscription {
    pub fn into_sse_stream(self) -> impl Stream<Item = Result<Event, Infallible>> {
        async_stream::stream! {
            let EventSubscription {
                mut rx,
                mut cancel,
                guard,
            } = self;
            let _guard = guard;

            loop {
                tokio::select! {
                    biased;
                    _ = &mut cancel => break,
                    result = rx.recv() => {
                        match result {
                            Ok(event) => match event.to_sse_event() {
                                Ok(frame) => yield Ok(frame),
                                Err(e) => {
                                    tracing::error!(?e, "sse serialize error");
                                    break;
                                }
                            },
                            // 落后则关流，客户端重连后走 REST 对齐，避免静默丢事件。
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!(skipped, "sse subscriber lagged");
                                break;
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let mut mailboxes = write_lock(&self.inner.mailboxes);
        let Some(mailbox) = mailboxes.get_mut(&self.user_id) else {
            return;
        };
        mailbox.conns.retain(|conn| conn.id != self.conn_id);
        if mailbox.conns.is_empty() {
            mailboxes.remove(&self.user_id);
            drop(mailboxes);
            let _ = self
                .inner
                .presence
                .send(PresenceSignal::Disconnected(self.user_id));
        }
    }
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
    fn first_subscribe_emits_connected_last_drop_emits_disconnected() {
        let (hub, mut rx) = EventHub::pair();
        let user_id = Uuid::new_v4();

        let first = hub.subscribe(user_id);
        assert_eq!(rx.try_recv(), Ok(PresenceSignal::Connected(user_id)));

        let second = hub.subscribe(user_id);
        assert!(rx.try_recv().is_err());

        drop(second);
        assert!(rx.try_recv().is_err());

        drop(first);
        assert_eq!(rx.try_recv(), Ok(PresenceSignal::Disconnected(user_id)));
    }

    #[test]
    fn disconnect_emits_once_and_drops_do_not_repeat() {
        let (hub, mut rx) = EventHub::pair();
        let user_id = Uuid::new_v4();
        let sub = hub.subscribe(user_id);
        assert_eq!(rx.try_recv(), Ok(PresenceSignal::Connected(user_id)));

        hub.disconnect(user_id);
        assert_eq!(rx.try_recv(), Ok(PresenceSignal::Disconnected(user_id)));

        drop(sub);
        assert!(rx.try_recv().is_err());
    }
}
