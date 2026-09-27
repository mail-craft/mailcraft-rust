# mailcraft-rust

Official Rust SDK for the [MailCraft](https://mailcraft.host) email API: transactional email, contacts, lists, segments, templates, campaigns, webhooks and more.

Async, built on [reqwest](https://docs.rs/reqwest) with rustls (no OpenSSL needed). Works with any tokio-based runtime.

## Install

```bash
cargo add mailcraft
```

## Quick start

```rust
use mailcraft::{MailCraft, SendEmailParams};

let mailcraft = MailCraft::new(std::env::var("MAILCRAFT_API_KEY")?);

let email = mailcraft.emails().send(SendEmailParams {
    from: "hello@yourdomain.com".into(),
    to: vec!["person@example.com".into()],
    subject: "Welcome!".into(),
    html: Some("<p>Thanks for signing up.</p>".into()),
    ..Default::default()
}).await?;

println!("{}", email["data"]["id"]);
```

Create an API key under **Settings → API keys** in your MailCraft dashboard. `MailCraft` is cheap to clone; clones share one connection pool.

## Resources

Every MailCraft SDK has the same resources and methods:

| Resource | Methods |
| --- | --- |
| `emails()` | `send`, `list`, `get`, `validate` |
| `domains()` | `create`, `list`, `get`, `verify`, `delete` |
| `senders()` | `create`, `list`, `get`, `delete` |
| `contacts()` | `upsert`, `list`, `get`, `delete`, `unsubscribe`, `add_to_lists`, `lists`, `remove_from_list` |
| `lists()` | `create`, `list`, `get`, `delete` |
| `segments()` | `create`, `list`, `get`, `delete` |
| `properties()` | `create`, `list`, `delete` |
| `templates()` | `create`, `list`, `get`, `update`, `delete` |
| `template_folders()` | `create`, `list`, `delete` |
| `campaigns()` | `create`, `list`, `get`, `send`, `delete` |
| `webhooks()` | `create`, `list`, `delete` |
| `suppressions()` | `add`, `list`, `delete` |
| `metrics()` | `get`, `reputation` |

Requests take typed params (`SendEmailParams`, `UpsertContactParams`, ...) that implement `Default`; `None` fields are left out of the request. Responses are the API's JSON as a `serde_json::Value`. See the [API reference](https://docs.mailcraft.host/api-reference) for every field.

## Errors

Every method returns `mailcraft::Result<T>`. Non-2xx responses are `Error::Api`:

```rust
use mailcraft::{CreateDomainParams, Error};

match mailcraft.domains().create(CreateDomainParams { name: "acme.com".into(), ..Default::default() }).await {
    Err(Error::Api { status, message, error_type, errors }) => {
        // status: e.g. 402
        // error_type: e.g. Some("plan_limit_reached") for business-rule errors
        // errors: field errors for 422 validation failures
    }
    Err(Error::Http(error)) => { /* network or timeout */ }
    Ok(domain) => { /* ... */ }
}
```

## Options

```rust
use std::time::Duration;

let mailcraft = MailCraft::builder(api_key)
    .base_url("https://api.mailcraft.host/v1") // override for staging or self-hosting
    .timeout(Duration::from_secs(10))         // default 30s
    .build();
```

Pass your own `reqwest::Client` with `.http_client(...)` for proxies or a shared pool.

## Development

```bash
cargo test
```

The tests run the real client against a local TCP server, so every request goes over HTTP exactly as a user's would.

## License

MIT
