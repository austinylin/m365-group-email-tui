use anyhow::Result;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, ClientId, ClientSecret, Scope,
    TokenResponse, TokenUrl,
};
use std::env;

pub async fn get_access_token() -> Result<String> {
    let client_id = ClientId::new(env::var("CLIENT_ID")?);
    let client_secret = ClientSecret::new(env::var("CLIENT_SECRET")?);
    let tenant_id = env::var("TENANT_ID")?;

    let auth_url = AuthUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
        tenant_id
    ))?;
    let token_url = TokenUrl::new(format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id
    ))?;

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url));

    let token_result = client
        .exchange_client_credentials()
        .add_scope(Scope::new(
            "https://graph.microsoft.com/.default".to_string(),
        ))
        .request_async(async_http_client)
        .await?;

    Ok(token_result.access_token().secret().to_string())
}
