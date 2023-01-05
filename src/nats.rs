use std::{env, str::from_utf8};

use async_nats::jetstream::{self, consumer::PullConsumer};
use futures::StreamExt;

use crate::data::event;
mod data;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("BROKER_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let entity_count = env::var("ENTITY_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or_else(|| 1);
    let event_count = env::var("EVENT_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok());
    let stream_name = env::var("TOPIC").unwrap_or_else(|_| "events".to_string());

    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);

    let consumer: PullConsumer = jetstream
        .create_stream(jetstream::stream::Config {
            name: stream_name,
            subjects: vec!["events.>".to_string()],
            ..Default::default()
        })
        .await?
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("consumer".to_string()),
            ..Default::default()
        })
        .await?;

    tokio::spawn(async move {
        let mut i = 0_usize;
        loop {
            if event_count.is_none() || event_count.unwrap() >= i {
                jetstream
                    .publish(format!("events.{}", i), event(entity_count).into())
                    .await
                    .unwrap();
                // client.flush();
                i += 1;
            } else {
                break;
            }
        }
    });
    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        println!(
            "got message on subject {} with payload {:?}",
            message.subject,
            from_utf8(&message.payload)?
        );

        message.ack().await?;
    }

    Ok(())
}
