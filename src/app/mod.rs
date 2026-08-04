// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::domain::{Activity, ActivityState, Message, PlanStep, Role, RuntimeInfo, ScopedFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Scope,
    Plan,
}

pub struct App {
    pub messages: Vec<Message>,
    pub activities: Vec<Activity>,
    pub plan: Vec<PlanStep>,
    pub scope: Vec<ScopedFile>,
    pub runtime: RuntimeInfo,
    pub input: String,
    pub status: String,
    pub sidebar_visible: bool,
    pub overlay: Option<Overlay>,
    pub should_quit: bool,
}

impl App {
    pub fn demo() -> Self {
        Self {
            messages: vec![
                Message {
                    role: Role::System,
                    content: "Local-first session ready. Scope is locked until you approve expansion."
                        .into(),
                    timestamp: timestamp(),
                },
                Message {
                    role: Role::User,
                    content: "Thiết kế lại composer nhưng không động vào provider và history."
                        .into(),
                    timestamp: timestamp(),
                },
                Message {
                    role: Role::Assistant,
                    content: "Tôi đã khoanh vùng phần giao diện. Hiện có 4 file ứng viên và chưa đọc sâu ngoài scope."
                        .into(),
                    timestamp: timestamp(),
                },
            ],
            activities: vec![
                Activity {
                    label: "Scope selected".into(),
                    detail: "4 files · 1,842 lines".into(),
                    state: ActivityState::Done,
                },
                Activity {
                    label: "Reading context".into(),
                    detail: "composer.rs".into(),
                    state: ActivityState::Running,
                },
                Activity {
                    label: "Protected boundary".into(),
                    detail: "providers/, storage/".into(),
                    state: ActivityState::Warning,
                },
            ],
            plan: vec![
                PlanStep {
                    title: "Khoanh vùng UI".into(),
                    done: true,
                    active: false,
                },
                PlanStep {
                    title: "Đọc composer".into(),
                    done: true,
                    active: false,
                },
                PlanStep {
                    title: "Dựng transcript".into(),
                    done: false,
                    active: true,
                },
                PlanStep {
                    title: "Review diff".into(),
                    done: false,
                    active: false,
                },
            ],
            scope: vec![
                ScopedFile {
                    path: "src/chat/tui/composer.rs".into(),
                    confidence: 97,
                    lines: 236,
                    selected: true,
                },
                ScopedFile {
                    path: "src/chat/tui/input.rs".into(),
                    confidence: 93,
                    lines: 156,
                    selected: true,
                },
                ScopedFile {
                    path: "src/chat/tui/render.rs".into(),
                    confidence: 81,
                    lines: 612,
                    selected: true,
                },
                ScopedFile {
                    path: "src/chat/theme.rs".into(),
                    confidence: 69,
                    lines: 104,
                    selected: true,
                },
            ],
            runtime: RuntimeInfo {
                runtime: "TurboFieldfare".into(),
                model: "Gemma 4 26B-A4B-IT".into(),
                context: "4K".into(),
                local: true,
            },
            input: String::new(),
            status: "Smart Scope active".into(),
            sidebar_visible: true,
            overlay: None,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => self.should_quit = true,
                KeyCode::Char('s') => self.toggle_overlay(Overlay::Scope),
                KeyCode::Char('p') => self.toggle_overlay(Overlay::Plan),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                if self.overlay.is_some() {
                    self.overlay = None;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Tab => self.sidebar_visible = !self.sidebar_visible,
            KeyCode::Enter => {
                if self.overlay.is_some() {
                    self.overlay = None;
                } else {
                    self.submit_input();
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(character) if self.overlay.is_none() => self.input.push(character),
            _ => {}
        }
    }

    fn toggle_overlay(&mut self, overlay: Overlay) {
        self.overlay = if self.overlay == Some(overlay) {
            None
        } else {
            Some(overlay)
        };
    }

    fn submit_input(&mut self) {
        let prompt = self.input.trim().to_owned();
        if prompt.is_empty() {
            return;
        }

        self.messages.push(Message {
            role: Role::User,
            content: prompt,
            timestamp: timestamp(),
        });
        self.messages.push(Message {
            role: Role::Assistant,
            content: "MVP UI đã nhận yêu cầu. Backend model sẽ được nối qua bridge sau khi luồng giao diện được chốt."
                .into(),
            timestamp: timestamp(),
        });
        self.activities.push(Activity {
            label: "Prompt queued".into(),
            detail: "mock bridge".into(),
            state: ActivityState::Done,
        });
        self.input.clear();
        self.status = "Mock response rendered".into();
    }
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 86_400;
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
