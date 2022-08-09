//! Implements a thread-safe multi-producer single-consumer queue

use std::{
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError, TrySendError},
        Arc,
    },
    time::Duration,
};

/// A MP/SC queue writer
pub struct Writer<T> {
    /// The disconnected flag
    disconnected: Arc<AtomicBool>,
    /// The underlying sender
    sender: SyncSender<T>,
}
impl<T> Writer<T> {
    /// Creates a new writer
    fn new(disconnected: &Arc<AtomicBool>, sender: SyncSender<T>) -> Self {
        Self { disconnected: disconnected.clone(), sender }
    }

    /// Whether the queue is connected or not
    pub fn disconnected(&self) -> bool {
        self.disconnected.load(SeqCst)
    }

    /// Tries to write an element to the queue
    pub fn try_write(&self, element: T) -> Result<(), T> {
        match self.sender.try_send(element) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(element)) => {
                // The queue is full, but not disconnected
                Err(element)
            }
            Err(TrySendError::Disconnected(element)) => {
                // Mark the connection as disconnected
                self.disconnected.store(true, SeqCst);
                Err(element)
            }
        }
    }
}
impl<T> Drop for Writer<T> {
    fn drop(&mut self) {
        self.disconnected.store(true, SeqCst);
    }
}

/// A MP/SC reader
pub struct Reader<T> {
    /// The disconnected flag
    disconnected: Arc<AtomicBool>,
    /// The underlying receiver
    receiver: Receiver<T>,
}
impl<T> Reader<T> {
    /// Creates a new reader
    fn new(disconnected: &Arc<AtomicBool>, receiver: Receiver<T>) -> Self {
        Self { disconnected: disconnected.clone(), receiver }
    }

    /// Whether the queue is disconnected or not
    pub fn disconnected(&self) -> bool {
        self.disconnected.load(SeqCst)
    }

    /// Reads an element or returns `None` if the queue gets disconnected
    pub fn read(&self) -> Option<T> {
        match self.receiver.recv() {
            Ok(element) => Some(element),
            Err(_) => {
                // Mark the connection as disconnected
                self.disconnected.store(true, SeqCst);
                None
            }
        }
    }
    /// Tries to read an element
    pub fn try_read(&self) -> Option<T> {
        match self.receiver.try_recv() {
            Ok(element) => Some(element),
            Err(TryRecvError::Empty) => {
                // The queue is empty, but not disconnected
                None
            }
            Err(TryRecvError::Disconnected) => {
                // Mark the connection as disconnected
                self.disconnected.store(true, SeqCst);
                None
            }
        }
    }
    /// Reads the next element from the queue or returns if the timeout is reached
    pub fn read_timeout(&self, timeout: Duration) -> Option<T> {
        match self.receiver.recv_timeout(timeout) {
            Ok(element) => Some(element),
            Err(RecvTimeoutError::Timeout) => {
                // The queue is empty, but not disconnected
                None
            }
            Err(RecvTimeoutError::Disconnected) => {
                // Mark the connection as disconnected
                self.disconnected.store(true, SeqCst);
                None
            }
        }
    }
}
impl<T> Drop for Reader<T> {
    fn drop(&mut self) {
        self.disconnected.store(true, SeqCst);
    }
}

/// Creates a new writer-reader pair
pub fn new<T>(limit: usize) -> (Writer<T>, Reader<T>) {
    let (sender, receiver) = mpsc::sync_channel(limit);
    let disconnected = Arc::default();
    (Writer::new(&disconnected, sender), Reader::new(&disconnected, receiver))
}
