//! TUI Rendering — Ratatui-based UI

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode, ConfirmAction, Panel};

/// Main draw function — called every frame
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    // Main layout: [sidebar(22)] | [content]
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(size);

    // Content layout: [entry list] | [detail]
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    // Bottom bar layout
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(main_chunks[1]);

    let entry_and_detail = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(right_chunks[0]);

    // Draw panels
    draw_sidebar(f, app, main_chunks[0]);
    draw_entry_list(f, app, entry_and_detail[0]);
    draw_entry_detail(f, app, entry_and_detail[1]);
    draw_status_bar(f, app, right_chunks[1]);

    // Overlays
    match &app.mode {
        AppMode::Search => draw_search_overlay(f, app, size),
        AppMode::Command => draw_command_bar(f, app, right_chunks[1]),
        AppMode::Help => draw_help_overlay(f, size),
        AppMode::Confirm(action) => draw_confirm_dialog(f, app, action, size),
        _ => {}
    }
}

// ─── Sidebar (Groups) ─────────────────────────────────────────────────────────

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Groups;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " 📁 Groups ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let indent = "  ".repeat(group.depth);
            let icon = if group.is_expanded { "▼" } else { "▶" };
            let text = format!("{}{} {} ({})", indent, icon, group.name, group.entry_count);
            let style = if i == app.selected_group_idx && is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_group_idx {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(text).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_group_idx));

    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

// ─── Entry List ───────────────────────────────────────────────────────────────

fn draw_entry_list(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Entries;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let entries = if app.search_query.is_empty() {
        &app.entries
    } else {
        &app.search_results
    };

    let title = if app.search_query.is_empty() {
        format!(" 🔑 Entries ({}) ", entries.len())
    } else {
        format!(" 🔍 Results ({}) ", entries.len())
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut badges = String::new();
            if entry.is_favorite {
                badges.push('⭐');
            }
            if entry.has_otp {
                badges.push_str("🔐");
            }
            if entry.has_passkey {
                badges.push_str("🗝️");
            }
            if entry.is_expired {
                badges.push_str("⚠️");
            }

            let text = format!("{} {}{}", entry.title, badges, "");
            let sub = format!("  {}", entry.username);

            let style = if i == app.selected_entry_idx && is_active {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_entry_idx {
                Style::default().fg(Color::Cyan)
            } else if entry.is_expired {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(vec![
                Line::from(Span::styled(text, style)),
                Line::from(Span::styled(sub, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_entry_idx));

    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

// ─── Entry Detail ─────────────────────────────────────────────────────────────

fn draw_entry_detail(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Detail;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(Span::styled(
            " 📋 Detail ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style);

    if let Some(entry) = &app.selected_entry {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &entry.title,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Username: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.username, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Password: ", Style::default().fg(Color::DarkGray)),
                Span::styled("••••••••••••", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("URL:      ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &entry.url,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(vec![
                Span::styled("Group:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.group_name, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Modified: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.modified_ago, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
        ];

        // Badges
        let mut badge_spans = vec![Span::styled(
            "Features: ",
            Style::default().fg(Color::DarkGray),
        )];
        if entry.has_otp {
            badge_spans.push(Span::styled("🔐 OTP  ", Style::default().fg(Color::Green)));
        }
        if entry.has_passkey {
            badge_spans.push(Span::styled(
                "🗝️ Passkey  ",
                Style::default().fg(Color::Cyan),
            ));
        }
        if entry.is_expired {
            badge_spans.push(Span::styled("⚠️ EXPIRED", Style::default().fg(Color::Red)));
        }
        if entry.is_favorite {
            badge_spans.push(Span::styled(
                "⭐ Favorite",
                Style::default().fg(Color::Yellow),
            ));
        }
        lines.push(Line::from(badge_spans));
        lines.push(Line::from(""));

        // Keybindings hint
        lines.push(Line::from(Span::styled(
            "y=copy pwd  u=copy user  o=open url  e=edit",
            Style::default().fg(Color::DarkGray),
        )));

        let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        f.render_widget(para, area);
    } else {
        let para = Paragraph::new("No entry selected")
            .block(block)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
    }
}

// ─── Status Bar ───────────────────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let left = if let Some(status) = &app.status {
        let color = if status.is_error {
            Color::Red
        } else {
            Color::Green
        };
        Span::styled(&status.text, Style::default().fg(color))
    } else {
        Span::styled(
            "j/k=nav  /=search  y=copy  e=edit  n=new  ?=help  q=quit",
            Style::default().fg(Color::DarkGray),
        )
    };

    let right = Span::styled(
        format!(" {} | {} entries ", app.vault_name, app.entry_count),
        Style::default().fg(Color::DarkGray),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(Line::from(vec![left, Span::raw("  "), right])).block(block);
    f.render_widget(para, area);
}

// ─── Search Overlay ───────────────────────────────────────────────────────────

fn draw_search_overlay(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 3, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " 🔍 Search (NL supported: 'find weak passwords') ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let para = Paragraph::new(format!("/ {}_", app.search_query))
        .block(block)
        .style(Style::default().fg(Color::White));
    f.render_widget(para, popup_area);
}

// ─── Command Bar ──────────────────────────────────────────────────────────────

fn draw_command_bar(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let para = Paragraph::new(format!(":{}_", app.command_input))
        .block(block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(para, area);
}

// ─── Help Overlay ─────────────────────────────────────────────────────────────

fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 24, area);
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "KeePassEx TUI — Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k ↑/↓   Navigate entries"),
        Line::from("  h/l ←/→   Switch panels"),
        Line::from("  g          Go to top"),
        Line::from("  G          Go to bottom"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Enter      View entry detail"),
        Line::from("  e          Edit entry"),
        Line::from("  n          New entry"),
        Line::from("  d          Delete entry"),
        Line::from("  y          Copy password"),
        Line::from("  u          Copy username"),
        Line::from("  o          Open URL"),
        Line::from(""),
        Line::from(Span::styled(
            "Search & Commands",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  /          Search (NL supported)"),
        Line::from("  :          Command mode"),
        Line::from("  :q         Quit"),
        Line::from("  :w         Save vault"),
        Line::from("  :lock      Lock vault"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let para = Paragraph::new(help_text).block(block);
    f.render_widget(para, popup_area);
}

// ─── Confirm Dialog ───────────────────────────────────────────────────────────

fn draw_confirm_dialog(f: &mut Frame, _app: &App, action: &ConfirmAction, area: Rect) {
    let popup_area = centered_rect(40, 5, area);
    f.render_widget(Clear, popup_area);

    let msg = match action {
        ConfirmAction::DeleteEntry(_) => "Delete this entry? (y/n)",
        ConfirmAction::EmptyRecycleBin => "Empty recycle bin? (y/n)",
        ConfirmAction::LockVault => "Lock vault and exit? (y/n)",
    };

    let block = Block::default()
        .title(Span::styled(
            " Confirm ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let para = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    f.render_widget(para, popup_area);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
