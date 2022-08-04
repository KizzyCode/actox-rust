use actox::{Bus, Subscriber};
use std::{thread, time::Duration};

#[test]
fn test_subscribe() {
    // Prepare bus
    let bus = Bus::new();
    let topic = String::from("dispatch/test/subscribe");
    let message = "Dispatch message";

    // Register new subscribers
    let (subscriber_a, subscriber_b) = (Subscriber::new(1024), Subscriber::new(1204));
    bus.subscribe(&topic, &subscriber_a);
    bus.subscribe(&topic, &subscriber_b);

    // Start publisher
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        bus.publish(&topic, message);
    });

    // Wait for messages
    let message_a = subscriber_a.read_timeout(Duration::from_secs(2)).expect("Failed to receive dispatch message?!");
    let message_b = subscriber_b.read_timeout(Duration::from_secs(2)).expect("Failed to receive dispatch message?!");

    // Compare messages
    assert_eq!("Dispatch message", message_a, "Invalid message payload?!");
    assert_eq!("Dispatch message", message_b, "Invalid message payload?!");
}

#[test]
fn test_unsubscribe() {
    // Prepare bus
    let bus = Bus::new();
    let topic = String::from("dispatch/test/unsubscribe");
    let message = "Dispatch message";

    // Register new subscriber
    let subscriber = Subscriber::new(1024);
    bus.subscribe(&topic, &subscriber);

    // Start publisher
    let (_topic, _dispatch) = (topic.clone(), bus.clone());
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        _dispatch.publish(&_topic, message);
        thread::sleep(Duration::from_secs(3));
        _dispatch.publish(&_topic, message);
    });

    // Wait for first message
    let message = subscriber.read_timeout(Duration::from_secs(2)).expect("Failed to receive dispatch message?!");
    assert_eq!("Dispatch message", message, "Invalid message payload?!");

    // Unregister and wait for second message
    bus.unsubscribe(&topic, &subscriber);
    assert!(subscriber.read_timeout(Duration::from_secs(5)).is_none(), "Received unexpected dispatch message?!");
}
