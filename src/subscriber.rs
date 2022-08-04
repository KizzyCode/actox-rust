//! Implements a message subscriber

use crate::queue::{self, Reader, Writer};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicU64, Ordering::SeqCst},
        Arc,
    },
};

/// A process-scoped unique ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueID {
    /// The unique ID
    uid: u64,
}
impl UniqueID {
    /// Creates a new process-scoped unique ID
    pub fn unique() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self { uid: COUNTER.fetch_add(1, SeqCst) }
    }
}

/// A subscriber
pub struct Subscriber<M> {
    /// The subscriber UID
    uid: UniqueID,
    /// The message writer
    writer: Arc<Writer<M>>,
    /// The message reader
    reader: Reader<M>,
}
impl<M> Subscriber<M> {
    /// Creates a new subscriber with the given backlog limit
    pub fn new(backlog: usize) -> Self {
        let (writer, reader) = queue::new(backlog);
        Self { uid: UniqueID::unique(), writer: Arc::new(writer), reader }
    }

    /// The subscribers UID
    pub fn uid(&self) -> &UniqueID {
        &self.uid
    }
    /// Creates a new writer for the subscriber
    pub(in crate) fn writer(&self) -> Arc<Writer<M>> {
        Arc::clone(&self.writer)
    }
}
impl<M> PartialEq for Subscriber<M> {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}
impl<M> Eq for Subscriber<M> {
    /* No members to implement */
}
impl<M> PartialOrd for Subscriber<M> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.uid.partial_cmp(&other.uid)
    }
}
impl<M> Ord for Subscriber<M> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid.cmp(&other.uid)
    }
}
impl<M> Hash for Subscriber<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}
impl<M> Deref for Subscriber<M> {
    type Target = Reader<M>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}
impl<M> DerefMut for Subscriber<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
