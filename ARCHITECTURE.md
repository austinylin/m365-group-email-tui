# Architecture - M365 Group Email TUI (Rust)

## Overview

This application is a terminal-based TUI (Text User Interface) written in Rust. It is designed to read email messages from a Microsoft 365 group mailbox using Microsoft Graph API with app-only (client credentials) authentication.

## Goals

- MVP: Read and display emails from a Microsoft 365 group mailbox.
- Future-ready: Extend to support sending emails and filtering/searching.
- Modular: Clean separation between authentication, API access, TUI rendering, and data models.

## Key Components

### 1. Auth Module (`auth.rs`)
- Handles OAuth2 client credentials flow.
- Uses the `oauth2` crate to obtain and refresh tokens.
- Reads configuration (client ID, secret, tenant ID) from environment or config file.

### 2. Graph API Module (`graph.rs`)
- Contains functions to interact with Microsoft Graph API.
- Fetches messages from `/groups/{group-id}/messages`.
- Parses and returns normalized structs for use in the UI.

### 3. TUI Module (`tui/`)
- Uses `ratatui` and `crossterm` for rendering and input handling.
- Displays messages in a scrollable list.
- Manages app state and navigation.

### 4. Config
- Loads sensitive values from `.env` or config file.
- Expected keys: `CLIENT_ID`, `CLIENT_SECRET`, `TENANT_ID`, `GROUP_ID`.

## Async Runtime

The app uses the `tokio` runtime to support non-blocking IO for Graph API requests and future enhancements like live message polling.

## Data Flow

1. `auth.rs` authenticates and retrieves a token.
2. `graph.rs` uses the token to call Microsoft Graph API.
3. Parsed message data is passed to the TUI layer.
4. TUI renders messages and handles user input.

## Extensibility

- Add `POST /sendMail` support to `graph.rs` for sending messages.
- Support viewing message bodies, filtering, and search.
- Implement caching layer in a `store.rs` module for local persistence.