// https://raw.githubusercontent.com/amqp-rs/lapin/main/examples/tokio.rs
use lapin::{
    message::DeliveryResult,
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use std::env;

use crate::data::event;
mod data;

#[tokio::main]
async fn main() {
    let broker_url = env::var("BROKER_URL").unwrap_or_else(|_| "amqp://localhost:5672".to_string());
    let entity_count = env::var("ENTITY_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or_else(|| 1);
    let event_count = env::var("EVENT_COUNT")
        .ok()
        .and_then(|count| count.parse::<usize>().ok());
    let topic = env::var("TOPIC").unwrap_or_else(|_| "events".to_string());

    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);

    let connection = Connection::connect(&broker_url, options).await.unwrap();
    let channel = connection.create_channel().await.unwrap();

    let _queue = channel
        .queue_declare(
            &topic,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let consumer = channel
        .basic_consume(
            "queue_test",
            "tag_foo",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = match delivery {
            // Carries the delivery alongside its channel
            Ok(Some(delivery)) => delivery,
            // The consumer got canceled
            Ok(None) => return,
            // Carries the error and is always followed by Ok(None)
            Err(error) => {
                dbg!("Failed to consume queue message {}", error);
                return;
            }
        };

        // Do something with the delivery data (The message payload)
        match std::str::from_utf8(&delivery.data) {
            Ok(m) => println!("Received {}", m),
            Err(e) => println!("Received invalid sequence {}", e),
        };
        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("Failed to ack send_webhook_event message");
    });

    tokio::spawn(async move {
        let mut i = 0_usize;
        loop {
            if event_count.is_none() || event_count.unwrap() > i {
                channel
                    .basic_publish(
                        "",
                        "queue_test",
                        BasicPublishOptions::default(),
                        event(entity_count).as_bytes(),
                        BasicProperties::default(),
                    )
                    .await
                    .unwrap()
                    .await
                    .unwrap();
                i += 1
            } else {
                break;
            }
        }
    });
}
