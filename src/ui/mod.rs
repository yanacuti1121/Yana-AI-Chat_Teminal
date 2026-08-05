// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    app::{App, Overlay},
    domain::{ActivityState, Role},
};

const CYAN: Color = Color::Rgb(76, 201, 240);
const PURPLE: Color = Color::Rgb(183, 130, 255);
const GREEN: Color = Color::Rgb(107, 203, 119);
const YELLOW: Color = Color::Rgb(244, 196, 91);
const MUTED: Color = Color::Rgb(160, 160, 160);
const PANEL: Color = Color::Rgb(47, 55, 68);

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, rows[0], app);
    render_body(frame, rows[1], app);
    render_composer(frame, rows[2], app);
    render_status(frame, rows[3], app);

    if let Some(overlay) = app.overlay {
        render_overlay(frame, area, app, overlay);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let privacy = if app.runtime.local { "LOCAL" } else { "REMOTE" };
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                " YANA ",
                Style::default()
                    .fg(Color::Black)
                    .bg(CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Terminal AI Workspace", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", app.runtime.model),
                Style::default().fg(PURPLE),
            ),
            Span::styled("· ", Style::default().fg(MUTED)),
            Span::styled(&app.runtime.runtime, Style::default().fg(Color::White)),
            Span::styled(" · ", Style::default().fg(MUTED)),
            Span::styled(privacy, Style::default().fg(GREEN)),
            Span::styled(" · context ", Style::default().fg(MUTED)),
            Span::styled(&app.runtime.context, Style::default().fg(Color::White)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(PANEL)),
    );

    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let show_sidebar = app.sidebar_visible && area.width >= 120;
    let columns = if show_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(60), Constraint::Length(36)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    render_transcript(frame, columns[0], app);
    if show_sidebar {
        render_sidebar(frame, columns[1], app);
    }
}

fn render_transcript(frame: &mut Frame, area: Rect, app: &App) {
    let items = app
        .messages
        .iter()
        .map(|message| {
            let (label, color) = match message.role {
                Role::User => ("YOU", GREEN),
                Role::Assistant => ("YANA", CYAN),
                Role::System => ("SYSTEM", YELLOW),
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        label,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  {}", message.timestamp),
                        Style::default().fg(MUTED),
                    ),
                ]),
                Line::from(Span::styled(
                    message.content.as_str(),
                    Style::default().fg(Color::White),
                )),
                Line::from(""),
            ])
        })
        .collect::<Vec<_>>();

    let transcript = List::new(items).block(
        Block::default()
            .title(" Transcript ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(PANEL)),
    );
    frame.render_widget(transcript, area);
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    if area.height < 18 {
        render_activity(frame, area, app);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(area);
    render_activity(frame, rows[0], app);
    render_plan(frame, rows[1], app);
}

fn render_activity(frame: &mut Frame, area: Rect, app: &App) {
    let items = app
        .activities
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .map(|activity| {
            let (symbol, color) = match activity.state {
                ActivityState::Running => ("◇", CYAN),
                ActivityState::Done => ("✓", GREEN),
                ActivityState::Warning => ("!", YELLOW),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{symbol} "), Style::default().fg(color)),
                Span::styled(&activity.label, Style::default().fg(Color::White)),
                Span::styled(format!("  {}", activity.detail), Style::default().fg(MUTED)),
            ]))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(" Activity ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PANEL)),
        ),
        area,
    );
}

fn render_plan(frame: &mut Frame, area: Rect, app: &App) {
    let items = app
        .plan
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let (symbol, color) = if step.done {
                ("✓", GREEN)
            } else if step.active {
                ("›", CYAN)
            } else {
                ("·", MUTED)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{symbol} "), Style::default().fg(color)),
                Span::styled(
                    format!("{}. {}", index + 1, step.title),
                    Style::default().fg(if step.active { Color::White } else { MUTED }),
                ),
            ]))
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items).block(
            Block::default()
                .title(" Plan · Ctrl+P ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(PANEL)),
        ),
        area,
    );
}

fn render_composer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if app.input.is_empty() {
        Line::from(vec![
            Span::styled("› ", Style::default().fg(CYAN)),
            Span::styled("Ask Yana or type /help", Style::default().fg(MUTED)),
        ])
    } else {
        Line::from(vec![
            Span::styled("› ", Style::default().fg(CYAN)),
            Span::styled(&app.input, Style::default().fg(Color::White)),
        ])
    };

    let composer = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CYAN)),
    );
    frame.render_widget(composer, area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" Ctrl+S ", Style::default().fg(PURPLE)),
        Span::styled("scope", Style::default().fg(MUTED)),
        Span::styled("  Ctrl+P ", Style::default().fg(PURPLE)),
        Span::styled("plan", Style::default().fg(MUTED)),
        Span::styled("  Tab ", Style::default().fg(PURPLE)),
        Span::styled("sidebar", Style::default().fg(MUTED)),
        Span::styled("  Esc ", Style::default().fg(PURPLE)),
        Span::styled("quit", Style::default().fg(MUTED)),
        Span::styled("   ·   ", Style::default().fg(PANEL)),
        Span::styled(&app.status, Style::default().fg(GREEN)),
        Span::styled("   ·   ", Style::default().fg(PANEL)),
        Span::styled(
            format!(
                "session {} · {:?}",
                app.engines.session.id(),
                app.engines.workflow.state()
            ),
            Style::default().fg(MUTED),
        ),
    ]));
    frame.render_widget(status, area);
}

fn render_overlay(frame: &mut Frame, area: Rect, app: &App, overlay: Overlay) {
    let popup = centered_rect(72, 70, area);
    frame.render_widget(Clear, popup);

    match overlay {
        Overlay::Scope => {
            let mut lines = vec![
                Line::from(Span::styled(
                    "SMART SCOPE",
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            for file in &app.scope {
                let marker = if file.selected { "✓" } else { "·" };
                lines.push(Line::from(vec![
                    Span::styled(format!("{marker} "), Style::default().fg(GREEN)),
                    Span::styled(&file.path, Style::default().fg(Color::White)),
                    Span::styled(
                        format!("  {}% · {} lines", file.confidence, file.lines),
                        Style::default().fg(MUTED),
                    ),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter/Esc close · scope expansion is not automatic",
                Style::default().fg(MUTED),
            )));

            frame.render_widget(
                Paragraph::new(lines).wrap(Wrap { trim: false }).block(
                    Block::default()
                        .title(" Scope Inspector ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(CYAN)),
                ),
                popup,
            );
        }
        Overlay::Plan => {
            let lines = app
                .plan
                .iter()
                .enumerate()
                .map(|(index, step)| {
                    let symbol = if step.done {
                        "✓"
                    } else if step.active {
                        "›"
                    } else {
                        "·"
                    };
                    Line::from(format!("{symbol} {}. {}", index + 1, step.title))
                })
                .collect::<Vec<_>>();

            frame.render_widget(
                Paragraph::new(lines).alignment(Alignment::Left).block(
                    Block::default()
                        .title(" Plan Tracker ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(PURPLE)),
                ),
                popup,
            );
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
