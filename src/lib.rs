//! Post product events, identify users and set insights.
//!
//! ```no_run
//! use snekot_events::{Client, Track};
//!
//! # async fn run() -> Result<(), snekot_events::Error> {
//! let events = Client::builder("sk_...", "acme-store").build();
//! events
//!     .track(Track::new("payments", "New subscription").icon(":moneybag:").notify(true).tag("plan", "pro"))
//!     .await?;
//! events.insight_increment("Orders", 1, None).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;
use serde_json::{json, Map, Value as Json};

/// Where the API lives until a client is told otherwise.
pub const DEFAULT_ENDPOINT: &str = "https://api.snekot.co";

/// A tag, property or insight value: text, a number or a flag.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Value {
    Text(String),
    Integer(i64),
    Number(f64),
    Flag(bool),
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_owned())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value.into())
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Flag(value)
    }
}

/// Keys are lowercase letters, digits and dashes; the server refuses anything else.
pub type Properties = BTreeMap<String, Value>;

/// How a description is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Parser {
    /// Bold, italic, inline code and links.
    Markdown,
    Text,
}

/// An event to post. Only the channel and the name are required.
#[derive(Debug, Clone, Serialize)]
pub struct Track {
    channel: String,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notify: Option<bool>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    tags: Properties,
    #[serde(skip_serializing_if = "Option::is_none")]
    parser: Option<Parser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
}

impl Track {
    pub fn new(channel: impl Into<String>, event: impl Into<String>) -> Self {
        Self { channel: channel.into(), event: event.into(), description: None, user_id: None, icon: None, notify: None, tags: Properties::new(), parser: None, timestamp: None }
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    /// One emoji, or a shortcode such as `:moneybag:`.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Push this event to the phones of everyone on the project.
    pub fn notify(mut self, notify: bool) -> Self {
        self.notify = Some(notify);
        self
    }
    pub fn tag(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
    pub fn parser(mut self, parser: Parser) -> Self {
        self.parser = Some(parser);
        self
    }
    /// Unix seconds. A past moment makes this a backfill: stored at that time, never pushed to a phone.
    pub fn timestamp(mut self, seconds: i64) -> Self {
        self.timestamp = Some(seconds);
        self
    }
}

/// A group such as a company or a team.
#[derive(Debug, Clone, Serialize)]
pub struct Group {
    group_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    properties: Properties,
}

impl Group {
    pub fn new(group_id: impl Into<String>) -> Self {
        Self { group_id: group_id.into(), user_id: None, properties: Properties::new() }
    }
    /// Puts this user in the group.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    pub fn property(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
}

/// A newsletter signup. Only the address is required.
#[derive(Debug, Clone, Serialize)]
pub struct Subscribe {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    properties: Properties,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
}

impl Subscribe {
    /// Capitals and spaces around the address do not matter.
    pub fn new(email: impl Into<String>) -> Self {
        Self { email: email.into(), name: None, properties: Properties::new(), source: None, notify: None, language: None, timezone: None }
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn property(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
    /// The page the signup came from, such as `/blog/launch`.
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    /// Push the signup to the phones of everyone on the project.
    pub fn notify(mut self, notify: bool) -> Self {
        self.notify = Some(notify);
        self
    }
    /// The person's language, as a tag such as `sv` or `sv-SE`.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
    /// The person's time zone, as its IANA name, such as `Europe/Stockholm`.
    pub fn timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

/// A question about an A/B test: which variant does this person get?
#[derive(Debug, Clone, Serialize)]
pub struct Experiment {
    experiment: String,
    variants: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
}

impl Experiment {
    /// Two to eight variants. The first is the one the others are measured against; keep the order the same everywhere.
    pub fn new<V: Into<String>>(name: impl Into<String>, variants: impl IntoIterator<Item = V>) -> Self {
        Self { experiment: name.into(), variants: variants.into_iter().map(Into::into).collect(), user_id: None, ip: None, user_agent: None }
    }
    /// The person, when they are known.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    /// For someone with no id: their address and user agent, which tie them to their page views and are not stored.
    pub fn visitor(mut self, ip: impl Into<String>, user_agent: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self.user_agent = Some(user_agent.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The server understood the request and said no. `message` is its own sentence for what to fix. Never retried.
    #[error("refused ({status}): {message}")]
    Refused { status: u16, message: String },
    /// The server errored twice in a row.
    #[error("the server answered {status}")]
    Server { status: u16 },
    /// The request could not be sent or timed out, twice in a row.
    #[error("the server could not be reached: {0}")]
    Unreachable(#[from] reqwest::Error),
}

/// A request as it goes on the wire, before the key is attached.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub method: &'static str,
    pub path: &'static str,
    pub body: Json,
}

/// Made by [`Client::builder`].
pub struct ClientBuilder {
    token: String,
    project: String,
    endpoint: String,
    timeout: Duration,
    disabled: bool,
}

impl ClientBuilder {
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into().trim_end_matches('/').to_owned();
        self
    }
    /// How long one attempt may take; 10 seconds by default.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    /// Makes every call succeed without sending anything, for tests and local development.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn build(self) -> Client {
        let http = reqwest::Client::builder().timeout(self.timeout).build().expect("the TLS backend is compiled in");
        Client { http, token: self.token, project: self.project, endpoint: self.endpoint, disabled: self.disabled }
    }
}

/// Cheap to clone; clones share one connection pool.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    token: String,
    project: String,
    endpoint: String,
    disabled: bool,
}

/// Shows where the client sends and for which project, and never the key.
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client").field("project", &self.project).field("endpoint", &self.endpoint).field("disabled", &self.disabled).finish_non_exhaustive()
    }
}

impl Client {
    pub fn builder(token: impl Into<String>, project: impl Into<String>) -> ClientBuilder {
        ClientBuilder { token: token.into(), project: project.into(), endpoint: DEFAULT_ENDPOINT.to_owned(), timeout: Duration::from_secs(10), disabled: false }
    }

    pub fn project(&self) -> &str {
        &self.project
    }

    /// Posts an event to a channel.
    pub async fn track(&self, event: Track) -> Result<(), Error> {
        self.send(self.track_request(&event)).await
    }

    /// Sets properties on a user; keys sent again replace their old values.
    pub async fn identify(&self, user_id: &str, properties: Properties) -> Result<(), Error> {
        self.send(self.identify_request(user_id, &properties)).await
    }

    /// Describes a group, and puts its `user_id` in it when one is set.
    pub async fn group(&self, group: Group) -> Result<(), Error> {
        self.send(self.group_request(&group)).await
    }

    /// Puts an address on the project's newsletter list. Signing it up again changes nothing.
    pub async fn subscribe(&self, signup: Subscribe) -> Result<(), Error> {
        self.send(self.subscribe_request(&signup)).await
    }

    /// Sets an insight to a value.
    pub async fn insight_track(&self, title: &str, value: impl Into<Value>, icon: Option<&str>) -> Result<(), Error> {
        self.send(self.insight_track_request(title, value.into(), icon)).await
    }

    /// Adds to an insight's number, starting from 0 if it has none. Negative subtracts.
    pub async fn insight_increment(&self, title: &str, by: impl Into<Value>, icon: Option<&str>) -> Result<(), Error> {
        self.send(self.insight_increment_request(title, by.into(), icon)).await
    }

    /// Which variant of an A/B test a person gets. The same person always gets the same one.
    /// With the client disabled it is the first variant and nothing is sent.
    pub async fn experiment(&self, experiment: Experiment) -> Result<String, Error> {
        let first = experiment.variants.first().cloned().unwrap_or_default();
        if self.disabled {
            return Ok(first);
        }
        let answer = self.deliver(self.experiment_request(&experiment)).await?;
        Ok(answer.get("variant").and_then(Json::as_str).map(str::to_owned).unwrap_or(first))
    }

    // The requests are built apart from sending them, so the conformance tests can check the wire format without a server.

    pub fn track_request(&self, event: &Track) -> Request {
        self.request("POST", "/v1/log", event)
    }
    pub fn identify_request(&self, user_id: &str, properties: &Properties) -> Request {
        self.request("POST", "/v1/identify", &json!({ "user_id": user_id, "properties": properties }))
    }
    pub fn group_request(&self, group: &Group) -> Request {
        self.request("POST", "/v1/group", group)
    }
    pub fn subscribe_request(&self, signup: &Subscribe) -> Request {
        self.request("POST", "/v1/subscribe", signup)
    }
    pub fn experiment_request(&self, experiment: &Experiment) -> Request {
        self.request("POST", "/v1/experiment", experiment)
    }
    pub fn insight_track_request(&self, title: &str, value: Value, icon: Option<&str>) -> Request {
        self.request("POST", "/v1/insight", &with_icon(json!({ "title": title, "value": value }), icon))
    }
    pub fn insight_increment_request(&self, title: &str, by: Value, icon: Option<&str>) -> Request {
        self.request("PATCH", "/v1/insight", &with_icon(json!({ "title": title, "value": { "$inc": by } }), icon))
    }

    fn request(&self, method: &'static str, path: &'static str, body: &impl Serialize) -> Request {
        let mut body = match serde_json::to_value(body) {
            Ok(Json::Object(fields)) => fields,
            _ => Map::new(),
        };
        body.insert("project".to_owned(), Json::String(self.project.clone()));
        Request { method, path, body: Json::Object(body) }
    }

    async fn send(&self, request: Request) -> Result<(), Error> {
        if self.disabled {
            return Ok(());
        }
        self.deliver(request).await.map(|_| ())
    }

    /// Sends one request, with one retry, and gives back what the server answered with.
    async fn deliver(&self, request: Request) -> Result<Json, Error> {
        let method = if request.method == "PATCH" { reqwest::Method::PATCH } else { reqwest::Method::POST };
        let url = format!("{}{}", self.endpoint, request.path);
        // One retry, and only where trying again can help: the network failed or the server did. A refusal is final.
        let mut retried = false;
        loop {
            let outcome = self.http.request(method.clone(), &url).bearer_auth(&self.token).json(&request.body).send().await;
            let failure = match outcome {
                Ok(response) if response.status().is_success() => return Ok(response.json().await.unwrap_or(Json::Null)),
                Ok(response) if response.status().is_server_error() => Error::Server { status: response.status().as_u16() },
                Ok(response) => {
                    let status = response.status().as_u16();
                    let body: Json = response.json().await.unwrap_or(Json::Null);
                    let message = body.get("error").and_then(Json::as_str).map(str::to_owned).unwrap_or_else(|| format!("the server answered {status}"));
                    return Err(Error::Refused { status, message });
                }
                Err(error) => Error::Unreachable(error),
            };
            if retried {
                return Err(failure);
            }
            retried = true;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
}

fn with_icon(mut body: Json, icon: Option<&str>) -> Json {
    if let (Some(icon), Some(fields)) = (icon, body.as_object_mut()) {
        fields.insert("icon".to_owned(), Json::String(icon.to_owned()));
    }
    body
}

/// The same client for programs that are not async. Needs the `blocking` feature.
///
/// ```no_run
/// use snekot_events::{blocking::Client, Track};
///
/// # fn run() -> Result<(), snekot_events::Error> {
/// let events = Client::new(snekot_events::Client::builder("sk_...", "acme-store"));
/// events.track(Track::new("payments", "New subscription").notify(true))?;
/// # Ok(())
/// # }
/// ```
///
/// Each call waits for the server's answer on the thread that made it. Do not call it from inside an async
/// runtime, where it panics; use [`crate::Client`] there.
#[cfg(feature = "blocking")]
pub mod blocking {
    use super::{ClientBuilder, Error, Experiment, Group, Properties, Subscribe, Track, Value};

    /// Cheap to share behind an `Arc`; every call goes through one connection pool.
    #[derive(Debug)]
    pub struct Client {
        inner: super::Client,
        runtime: tokio::runtime::Runtime,
    }

    impl Client {
        /// Builds from the same builder as the async client: `Client::new(snekot_events::Client::builder(token, project))`.
        pub fn new(builder: ClientBuilder) -> Self {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime can be built");
            Self { inner: builder.build(), runtime }
        }
        pub fn project(&self) -> &str {
            self.inner.project()
        }
        /// Posts an event to a channel.
        pub fn track(&self, event: Track) -> Result<(), Error> {
            self.runtime.block_on(self.inner.track(event))
        }
        /// Sets properties on a user; keys sent again replace their old values.
        pub fn identify(&self, user_id: &str, properties: Properties) -> Result<(), Error> {
            self.runtime.block_on(self.inner.identify(user_id, properties))
        }
        /// Describes a group, and puts its `user_id` in it when one is set.
        pub fn group(&self, group: Group) -> Result<(), Error> {
            self.runtime.block_on(self.inner.group(group))
        }
        /// Puts an address on the project's newsletter list. Signing it up again changes nothing.
        pub fn subscribe(&self, signup: Subscribe) -> Result<(), Error> {
            self.runtime.block_on(self.inner.subscribe(signup))
        }
        /// Which variant of an A/B test a person gets.
        pub fn experiment(&self, experiment: Experiment) -> Result<String, Error> {
            self.runtime.block_on(self.inner.experiment(experiment))
        }
        /// Sets an insight to a value.
        pub fn insight_track(&self, title: &str, value: impl Into<Value>, icon: Option<&str>) -> Result<(), Error> {
            self.runtime.block_on(self.inner.insight_track(title, value, icon))
        }
        /// Adds to an insight's number, starting from 0 if it has none. Negative subtracts.
        pub fn insight_increment(&self, title: &str, by: impl Into<Value>, icon: Option<&str>) -> Result<(), Error> {
            self.runtime.block_on(self.inner.insight_increment(title, by, icon))
        }
    }
}
