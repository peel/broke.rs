use std::{env, str::from_utf8};

use async_nats::jetstream::{self, consumer::PullConsumer};
use futures::StreamExt;

use crate::data::event;
mod data;

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    let nats_url = env::var("BROKER_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);
    let stream_name = String::from("EVENTS");

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
            jetstream
                .publish(format!("events.{}", i), event.into())
                .await
                .unwrap();
            i += 1;
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
