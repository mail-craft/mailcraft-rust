//! Request parameters. `None` fields are left out of the request.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// A transactional email to send.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SendEmailParams {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
}

/// Registers a sending domain.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDomainParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

/// Creates a sender identity on a verified domain.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateSenderParams {
    pub domain_id: u64,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// Creates a contact, or updates it if one exists for this email.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpsertContactParams {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// `subscribed`, `unsubscribed` or `pending`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Value>>,
}

/// Creates a list. `list_type` is `static` or `dynamic` (with `segment_id`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateListParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub list_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<u64>,
}

/// Creates a segment from filter conditions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateSegmentParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub filters: SegmentFilters,
}

/// Matches contacts on all (`and`) or any (`or`) of its conditions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SegmentFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    pub conditions: Vec<SegmentCondition>,
}

/// Compares a contact field. `operator` is eq, neq, gt, gte, lt, lte or contains.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SegmentCondition {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// Creates a custom contact property. `property_type` is text, number, boolean, date or list.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreatePropertyParams {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
}

/// Creates or updates a template. `name` and `subject` are required to create.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TemplateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_folder_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Value>>,
}

/// Creates a draft campaign for a list or a segment.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateCampaignParams {
    pub name: String,
    pub subject: String,
    pub template_id: u64,
    pub sender_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<u64>,
}

/// Subscribes a URL to events.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateWebhookParams {
    pub url: String,
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Stops sending to an address.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AddSuppressionParams {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
