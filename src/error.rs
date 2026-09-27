use std::collections::HashMap;
use std::fmt;

/// Everything a MailCraft call can fail with.
#[derive(Debug)]
pub enum Error {
    /// The API returned a non-2xx response.
    ///
    /// Covers both error shapes the API returns: `{"error": {"type", "message"}}`
    /// for business-rule failures (plan limits, suppressed recipients, ...), and
    /// `{"message", "errors": {"field": [...]}}` for validation failures (422).
    Api {
        /// The HTTP status, e.g. 402 or 422.
        status: u16,
        /// What went wrong.
        message: String,
        /// The business-rule error type, e.g. `plan_limit_reached`. `None` for validation errors.
        error_type: Option<String>,
        /// Field errors for 422 responses.
        errors: HashMap<String, Vec<String>>,
    },
    /// The request couldn't be sent or the response couldn't be read.
    Http(reqwest::Error),
}

impl Error {
    pub(crate) fn from_response(status: u16, body: &str) -> Self {
        let value: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let fallback = || format!("Request failed with status {status}");

        if let Some(error) = value.get("error").filter(|e| e.is_object()) {
            return Error::Api {
                status,
                message: error["message"]
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(fallback),
                error_type: error["type"].as_str().map(String::from),
                errors: HashMap::new(),
            };
        }

        Error::Api {
            status,
            message: value["message"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(fallback),
            error_type: None,
            errors: serde_json::from_value(value["errors"].clone()).unwrap_or_default(),
        }
    }

    /// The HTTP status for API errors.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api { status, .. } => Some(*status),
            Error::Http(error) => error.status().map(|s| s.as_u16()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api {
                status,
                message,
                error_type: Some(kind),
                ..
            } => {
                write!(f, "MailCraft API error {status} ({kind}): {message}")
            }
            Error::Api {
                status, message, ..
            } => write!(f, "MailCraft API error {status}: {message}"),
            Error::Http(error) => write!(f, "MailCraft request failed: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(error) => Some(error),
            Error::Api { .. } => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Http(error)
    }
}

/// Shorthand for results from this crate.
pub type Result<T> = std::result::Result<T, Error>;
