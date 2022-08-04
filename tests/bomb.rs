//! Implements a stress test
//!
//! Run with: `cargo test --release test_bomb -- --ignored`

use actox::{Bus, Subscriber};
use std::{thread, time::Duration};

/// The bomb level - greater means more stress
const BOMB_LEVEL: usize = 75;
/// The bomb timeout
const TIMEOUT: Duration = Duration::from_secs(5 * 60);

#[test]
fn test_bomb() {
    // Prepare bus
    let bus: Bus<usize, u8> = Bus::new();
    let message = 7;

    // Create subscriber threads
    let mut subscriber_threads = Vec::new();
    for _ in 0..BOMB_LEVEL {
        // Subscribe to topics
        let subscriber = Subscriber::new(BOMB_LEVEL * BOMB_LEVEL * BOMB_LEVEL);
        for index in 0..BOMB_LEVEL {
            bus.subscribe(&index, &subscriber);
        }

        // Spawn and register the receiver thread
        let thread_handle = thread::spawn(move || {
            // Receive all expected messages
            let expected = BOMB_LEVEL * BOMB_LEVEL * BOMB_LEVEL;
            for _ in 0..expected {
                // Receive the message
                let message = subscriber.read_timeout(TIMEOUT).expect("Failed to receive expected message");
                assert_eq!(message, 7, "Invalid message value?!");
            }
        });
        subscriber_threads.push(thread_handle);
    }

    // Create publisher threads
    for _ in 0..BOMB_LEVEL {
        // Spawn the sender thread
        let dispatch = bus.clone();
        thread::spawn(move || {
            // Publish messages
            for _ in 0..BOMB_LEVEL {
                // Publish the message to each topic
                for index in 0..BOMB_LEVEL {
                    dispatch.publish(&index, message);
                }
            }
        });
    }

    // Wait for all subscriber threads
    for thread_handle in subscriber_threads {
        thread_handle.join().expect("Subscriber thread has panicked?!");
    }
}
