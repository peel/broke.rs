//https://github.com/streamnative/pulsar-rs/blob/master/examples/round_trip.rs
use serde::{Deserialize, Serialize};
use std::env;

use futures::TryStreamExt;
use pulsar::{
    message::{proto, proto::command_subscribe::SubType, Payload},
    producer, Consumer, DeserializeMessage, Error as PulsarError, Pulsar, SerializeMessage,
    TokioExecutor,
};

use stream_operator::data::event;
use stream_operator::Mode;

#[derive(Serialize, Deserialize)]
struct TestData {
    data: String,
}

impl SerializeMessage for TestData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for TestData {
    type Output = Result<TestData, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
pub async fn run(
    broker_url: String,
    topic: String,
    entity_count: usize,
    event_count: Option<usize>,
    modes: Vec<&Mode>,
) -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
    let pulsar: Pulsar<_> = Pulsar::builder(&broker_url, TokioExecutor).build().await?;
    if modes.contains(&&Mode::Pub) {
        let mut producer = pulsar
            .producer()
            .with_topic(&topic)
            .with_name("{topic}-producer")
            .with_options(producer::ProducerOptions {
                schema: Some(proto::Schema {
                    r#type: proto::schema::Type::String as i32,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .build()
            .await?;

        tokio::task::spawn(async move {
            let mut counter = 0usize;
            loop {
                producer
                    .send(TestData {
                        data: event(entity_count).to_string(),
                    })
                    .await
                    .unwrap()
                    .await
                    .unwrap();
                counter += 1;
                if counter % 1000 == 0 {
                    println!("sent {} messages", counter);
                }
            }
        });
    }
    if modes.contains(&&Mode::Sub) {
        let pulsar2: Pulsar<_> = Pulsar::builder(&broker_url, TokioExecutor).build().await?;

        let mut consumer: Consumer<TestData, _> = pulsar2
            .consumer()
            .with_topic(&topic)
            .with_consumer_name("{topic}_consumer")
            .with_subscription_type(SubType::Exclusive)
            .with_subscription("{topic}_subscription")
            .build()
            .await?;

        let mut counter = 0usize;
        while let Some(msg) = consumer.try_next().await? {
            println!("id: {:?}", msg.message_id());
            consumer.ack(&msg).await?;
            let data = msg.deserialize().unwrap();
            if data.data.as_str() != "data" {
                panic!("Unexpected payload: {}", &data.data);
            }
            counter += 1;
            if counter % 1000 == 0 {
                println!("received {} messages", counter);
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + std::marker::Send>> {
    let brokers = env::var("BROKER_URL").unwrap_or_else(|_| "pulsar://127.0.0.1:6650".to_string());
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
