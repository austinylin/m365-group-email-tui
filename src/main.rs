mod auth;
mod graph;
mod tui;

use anyhow::anyhow;
use anyhow::Result;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let token = auth::get_access_token().await?;
    let args: Vec<String> = env::args().collect();
    let group_email = args
        .get(1)
        .cloned()
        .ok_or_else(|| anyhow!("Usage: <program> <GROUP_EMAIL>"))?;

    // First get the group ID from the email
    let group = graph::get_group_by_email(&token, &group_email).await?;
    println!(
        "Found group: {}",
        group.display_name.unwrap_or_else(|| "Unknown".to_string())
    );

    let messages = graph::fetch_messages(&token, &group.id).await?;

    // Create and run the TUI
    let mut app = tui::App::new(token, group.id, messages);
    app.run().await?;

    Ok(())
}
