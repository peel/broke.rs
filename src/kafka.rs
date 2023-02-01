use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::env;

use stream_operator::data::event;
use stream_operator::Mode;

pub async fn run(
    broker_url: String,
    topic: String,
    entity_count: usize,
    event_count: Option<usize>,
    modes: Vec<&Mode>,
) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
    if modes.contains(&&Mode::Sub) {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &broker_url)
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("group.id", "rust-rdkafka-roundtrip-example")
            .create()
            .expect("Consumer creation failed");
        consumer.subscribe(&[&topic]).unwrap();

        while let Ok(message) = consumer.recv().await {
            println!(
                "got message on topic {} with payload {:?}",
                message.topic(),
                message.payload()
            );
        }
    }

    if modes.contains(&&Mode::Pub) {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &broker_url)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        tokio::spawn(async move {
            let mut i = 0_usize;
            loop {
                if event_count.is_none() || event_count.unwrap() > i {
                    producer
                        .send_result(
                            FutureRecord::to(&topic)
                                .key(&i.to_string())
                                .payload(&event(entity_count)),
                        )
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
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
    let brokers = env::var("BROKER_URL").unwrap_or_else(|_| "localhost:9092".to_string());
    let entity_count = env::var("ENTITY_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or_else(|| 1);
    let event_count = env::var("EVENT_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok());
    let topic = env::var("TOPIC").unwrap_or_else(|_| "events".to_string());

    run(
        brokers,
        topic,
        entity_count,
        event_count,
        vec![&Mode::Pub, &Mode::Sub],
    )
    .await
}
