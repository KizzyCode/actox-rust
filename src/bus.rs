//! Implements a shared message bus

use crate::{
    queue::Writer,
    subscriber::{Subscriber, UniqueID},
};
use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

/// A topic subscription channel
type Subscription<M> = Arc<Writer<M>>;
/// An `Arc`ed RW lock
type Lock<T> = Arc<RwLock<T>>;

/// A shared message bus
#[derive(Default, Clone)]
pub struct Dispatch<T, M> {
    /// The topics together with their subscribers
    topics: Lock<HashMap<T, HashMap<UniqueID, Subscription<M>>>>,
}
impl<T, M> Dispatch<T, M> {
    /// Creates a new bus
    pub fn new() -> Self {
        Self { topics: Arc::default() }
    }

    /// Lists all topics
    pub fn topics(&self) -> Vec<T>
    where
        T: Clone,
    {
        // Lock the topic list and clone all topics
        let topics = self.topics.read().expect("Some thread has panicked while dispatching?!");
        topics.keys().cloned().collect()
    }
    /// Publishes a message to all available subscribers for the given topic
    ///
    /// # Note
    /// The message is not retained for future subscribers. Furthermore, if a subscriber's queue is full, the message won't
    /// be delivered to this subscriber but is dropped instead.
    pub fn publish<Q>(&self, topic: &Q, message: M)
    where
        T: Borrow<Q> + Eq + Hash,
        M: Clone,
        Q: Eq + Hash,
    {
        // Lock the topic list and get the subscribers for the given topic
        let topics = self.topics.read().expect("Some thread has panicked while dispatching?!");
        if let Some(subscribers) = topics.get(topic) {
            // Send the message to each subscriber
            for subscriber in subscribers.values() {
                // This is a best-effort write; if the subscriber's queue is full, the message will be lost
                let _ = subscriber.try_write(message.clone());
            }
        }
    }

    /// Subscribes to a topic
    pub fn subscribe<Q>(&self, topic: &Q, subscriber: &Subscriber<M>)
    where
        T: Borrow<Q> + Eq + Hash,
        Q: ToOwned<Owned = T> + Eq + Hash,
    {
        // Lock the topic list and insert a new subscriber map if necessary
        let mut topics = self.topics.write().expect("Some thread has panicked while dispatching?!");
        if !topics.contains_key(topic) {
            topics.insert(topic.to_owned(), HashMap::new());
        }

        // Register our subscriber
        let subscribers = topics.get_mut(topic).expect("No subscriber map for given topic?!");
        subscribers.insert(*subscriber.uid(), subscriber.writer());
    }

    /// Unsubscribes from a topic
    pub fn unsubscribe<Q>(&self, topic: &Q, subscriber: &Subscriber<M>)
    where
        T: Borrow<Q> + Eq + Hash,
        Q: Eq + Hash,
    {
        // Lock the topic list and remove our subscriber
        let mut topics = self.topics.write().expect("Some thread has panicked while dispatching?!");
        if let Some(subscribers) = topics.get_mut(topic) {
            subscribers.remove(subscriber.uid());
        }
    }

    /// Deallocates unused memory
    ///
    /// # Note
    /// Depending on the size of the dispatcher, this function may block the dispatcher for a significant amount of time.
    pub fn shrink_to_fit(&self)
    where
        T: Eq + Hash,
    {
        // Lock the topic list
        let mut topics = self.topics.write().expect("Some thread has panicked while dispatching?!");

        // Remove all obsolete writers and subscriber maps
        for subscribers in topics.values_mut() {
            subscribers.retain(|_, writer| !writer.disconnected());
        }
        topics.retain(|_, subscribers| !subscribers.is_empty());

        // Call shrink to fit on all remaining maps
        for subscribers in topics.values_mut() {
            subscribers.shrink_to_fit();
        }
        topics.shrink_to_fit();
    }
}
