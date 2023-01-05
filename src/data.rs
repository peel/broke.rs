use base64;
use fake::{Dummy, Fake, Faker};
use serde::Serialize;

#[derive(Debug, Dummy, Serialize)]
struct Email {
    id: String,
}

pub fn event(count: usize) -> String {
    let events = (0..count)
        .collect::<Vec<_>>()
        .iter()
        .map(|_| Faker.fake::<Email>())
        .map(|event| serde_json::to_string::<Email>(&event))
        .fold(String::new(), |mut a, b| {
            a.push_str(b.unwrap().as_str());
            a
        });

    r###"{"schema":"iglu:com.snowplowanalytics.snowplow/payload_data/jsonschema/1-0-4","data":[{"e":"pv","url":"/docs/open-source-quick-start/quick-start-installation-guide-on-aws/send-test-events-to-your-pipeline/","page":"Send test events to your pipeline - Snowplow Docs","refr":"https://docs.snowplow.io/","tv":"js-2.17.2","tna":"spExample","aid":"docs-example","p":"web","tz":"Europe/London","lang":"en-GB","cs":"UTF-8","res":"3440x1440","cd":"24","cookie":"1","eid":"4e35e8c6-03c4-4c17-8202-80de5bd9d953","dtm":"1626182778191","cx":"###.to_owned()  + base64::encode(events).as_str() + r###","vp":"863x1299","ds":"848x5315","vid":"3","sid":"87c18fc8-2055-4ec4-8ad6-fff64081c2f3","duid":"5f06dbb0-a893-472b-b61a-7844032ab3d6","stm":"1626182778194"}]}"###
}
