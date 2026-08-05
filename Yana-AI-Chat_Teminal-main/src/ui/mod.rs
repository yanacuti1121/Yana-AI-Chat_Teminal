// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::{app::{App, Overlay, SidePanel}, capabilities::event::EventKind, domain::{ActivityState, Role}};

const CYAN: Color = Color::Rgb(76, 201, 240);
const PURPLE: Color = Color::Rgb(183, 130, 255);
const GREEN: Color = Color::Rgb(107, 203, 119);
const YELLOW: Color = Color::Rgb(244, 196, 91);
const RED: Color = Color::Rgb(238, 110, 115);
const MUTED: Color = Color::Rgb(145, 153, 166);
const PANEL: Color = Color::Rgb(62, 72, 88);

pub fn draw(frame: &mut Frame, app: &App) {
    let rows = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(3), Constraint::Min(10), Constraint::Length(4), Constraint::Length(1)
    ]).split(frame.area());
    render_header(frame, rows[0], app); render_body(frame, rows[1], app); render_composer(frame, rows[2], app); render_status(frame, rows[3], app);
    if let Some(overlay) = app.overlay { render_overlay(frame, frame.area(), app, overlay); }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let privacy = if app.runtime.local { "LOCAL" } else { "REMOTE" };
    let pending = app.capabilities.approval.pending().len();
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(" YANA ", Style::default().fg(Color::Black).bg(CYAN).add_modifier(Modifier::BOLD)), Span::styled("  AI Operating Workspace", Style::default().fg(Color::White)), Span::styled(format!("   {}", app.capabilities.agent.active().label()), Style::default().fg(PURPLE))]),
        Line::from(vec![Span::styled(format!(" {} ", app.runtime.model), Style::default().fg(PURPLE)), Span::styled("· ", Style::default().fg(MUTED)), Span::styled(&app.runtime.runtime, Style::default().fg(Color::White)), Span::styled(format!(" · {privacy} · memory {} · approval {pending}", app.capabilities.memory.len()), Style::default().fg(GREEN))]),
    ]).block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(PANEL)));
    frame.render_widget(header, area);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let show_sidebar = app.sidebar_visible && area.width >= 105;
    let columns = Layout::default().direction(Direction::Horizontal).constraints(if show_sidebar { vec![Constraint::Min(58), Constraint::Length(42)] } else { vec![Constraint::Percentage(100)] }).split(area);
    render_transcript(frame, columns[0], app); if show_sidebar { render_sidebar(frame, columns[1], app); }
}

fn render_transcript(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.messages.iter().map(|message| {
        let (label, color, marker) = match message.role { Role::User => ("YOU", GREEN, "◆"), Role::Assistant => ("YANA", CYAN, "●"), Role::System => ("SYSTEM", YELLOW, "◇") };
        ListItem::new(vec![
            Line::from(vec![Span::styled(format!("{marker} {label}"), Style::default().fg(color).add_modifier(Modifier::BOLD)), Span::styled(format!("  {}", message.timestamp), Style::default().fg(MUTED))]),
            Line::from(Span::styled(message.content.as_str(), Style::default().fg(Color::White))),
            Line::from(Span::styled("────────────────────────────────", Style::default().fg(PANEL))),
        ])
    }).collect::<Vec<_>>();
    frame.render_widget(List::new(items).block(Block::default().title(" Conversation ").borders(Borders::RIGHT).border_style(Style::default().fg(PANEL))), area);
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(48), Constraint::Percentage(52)]).split(area);
    render_events(frame, rows[0], app);
    match app.side_panel { SidePanel::Activity => render_activity(frame, rows[1], app), SidePanel::Plan => render_plan(frame, rows[1], app), SidePanel::Memory => render_memory(frame, rows[1], app) }
}

fn render_events(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.capabilities.events.recent(area.height.saturating_sub(2) as usize).into_iter().map(|event| {
        let (symbol, color) = match event.kind { EventKind::Think => ("◇", CYAN), EventKind::Read => ("↳", PURPLE), EventKind::Act => ("▶", YELLOW), EventKind::Verify => ("✓", GREEN), EventKind::Remember => ("◆", PURPLE), EventKind::Warn => ("!", RED) };
        ListItem::new(Line::from(vec![Span::styled(format!("{symbol} "), Style::default().fg(color)), Span::styled(&event.text, Style::default().fg(Color::White))]))
    }).collect::<Vec<_>>();
    frame.render_widget(List::new(items).block(panel(" Live workflow ")), area);
}

fn render_activity(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.activities.iter().rev().take(area.height.saturating_sub(2) as usize).map(|a| {
        let (symbol, color) = match a.state { ActivityState::Running => ("◇", CYAN), ActivityState::Done => ("✓", GREEN), ActivityState::Warning => ("!", YELLOW) };
        ListItem::new(vec![Line::from(vec![Span::styled(format!("{symbol} "), Style::default().fg(color)), Span::styled(&a.label, Style::default().fg(Color::White))]), Line::from(Span::styled(format!("  {}", a.detail), Style::default().fg(MUTED)))])
    }).collect::<Vec<_>>();
    frame.render_widget(List::new(items).block(panel(" Activity · Ctrl+M ")), area);
}

fn render_plan(frame: &mut Frame, area: Rect, app: &App) {
    let items = app.plan.iter().enumerate().map(|(i, step)| { let (symbol, color) = if step.done { ("✓", GREEN) } else if step.active { ("›", CYAN) } else { ("·", MUTED) }; ListItem::new(Line::from(vec![Span::styled(format!("{symbol} "), Style::default().fg(color)), Span::styled(format!("{}. {}", i+1, step.title), Style::default().fg(if step.active { Color::White } else { MUTED }))])) }).collect::<Vec<_>>();
    frame.render_widget(List::new(items).block(panel(" Compose lifecycle · Ctrl+M ")), area);
}

fn render_memory(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = app.capabilities.memory.recent(area.height.saturating_sub(4) as usize).into_iter().map(|fact| ListItem::new(vec![Line::from(vec![Span::styled(format!("#{} [{}]", fact.id, fact.kind.label()), Style::default().fg(PURPLE))]), Line::from(Span::styled(&fact.subject, Style::default().fg(Color::White))), Line::from(Span::styled(&fact.value, Style::default().fg(MUTED)))])).collect::<Vec<_>>();
    if items.is_empty() { items.push(ListItem::new("No memory facts")); }
    frame.render_widget(List::new(items).block(panel(" Zero-Memory · original evidence · Ctrl+M ")), area);
}

fn render_composer(frame: &mut Frame, area: Rect, app: &App) {
    let pending = app.capabilities.approval.latest().map(|r| format!(" approval #{}", r.id)).unwrap_or_else(|| " no pending action".into());
    let lines = vec![
        if app.input.is_empty() { Line::from(vec![Span::styled("› ", Style::default().fg(CYAN)), Span::styled("Ask Yana, @file or /help", Style::default().fg(MUTED))]) } else { Line::from(vec![Span::styled("› ", Style::default().fg(CYAN)), Span::styled(&app.input, Style::default().fg(Color::White))]) },
        Line::from(vec![Span::styled(format!(" {} · {} ·{}", app.capabilities.compose.stage().label(), app.capabilities.agent.active().label(), pending), Style::default().fg(MUTED))]),
    ];
    frame.render_widget(Paragraph::new(lines).block(Block::default().title(" Composer ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(CYAN))), area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" Ctrl+S ", Style::default().fg(PURPLE)), Span::styled("scope", Style::default().fg(MUTED)),
        Span::styled("  Ctrl+P ", Style::default().fg(PURPLE)), Span::styled("plan", Style::default().fg(MUTED)),
        Span::styled("  Ctrl+M ", Style::default().fg(PURPLE)), Span::styled("panel", Style::default().fg(MUTED)),
        Span::styled("  Tab ", Style::default().fg(PURPLE)), Span::styled("focus", Style::default().fg(MUTED)),
        Span::styled("   ·   ", Style::default().fg(PANEL)), Span::styled(&app.status, Style::default().fg(GREEN)),
    ])), area);
}

fn panel(title: &'static str) -> Block<'static> { Block::default().title(title).borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(PANEL)) }

fn render_overlay(frame: &mut Frame, area: Rect, app: &App, overlay: Overlay) {
    let popup = centered_rect(74, 72, area); frame.render_widget(Clear, popup);
    match overlay {
        Overlay::Scope => { let mut lines = vec![Line::from(Span::styled("SMART SCOPE", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))), Line::from("")]; for file in &app.scope { lines.push(Line::from(vec![Span::styled(if file.selected { "✓ " } else { "· " }, Style::default().fg(GREEN)), Span::styled(&file.path, Style::default().fg(Color::White)), Span::styled(format!("  {}% · {} lines", file.confidence, file.lines), Style::default().fg(MUTED))])); } frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(panel(" Scope Inspector ")), popup); }
        Overlay::Plan => { let lines = app.plan.iter().enumerate().map(|(i,s)| Line::from(format!("{} {}. {}", if s.done { "✓" } else if s.active { "›" } else { "·" }, i+1, s.title))).collect::<Vec<_>>(); frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left).block(panel(" Compose Plan ")), popup); }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect { let vertical = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100-percent_y)/2), Constraint::Percentage(percent_y), Constraint::Percentage((100-percent_y)/2)]).split(area); Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100-percent_x)/2), Constraint::Percentage(percent_x), Constraint::Percentage((100-percent_x)/2)]).split(vertical[1])[1] }
