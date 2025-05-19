use crate::graph;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState, Wrap},
    text::{Span, Line},
};
use std::{io, panic};
use copypasta::{ClipboardContext, ClipboardProvider};
use html2text::from_read;
use scraper::{Html, Selector};

use crate::graph::ConversationSummary;

pub struct App {
    pub token: String,
    pub group_id: String,
    pub messages: Vec<ConversationSummary>,
    selected_index: usize,
    should_quit: bool,
    list_state: ListState,
    details: Option<String>,
    details_scroll: u16,
    status_msg: String,
    links: Vec<String>,
}

// Helper function to format the date/time in the user's local timezone
fn format_datetime(datetime: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(datetime) {
        let local = dt.with_timezone(&chrono::Local);
        local.format("%b %d %H:%M").to_string()
    } else {
        datetime.to_string()
    }
}

// Helper function to truncate preview text
fn truncate_preview(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
}

impl App {
    pub fn new(token: String, group_id: String, messages: Vec<ConversationSummary>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            token,
            group_id,
            messages,
            selected_index: 0,
            should_quit: false,
            list_state,
            details: None,
            details_scroll: 0,
            status_msg: String::new(),
            links: Vec::new(),
        }
    }

    pub async fn refresh_messages(&mut self) -> anyhow::Result<()> {
        let new_messages = graph::fetch_messages(&self.token, &self.group_id).await?;
        self.messages = new_messages;
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.details = None;
        Ok(())
    }

    pub async fn fetch_and_set_details(&mut self) -> Result<()> {
        self.links.clear();
        self.details = None;
        self.details_scroll = 0;
        self.status_msg.clear();

        if let Some(msg) = self.messages.get(self.selected_index) {
            // Fetch the full conversation details (first thread, first post)
            let url = format!(
                "https://graph.microsoft.com/v1.0/groups/{}/conversations/{}/threads",
                self.group_id, msg.id
            );
            let client = reqwest::Client::new();
            let threads: serde_json::Value = client
                .get(&url)
                .bearer_auth(&self.token)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if let Some(thread) = threads["value"].as_array().and_then(|arr| arr.get(0)) {
                let thread_id = thread["id"].as_str().unwrap_or("");
                let url = format!(
                    "https://graph.microsoft.com/v1.0/groups/{}/conversations/{}/threads/{}/posts",
                    self.group_id, msg.id, thread_id
                );
                let posts: serde_json::Value = client
                    .get(&url)
                    .bearer_auth(&self.token)
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                if let Some(post) = posts["value"].as_array().and_then(|arr| arr.get(0)) {
                    let subject = post["subject"].as_str().unwrap_or("(No Subject)");
                    let from = post["from"]["emailAddress"]["address"].as_str().unwrap_or("<unknown>");
                    let date = post["createdDateTime"].as_str().unwrap_or("");
                    let body_type = post["body"]["contentType"].as_str().unwrap_or("");
                    let body = post["body"]["content"].as_str().unwrap_or("");

                    let mut details_content = format!(
                        "From: {}\nSubject: {}\nDate: {}\n\n",
                        from, subject, date
                    );

                    if body_type.to_lowercase() == "html" {
                        let plain_text_body = from_read(body.as_bytes(), 80);
                        details_content.push_str(&plain_text_body);
                        
                        let fragment = Html::parse_fragment(body);
                        let selector = Selector::parse("a").unwrap();
                        self.links = fragment
                            .select(&selector)
                            .filter_map(|el| el.value().attr("href").map(|s| s.to_string()))
                            .filter(|url| url != "#" && !url.starts_with("#"))
                            .collect();

                    } else {
                        details_content.push_str(body);
                        self.links.clear();
                    }
                    self.details = Some(details_content);
                    self.status_msg = format!("{} link(s) found. Press 1-{} to copy.", self.links.len(), self.links.len().min(9));
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create a custom panic hook to restore the terminal
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            original_hook(panic_info);
        }));

        // Fetch details for the initially selected message
        let _ = self.fetch_and_set_details().await;

        // Main loop
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.list_state.select(Some(self.selected_index));
                            let _ = self.fetch_and_set_details().await;
                        }
                    }
                    KeyCode::Down => {
                        if self.selected_index < self.messages.len().saturating_sub(1) {
                            self.selected_index += 1;
                            self.list_state.select(Some(self.selected_index));
                            let _ = self.fetch_and_set_details().await;
                        }
                    }
                    KeyCode::Char('r') => {
                        let _ = self.refresh_messages().await;
                        let _ = self.fetch_and_set_details().await;
                    }
                    KeyCode::PageDown | KeyCode::Char('J') => {
                        self.details_scroll = self.details_scroll.saturating_add(5);
                    }
                    KeyCode::PageUp | KeyCode::Char('K') => {
                        self.details_scroll = self.details_scroll.saturating_sub(5);
                    }
                    KeyCode::Char('j') => {
                        self.details_scroll = self.details_scroll.saturating_add(1);
                    }
                    KeyCode::Char('k') => {
                        self.details_scroll = self.details_scroll.saturating_sub(1);
                    }
                    KeyCode::Char(c) if c >= '1' && c <= '9' => {
                        let idx = (c as u8 - b'1') as usize;
                        if let Some(link) = self.links.get(idx) {
                            let mut ctx = ClipboardContext::new().unwrap();
                            ctx.set_contents(link.clone()).unwrap();
                            self.status_msg = format!("Copied link {} to clipboard!", idx + 1);
                        } else {
                            self.status_msg = format!("No link {} found.", idx + 1);
                        }
                    }
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Split the main area into two columns
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(chunks[0]);

        // Create message list items with responsive width
        let sidebar_width = main_chunks[0].width.saturating_sub(2) as usize;
        let max_subject_len = sidebar_width.saturating_sub(2);
        let mut items: Vec<ListItem> = Vec::new();
        let msg_count = self.messages.len();
        for (i, msg) in self.messages.iter().enumerate() {
            let mut subject = msg.topic.clone();
            if subject.len() > max_subject_len {
                subject.truncate(max_subject_len.saturating_sub(3));
                subject.push_str("...");
            }

            let preview = truncate_preview(&msg.preview, sidebar_width * 2);
            let (preview1, preview2) = if preview.len() > sidebar_width {
                (
                    preview[..sidebar_width].to_string(),
                    preview[sidebar_width..].to_string(),
                )
            } else {
                (preview.clone(), String::new())
            };

            let sender = msg
                .unique_senders
                .as_ref()
                .and_then(|s| s.get(0))
                .cloned()
                .unwrap_or_else(|| "".to_string());
            let datetime = msg
                .last_delivered
                .as_deref()
                .map(format_datetime)
                .unwrap_or_else(String::new);

            let dt_len = datetime.len();
            let sender_len = sender.len();
            let space = if sidebar_width > sender_len + dt_len {
                sidebar_width - sender_len - dt_len
            } else {
                1
            };
            let first_line = vec![Span::raw(format!("{}{}{}", sender, " ".repeat(space), datetime))];

            let second_line: Vec<Span> = vec![Span::styled(subject.clone(), Style::default().add_modifier(Modifier::BOLD))];
            let third_line: Vec<Span> = vec![Span::raw(preview1)];
            let fourth_line: Vec<Span> = vec![Span::raw(preview2)];
            let mut lines = vec![
                Line::from(first_line),
                Line::from(second_line),
                Line::from(third_line),
                Line::from(fourth_line),
            ];
            if i < msg_count - 1 {
                lines.push(Line::from(vec![Span::styled(
                    "─".repeat(sidebar_width),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            items.push(ListItem::new(lines));
        }

        // Create the message list
        let list = List::new(items)
            .block(Block::default().title("Messages").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        // Create the message details view
        let details_text = self.details.clone().unwrap_or_else(|| "No message selected".to_string());
        let details_lines: Vec<String> = details_text.lines().map(|s| s.to_string()).collect();

        let details_scroll = self.details_scroll as usize;
        let details_visible = main_chunks[1].height.saturating_sub(2) as usize;
        let details_slice = if details_lines.len() > details_visible {
            let start = details_scroll.min(details_lines.len().saturating_sub(details_visible));
            let end = start + details_visible;
            &details_lines[start..end]
        } else {
            &details_lines[..]
        };
        let details_text_scrolled = details_slice.join("\n");
        let details_view = Paragraph::new(details_text_scrolled)
            .block(Block::default().title("Message Details").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        // Create help text
        let help_text = Paragraph::new(format!("↑/↓: Navigate | q: Quit | r: Refresh | 1-9: Copy link\n{}", self.status_msg))
            .block(Block::default().borders(Borders::ALL));

        // Render widgets
        f.render_stateful_widget(list, main_chunks[0], &mut self.list_state);
        f.render_widget(details_view, main_chunks[1]);
        f.render_widget(help_text, chunks[1]);
    }
} 
