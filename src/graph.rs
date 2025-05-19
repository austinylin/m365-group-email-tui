use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Post {
    pub _id: String,
    #[serde(rename = "createdDateTime")]
    pub _created: Option<String>,
    #[serde(rename = "hasAttachments")]
    pub _has_attachments: bool,
    #[serde(rename = "from")]
    pub _from: Option<Sender>,
    pub _body: EmailBody,
    pub _subject: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailBody {
    #[serde(rename = "contentType")]
    pub _content_type: String,
    pub _content: String,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    #[serde(rename = "emailAddress")]
    pub _email_address: Option<EmailAddress>,
}

#[derive(Debug, Deserialize)]
pub struct EmailAddress {
    pub _name: Option<String>,
    pub _address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConversationResponse {
    _value: Vec<Conversation>,
}

#[derive(Debug, Deserialize)]
pub struct Conversation {
    pub _id: String,
    pub _topic: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThreadResponse {
    _value: Vec<Thread>,
}

#[derive(Debug, Deserialize)]
pub struct Thread {
    pub _id: String,
}

#[derive(Debug, Deserialize)]
struct PostResponse {
    _value: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct GroupResponse {
    value: Vec<Group>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    #[serde(rename = "mail")]
    pub _email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub topic: String,
    #[serde(rename = "preview")]
    pub preview: String,
}

#[derive(Debug, Deserialize)]
struct ConversationListResponse {
    value: Vec<ConversationSummary>,
}

pub async fn fetch_messages(token: &str, group_id: &str) -> Result<Vec<ConversationSummary>> {
    let url = format!(
        "https://graph.microsoft.com/v1.0/groups/{}/conversations?$top=20",
        group_id
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<ConversationListResponse>()
        .await?;

    Ok(resp.value)
}

pub async fn get_group_by_email(token: &str, email: &str) -> Result<Group> {
    let url = format!(
        "https://graph.microsoft.com/v1.0/groups?$filter=mail eq '{}'",
        email
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<GroupResponse>()
        .await?;

    resp.value
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No group found with email: {}", email))
}