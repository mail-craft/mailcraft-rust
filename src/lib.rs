//! Official Rust SDK for the [MailCraft](https://mailcraft.host) email API.
//!
//! ```no_run
//! use mailcraft::{MailCraft, SendEmailParams};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let mailcraft = MailCraft::new(std::env::var("MAILCRAFT_API_KEY")?);
//!
//! mailcraft.emails().send(SendEmailParams {
//!     from: "hello@yourdomain.com".into(),
//!     to: vec!["person@example.com".into()],
//!     subject: "Welcome!".into(),
//!     html: Some("<p>Thanks for signing up.</p>".into()),
//!     ..Default::default()
//! }).await?;
//! # Ok(())
//! # }
//! ```

mod error;
pub mod params;
pub mod resources;

use std::time::Duration;

use reqwest::{Method, RequestBuilder};
use serde::Serialize;
use serde_json::Value;

pub use error::{Error, Result};
pub use params::*;
use resources::*;

/// The SDK version, sent in the User-Agent header.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The MailCraft API's base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.mailcraft.host/v1";

/// The MailCraft API client. Cheap to clone; clones share a connection pool.
#[derive(Debug, Clone)]
pub struct MailCraft {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl MailCraft {
    /// Creates a client. Create an API key under Settings > API keys.
    ///
    /// # Panics
    ///
    /// If `api_key` is empty.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Starts a client with custom options (base URL, timeout, HTTP client).
    pub fn builder(api_key: impl Into<String>) -> MailCraftBuilder {
        MailCraftBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            http: None,
        }
    }

    pub fn emails(&self) -> Emails<'_> {
        Emails(self)
    }

    pub fn domains(&self) -> Domains<'_> {
        Domains(self)
    }

    pub fn senders(&self) -> Senders<'_> {
        Senders(self)
    }

    pub fn contacts(&self) -> Contacts<'_> {
        Contacts(self)
    }

    pub fn lists(&self) -> Lists<'_> {
        Lists(self)
    }

    pub fn segments(&self) -> Segments<'_> {
        Segments(self)
    }

    pub fn properties(&self) -> Properties<'_> {
        Properties(self)
    }

    pub fn templates(&self) -> Templates<'_> {
        Templates(self)
    }

    pub fn template_folders(&self) -> TemplateFolders<'_> {
        TemplateFolders(self)
    }

    pub fn campaigns(&self) -> Campaigns<'_> {
        Campaigns(self)
    }

    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks(self)
    }

    pub fn suppressions(&self) -> Suppressions<'_> {
        Suppressions(self)
    }

    pub fn metrics(&self) -> Metrics<'_> {
        Metrics(self)
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.http
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key)
            .header("Accept", "application/json")
            .header("User-Agent", format!("mailcraft-rust/{VERSION}"))
    }

    async fn send(&self, request: RequestBuilder) -> Result<Value> {
        let response = request.send().await?;
        let status = response.status().as_u16();
        let body = response.text().await?;

        if status >= 400 {
            return Err(Error::from_response(status, &body));
        }

        if body.trim().is_empty() {
            return Ok(Value::Null);
        }

        Ok(serde_json::from_str(&body).unwrap_or(Value::Null))
    }

    pub(crate) async fn get(&self, path: &str, query: &[(&str, String)]) -> Result<Value> {
        self.send(self.request(Method::GET, path).query(query))
            .await
    }

    pub(crate) async fn post<B: Serialize + ?Sized>(
        &self,
        path: &str,
        body: Option<&B>,
    ) -> Result<Value> {
        let request = self.request(Method::POST, path);
        let request = match body {
            Some(body) => request.json(body),
            None => request,
        };
        self.send(request).await
    }

    pub(crate) async fn patch<B: Serialize + ?Sized>(&self, path: &str, body: &B) -> Result<Value> {
        self.send(self.request(Method::PATCH, path).json(body))
            .await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        self.send(self.request(Method::DELETE, path))
            .await
            .map(|_| ())
    }
}

/// Builds a [`MailCraft`] client with custom options.
#[derive(Debug)]
pub struct MailCraftBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
    http: Option<reqwest::Client>,
}

impl MailCraftBuilder {
    /// Overrides the API base URL (staging, self-hosted).
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Request timeout. Defaults to 30 seconds. Ignored when a custom HTTP client is given.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Uses a pre-built reqwest client (proxies, custom TLS, shared pool).
    pub fn http_client(mut self, http: reqwest::Client) -> Self {
        self.http = Some(http);
        self
    }

    /// # Panics
    ///
    /// If the API key is empty.
    pub fn build(self) -> MailCraft {
        assert!(
            !self.api_key.is_empty(),
            "An API key is required. Find yours under Settings > API Keys."
        );

        let http = self.http.unwrap_or_else(|| {
            reqwest::Client::builder()
                .timeout(self.timeout)
                .build()
                .expect("failed to build the HTTP client")
        });

        MailCraft {
            api_key: self.api_key,
            base_url: self.base_url,
            http,
        }
    }
}
