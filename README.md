# snekot-events

The [Snekot](https://snekot.co) SDK for Rust. Post product events, get them as push notifications, identify users and set insights.

Async, on `reqwest` with rustls.

```bash
cargo add snekot-events --git https://github.com/snekot-co/snekot-rust --tag 0.2.0
```

That builds from this repository's `0.2.0` tag. The crate is not on crates.io yet. It needs a Tokio runtime.

## Use

```rust
use snekot_events::{Client, Subscribe, Track};

let events = Client::builder(token, "acme-store").build();

events
    .track(Track::new("payments", "New subscription").icon("💰").notify(true).tag("plan", "pro"))
    .await?;
events.insight_increment("Orders", 1, None).await?;
events.subscribe(Subscribe::new("jane@example.com").name("Jane Park")).await?;
```

Use a private key (`sk_…`) on a server. Keys are made in the app under Setup; see [Keys](https://snekot.co/docs/keys).

### Behaviour

- One attempt may take 10 seconds.
- A network failure or a server error is retried once, after a quarter of a second.
- A refusal (a 4xx, such as a bad channel name) is never retried.
- Errors are `Error::Refused { status, message }`, with the server's own sentence for what to fix, `Error::Server { status }` and `Error::Unreachable`.
- `.disabled(true)` on the builder makes every call succeed without sending anything, for tests and local runs.
- The endpoint defaults to `https://api.snekot.co`; `.endpoint(…)` on the builder changes it.

## Develop

```bash
cargo test
```

`tests/conformance.json` is the wire contract every Snekot SDK is tested against. `cargo run --example smoke` runs every method against the real API; it needs `SNEKOT_TOKEN` and `SNEKOT_PROJECT`.

## Licence

MIT
