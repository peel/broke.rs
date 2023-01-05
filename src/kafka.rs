//https://github.com/fede1024/rust-rdkafka/blob/master/examples/roundtrip.rs
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::env;

use crate::data::EVENT;
mod data;

#[tokio::main]
async fn main() -> Result<(), rdkafka::error::KafkaError> {
    let brokers = env::var("BROKER_URL").unwrap_or_else(|_| "localhost:9092".to_string());
    let entity_count = env::var("ENTITY_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or_else(|| 1);
    let event_count = env::var("EVENT_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok());
    let topic = env::var("TOPIC").unwrap_or_else(|_| "events".to_string());

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("group.id", "rust-rdkafka-roundtrip-example")
        .create()
        .expect("Consumer creation failed");
    consumer.subscribe(&[&topic]).unwrap();

    tokio::spawn(async move {
        let mut i = 0_usize;
        loop {
            if event_count.is_none() || event_count.unwrap() > i {
                producer
                    .send_result(FutureRecord::to(&topic).key(&i.to_string()).payload(EVENT))
                    .unwrap()
                    .await
                    .unwrap()
                    .unwrap();
                i += 1;
            } else {
                break;
            }
        }
    });

    while let Ok(message) = consumer.recv().await {
        println!(
            "got message on topic {} with payload {:?}",
            message.topic(),
            message.payload()
        );
    }

    Ok(())
}
