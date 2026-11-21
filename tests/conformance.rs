//! The shared cases in conformance.json: every call must put exactly that method, path and body on the wire.

use snekot_events::{Client, Group, Parser, Properties, Request, Subscribe, Track, Value};
use serde_json::Value as Json;

fn value(json: &Json) -> Value {
    match json {
        Json::String(text) => Value::Text(text.clone()),
        Json::Bool(flag) => Value::Flag(*flag),
        Json::Number(number) if number.is_i64() => Value::Integer(number.as_i64().unwrap()),
        Json::Number(number) => Value::Number(number.as_f64().unwrap()),
        other => panic!("not a tag value: {other}"),
    }
}

fn properties(json: Option<&Json>) -> Properties {
    json.and_then(Json::as_object).map(|fields| fields.iter().map(|(key, entry)| (key.clone(), value(entry))).collect()).unwrap_or_default()
}

fn text<'a>(args: &'a Json, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Json::as_str)
}

fn build(client: &Client, call: &str, args: &Json) -> Request {
    match call {
        "track" => {
            let mut event = Track::new(text(args, "channel").unwrap(), text(args, "event").unwrap());
            if let Some(description) = text(args, "description") {
                event = event.description(description);
            }
            if let Some(icon) = text(args, "icon") {
                event = event.icon(icon);
            }
            if let Some(user) = text(args, "user_id") {
                event = event.user_id(user);
            }
            if let Some(notify) = args.get("notify").and_then(Json::as_bool) {
                event = event.notify(notify);
            }
            if let Some(seconds) = args.get("timestamp").and_then(Json::as_i64) {
                event = event.timestamp(seconds);
            }
            if text(args, "parser") == Some("markdown") {
                event = event.parser(Parser::Markdown);
            }
            for (key, entry) in properties(args.get("tags")) {
                event = event.tag(key, entry);
            }
            client.track_request(&event)
        }
        "identify" => client.identify_request(text(args, "user_id").unwrap(), &properties(args.get("properties"))),
        "group" => {
            let mut group = Group::new(text(args, "group_id").unwrap());
            if let Some(user) = text(args, "user_id") {
                group = group.user_id(user);
            }
            for (key, entry) in properties(args.get("properties")) {
                group = group.property(key, entry);
            }
            client.group_request(&group)
        }
        "subscribe" => {
            let mut signup = Subscribe::new(text(args, "email").unwrap());
            if let Some(name) = text(args, "name") {
                signup = signup.name(name);
            }
            if let Some(source) = text(args, "source") {
                signup = signup.source(source);
            }
            if let Some(notify) = args.get("notify").and_then(Json::as_bool) {
                signup = signup.notify(notify);
            }
            for (key, entry) in properties(args.get("properties")) {
                signup = signup.property(key, entry);
            }
            client.subscribe_request(&signup)
        }
        "insight.track" => client.insight_track_request(text(args, "title").unwrap(), value(&args["value"]), text(args, "icon")),
        "insight.increment" => client.insight_increment_request(text(args, "title").unwrap(), value(&args["value"]), text(args, "icon")),
        other => panic!("unknown call {other}"),
    }
}

#[test]
fn every_case_matches_the_wire_format() {
    let suite: Json = serde_json::from_str(include_str!("conformance.json")).unwrap();
    let client = Client::builder("sk_test", suite["project"].as_str().unwrap()).build();
    let cases = suite["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let request = build(&client, case["call"].as_str().unwrap(), &case["args"]);
        let expect = &case["expect"];
        assert_eq!(request.method, expect["method"].as_str().unwrap(), "{case}");
        assert_eq!(request.path, expect["path"].as_str().unwrap(), "{case}");
        assert_eq!(request.body, expect["body"], "{case}");
    }
}

#[tokio::test]
async fn disabled_sends_nothing() {
    // The endpoint is unroutable, so this only passes if no request is made.
    let client = Client::builder("sk_test", "p").endpoint("http://127.0.0.1:1").disabled(true).build();
    client.track(Track::new("x", "y")).await.unwrap();
}

#[tokio::test]
async fn unreachable_is_reported_after_one_retry() {
    let client = Client::builder("sk_test", "p").endpoint("http://127.0.0.1:1").build();
    let started = std::time::Instant::now();
    let error = client.track(Track::new("x", "y")).await.unwrap_err();
    assert!(matches!(error, snekot_events::Error::Unreachable(_)));
    assert!(started.elapsed() >= std::time::Duration::from_millis(250), "it waited before the second attempt");
}

#[test]
fn debug_output_leaves_the_key_out() {
    let client = Client::builder("sk_secret", "p").build();
    let shown = format!("{client:?}");
    assert!(shown.contains("\"p\"") && !shown.contains("sk_secret"), "{shown}");
}

#[test]
fn an_experiment_puts_its_question_on_the_wire() {
    use snekot_events::Experiment;
    let client = Client::builder("sk_test", "p").build();
    let request = client.experiment_request(&Experiment::new("pricing", ["control", "new"]).user_id("user-1"));
    assert_eq!((request.method, request.path), ("POST", "/v1/experiment"));
    assert_eq!(request.body, serde_json::json!({ "project": "p", "experiment": "pricing", "variants": ["control", "new"], "user_id": "user-1" }));
}

#[tokio::test]
async fn a_disabled_experiment_is_its_first_variant() {
    use snekot_events::Experiment;
    let client = Client::builder("sk_test", "p").endpoint("http://127.0.0.1:1").disabled(true).build();
    assert_eq!(client.experiment(Experiment::new("pricing", ["control", "new"])).await.unwrap(), "control");
}

#[cfg(feature = "blocking")]
#[test]
fn the_blocking_client_reports_an_unreachable_server() {
    use snekot_events::{blocking, Error, Track};
    let client = blocking::Client::new(Client::builder("sk_test", "p").endpoint("http://127.0.0.1:1"));
    assert!(matches!(client.track(Track::new("x", "y")), Err(Error::Unreachable(_))));
    let quiet = blocking::Client::new(Client::builder("sk_test", "p").disabled(true));
    assert!(quiet.track(Track::new("x", "y")).is_ok());
}
