# snekot-events

The [Snekot](https://snekot.co) SDK for Rust. Post product events, get them as push notifications, identify users and set insights.

Async, on `reqwest` with rustls.

```bash
cargo add snekot-events
```

It needs a Tokio runtime.

## Use

```rust
use snekot_events::{Client, Subscribe, Track};

let events = Client::builder(token, "acme-store").build();

events
    .track(Track::new("payments", "New subscription").icon("💰").notify(true).tag("plan", "pro"))
    .await?;
events.insight_increment("Orders", 1, None).await?;
events.subscribe(Subscribe::new("jane@example.com").name("Jane Park").language("sv-SE").timezone("Europe/Stockholm")).await?;
```

`experiment` asks which variant of an [A/B test](https://snekot.co/docs/experiments) a person gets:

```rust
use snekot_events::Experiment;

let variant = events.experiment(Experiment::new("pricing-page", ["control", "annual-first"]).user_id("user-123")).await?;
```

`flag` asks whether a [feature flag](https://snekot.co/docs/flags) is on for a person:

```rust
use snekot_events::Flag;

let on = events.flag(Flag::new("new-checkout").user_id("user-123")).await?;
```

A key that has no flag is off. When the answer cannot be had, it is an `Error` like every other call.

Use a private key (`sk_…`) on a server. Keys are made in the app under Setup; see [Keys](https://snekot.co/docs/keys).

### Behaviour

- One attempt may take 10 seconds.
- A network failure or a server error is retried once, after a quarter of a second.
- A refusal (a 4xx, such as a bad channel name) is never retried.
- Errors are `Error::Refused { status, message }`, with the server's own sentence for what to fix, `Error::Server { status }` and `Error::Unreachable`.
- `.disabled(true)` on the builder makes every call succeed without sending anything, for tests and local runs.
- The endpoint defaults to `https://api.snekot.co`; `.endpoint(…)` on the builder changes it.

## Without async

Turn on the `blocking` feature (add `--features blocking` to the `cargo add` line) for a client whose calls wait for the answer:

```rust
use snekot_events::{blocking, Client, Track};

let events = blocking::Client::new(Client::builder(token, "acme-store"));
events.track(Track::new("payments", "New subscription").notify(true))?;
```

Do not call it from inside an async runtime; use the async client there.

## Develop

```bash
cargo test --all-features
```

Publishing a release here publishes its version to crates.io; the release's tag must be the version in `Cargo.toml`.

`tests/conformance.json` is the wire contract every Snekot SDK is tested against. `cargo run --example smoke` runs every method against the real API; it needs `SNEKOT_TOKEN` and `SNEKOT_PROJECT`.

## Licence

MIT
