# M365 Group Email TUI (Rust)

A terminal-based TUI application written in Rust to read email messages from a Microsoft 365 group mailbox using Microsoft Graph API with app-only authentication.

## Features

- Authenticates via OAuth2 client credentials flow
- Fetches messages from a Microsoft 365 group mailbox
- Displays messages in a scrollable terminal UI
- Future-ready architecture to support sending emails

## Getting Started

### Prerequisites

- Rust (latest stable)
- Azure AD App Registration with:
  - `Mail.Read`
  - `Group.Read.All`
  - (Future: `Mail.Send`)
  - Admin consent granted

### Setup

1. Clone this repo:
   ```
   git clone https://github.com/austinylin/m365-group-email-tui.git
   cd m365-group-email-tui
   ```

2. Create a `.env` file with the following:
   ```
   CLIENT_ID=your-client-id
   CLIENT_SECRET=your-client-secret
   TENANT_ID=your-tenant-id
   ```

3. Run the app:
   ```
   cargo run <group email address>
   ```

## Dependencies

- `tokio` for async runtime
- `reqwest` for HTTP
- `serde` for JSON deserialization
- `oauth2` for Microsoft Graph auth
- `ratatui` + `crossterm` for the TUI
- `dotenvy` for environment config

## Credits
This code was largely written by Cursor, ChatGPT, and OpenAI Codex.