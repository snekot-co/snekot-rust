//! Every method against a running API: `SNEKOT_TOKEN=sk_… SNEKOT_PROJECT=slug cargo run --example smoke`.

use snekot_events::{Client, Error, Group, Properties, Subscribe, Track};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("SNEKOT_TOKEN").expect("set SNEKOT_TOKEN");
    let project = std::env::var("SNEKOT_PROJECT").expect("set SNEKOT_PROJECT");
    let endpoint = std::env::var("SNEKOT_ENDPOINT").unwrap_or_else(|_| snekot_events::DEFAULT_ENDPOINT.to_owned());
    let events = Client::builder(token, project).endpoint(endpoint).build();

    events.track(Track::new("sdk", "Rust smoke test").icon(":crab:").user_id("sdk-user").tag("runtime", "rust")).await?;
    let mut properties = Properties::new();
    properties.insert("language".into(), "rust".into());
    events.identify("sdk-user", properties).await?;
    events.group(Group::new("sdk-team").user_id("sdk-user").property("name", "SDK Team")).await?;
    events.subscribe(Subscribe::new("sdk-user@example.com").name("SDK User").property("runtime", "rust").source("/sdk")).await?;
    events.insight_track("Rust runs", 0, None).await?;
    events.insight_increment("Rust runs", 1, None).await?;
    match events.track(Track::new("Not Valid", "x")).await {
        Err(Error::Refused { status, message }) => println!("refused as expected ({status}): {message}"),
        other => panic!("expected a refusal, got {other:?}"),
    }
    println!("rust ok");
    Ok(())
}
