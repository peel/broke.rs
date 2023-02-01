use std::error::Error;

use clap::{arg, value_parser, ArgAction, Command};
use stream_operator::Mode;
mod kafka;
mod nats;
mod pulsar;
mod rabbit;

fn cli() -> Command {
    let subcommands = ["kafka", "nats", "pubsub", "pulsar", "rabbit"]
        .map(|b| Command::new(b).about(format!("Generate/consume events to/from {b}")));
    Command::new("stream-operator")
        .version("1.0.0")
        .about("Snowplow broker test utility")
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .arg(arg!(--brokers <BROKER_URL> "Seed brokers URIs"))
        .arg(arg!(--topic <TOPIC> "Topic to publish/subscribe to"))
        .arg(
            arg!(--entities <ENTITY_COUNT> "Number of entities within event")
                .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!(--events <EVENT_COUNT> "Number of events to send")
                .value_parser(value_parser!(usize)),
        )
        .arg(
            arg!(--modes <MODE_TYPE> "Run modes")
                .value_parser(value_parser!(Mode))
                .action(ArgAction::Append),
        )
        .subcommands(subcommands)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let root = cli().get_matches();
    let brokers = root.get_one::<String>("brokers").unwrap().to_string();
    let topic = root.get_one::<String>("topic").unwrap().to_string();
    let entity_count = root.get_one::<usize>("entities").unwrap();
    let event_count = root.get_one::<usize>("events").unwrap();
    let modes: Vec<&Mode> = root.get_many::<Mode>("modes").unwrap().collect();
    match root.subcommand() {
        Some(("kafka", _)) => {
            kafka::run(brokers, topic, *entity_count, Some(*event_count), modes).await
        }
        Some(("nats", _)) => {
            nats::run(brokers, topic, *entity_count, Some(*event_count), modes).await
        }
        Some(("pulsar", _)) => {
            pulsar::run(brokers, topic, *entity_count, Some(*event_count), modes).await
        }
        Some(("rabbit", _)) => {
            rabbit::run(brokers, topic, *entity_count, Some(*event_count), modes).await
        }
        _ => unreachable!(),
    }
}
