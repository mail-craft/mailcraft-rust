//! One resource per API section. Every method returns the API's JSON as a
//! [`serde_json::Value`], or `()` for deletes.

use crate::params::*;
use crate::{MailCraft, Result};
use serde_json::{json, Value};

fn limit_query(limit: Option<u32>) -> Vec<(&'static str, String)> {
    limit
        .map(|l| vec![("limit", l.to_string())])
        .unwrap_or_default()
}

fn segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// Transactional email.
pub struct Emails<'a>(pub(crate) &'a MailCraft);

impl Emails<'_> {
    pub async fn send(&self, params: SendEmailParams) -> Result<Value> {
        self.0.post("/emails", Some(&params)).await
    }

    pub async fn list(&self, limit: Option<u32>) -> Result<Value> {
        self.0.get("/emails", &limit_query(limit)).await
    }

    pub async fn get(&self, id: &str) -> Result<Value> {
        self.0.get(&format!("/emails/{}", segment(id)), &[]).await
    }

    /// Checks an address's format, MX records and disposable domain, without sending.
    pub async fn validate(&self, email: &str) -> Result<Value> {
        self.0
            .get("/emails/validate", &[("email", email.to_string())])
            .await
    }
}

/// Sending domains.
pub struct Domains<'a>(pub(crate) &'a MailCraft);

impl Domains<'_> {
    pub async fn create(&self, params: CreateDomainParams) -> Result<Value> {
        self.0.post("/domains", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/domains", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/domains/{id}"), &[]).await
    }

    pub async fn verify(&self, id: u64) -> Result<Value> {
        self.0
            .post::<()>(&format!("/domains/{id}/verify"), None)
            .await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/domains/{id}")).await
    }
}

/// Sender identities.
pub struct Senders<'a>(pub(crate) &'a MailCraft);

impl Senders<'_> {
    pub async fn create(&self, params: CreateSenderParams) -> Result<Value> {
        self.0.post("/senders", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/senders", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/senders/{id}"), &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/senders/{id}")).await
    }
}

/// Contacts and their list memberships.
pub struct Contacts<'a>(pub(crate) &'a MailCraft);

impl Contacts<'_> {
    pub async fn upsert(&self, params: UpsertContactParams) -> Result<Value> {
        self.0.post("/contacts", Some(&params)).await
    }

    pub async fn list(&self, limit: Option<u32>) -> Result<Value> {
        self.0.get("/contacts", &limit_query(limit)).await
    }

    pub async fn get(&self, id: &str) -> Result<Value> {
        self.0.get(&format!("/contacts/{}", segment(id)), &[]).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.0.delete(&format!("/contacts/{}", segment(id))).await
    }

    pub async fn unsubscribe(&self, id: &str) -> Result<Value> {
        self.0
            .post::<()>(&format!("/contacts/{}/unsubscribe", segment(id)), None)
            .await
    }

    pub async fn add_to_lists(&self, id: &str, list_ids: &[u64]) -> Result<Value> {
        let body = json!({ "list_ids": list_ids });
        self.0
            .post(&format!("/contacts/{}/lists", segment(id)), Some(&body))
            .await
    }

    pub async fn lists(&self, id: &str) -> Result<Value> {
        self.0
            .get(&format!("/contacts/{}/lists", segment(id)), &[])
            .await
    }

    pub async fn remove_from_list(&self, id: &str, list_id: u64) -> Result<()> {
        self.0
            .delete(&format!("/contacts/{}/lists/{list_id}", segment(id)))
            .await
    }
}

/// Mailing lists.
pub struct Lists<'a>(pub(crate) &'a MailCraft);

impl Lists<'_> {
    pub async fn create(&self, params: CreateListParams) -> Result<Value> {
        self.0.post("/lists", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/lists", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/lists/{id}"), &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/lists/{id}")).await
    }
}

/// Segments.
pub struct Segments<'a>(pub(crate) &'a MailCraft);

impl Segments<'_> {
    pub async fn create(&self, params: CreateSegmentParams) -> Result<Value> {
        self.0.post("/segments", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/segments", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/segments/{id}"), &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/segments/{id}")).await
    }
}

/// Custom contact properties.
pub struct Properties<'a>(pub(crate) &'a MailCraft);

impl Properties<'_> {
    pub async fn create(&self, params: CreatePropertyParams) -> Result<Value> {
        self.0.post("/properties", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/properties", &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/properties/{id}")).await
    }
}

/// Versioned templates.
pub struct Templates<'a>(pub(crate) &'a MailCraft);

impl Templates<'_> {
    pub async fn create(&self, params: TemplateParams) -> Result<Value> {
        self.0.post("/templates", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/templates", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/templates/{id}"), &[]).await
    }

    /// Each save becomes a new version. `None` fields are left as they are.
    pub async fn update(&self, id: u64, params: TemplateParams) -> Result<Value> {
        self.0.patch(&format!("/templates/{id}"), &params).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/templates/{id}")).await
    }
}

/// Template folders.
pub struct TemplateFolders<'a>(pub(crate) &'a MailCraft);

impl TemplateFolders<'_> {
    pub async fn create(&self, name: &str) -> Result<Value> {
        self.0
            .post("/template-folders", Some(&json!({ "name": name })))
            .await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/template-folders", &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/template-folders/{id}")).await
    }
}

/// Marketing campaigns.
pub struct Campaigns<'a>(pub(crate) &'a MailCraft);

impl Campaigns<'_> {
    pub async fn create(&self, params: CreateCampaignParams) -> Result<Value> {
        self.0.post("/campaigns", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/campaigns", &[]).await
    }

    pub async fn get(&self, id: u64) -> Result<Value> {
        self.0.get(&format!("/campaigns/{id}"), &[]).await
    }

    pub async fn send(&self, id: u64) -> Result<Value> {
        self.0
            .post::<()>(&format!("/campaigns/{id}/send"), None)
            .await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/campaigns/{id}")).await
    }
}

/// Webhook endpoints.
pub struct Webhooks<'a>(pub(crate) &'a MailCraft);

impl Webhooks<'_> {
    pub async fn create(&self, params: CreateWebhookParams) -> Result<Value> {
        self.0.post("/webhooks", Some(&params)).await
    }

    pub async fn list(&self) -> Result<Value> {
        self.0.get("/webhooks", &[]).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/webhooks/{id}")).await
    }
}

/// The suppression list.
pub struct Suppressions<'a>(pub(crate) &'a MailCraft);

impl Suppressions<'_> {
    pub async fn add(&self, params: AddSuppressionParams) -> Result<Value> {
        self.0.post("/suppressions", Some(&params)).await
    }

    pub async fn list(&self, limit: Option<u32>) -> Result<Value> {
        self.0.get("/suppressions", &limit_query(limit)).await
    }

    pub async fn delete(&self, id: u64) -> Result<()> {
        self.0.delete(&format!("/suppressions/{id}")).await
    }
}

/// Sending metrics and reputation.
pub struct Metrics<'a>(pub(crate) &'a MailCraft);

impl Metrics<'_> {
    /// Dates are `YYYY-MM-DD`.
    pub async fn get(&self, start_date: Option<&str>, end_date: Option<&str>) -> Result<Value> {
        let mut query = Vec::new();
        if let Some(start) = start_date {
            query.push(("start_date", start.to_string()));
        }
        if let Some(end) = end_date {
            query.push(("end_date", end.to_string()));
        }
        self.0.get("/metrics", &query).await
    }

    pub async fn reputation(&self) -> Result<Value> {
        self.0.get("/reputation", &[]).await
    }
}
