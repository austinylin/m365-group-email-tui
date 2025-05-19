use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;


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
    #[serde(rename = "lastDeliveredDateTime")]
    pub last_delivered: Option<String>,
    #[serde(rename = "uniqueSenders")]
    pub unique_senders: Option<Vec<String>>,
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
