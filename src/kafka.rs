//https://github.com/fede1024/rust-rdkafka/blob/master/examples/roundtrip.rs
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};

#[tokio::main]
async fn main() -> Result<(), rdkafka::error::KafkaError> {
    let brokers = "localhost:9092";
    let topic = "events";

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .set("group.id", "rust-rdkafka-roundtrip-example")
        .create()
        .expect("Consumer creation failed");
    consumer.subscribe(&[&topic]).unwrap();

    tokio::spawn(async move {
        let mut i = 0_usize;
        loop {
            producer
                .send_result(FutureRecord::to(&topic).key(&i.to_string()).payload("data"))
                .unwrap()
                .await
                .unwrap()
                .unwrap();
            i += 1;
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
