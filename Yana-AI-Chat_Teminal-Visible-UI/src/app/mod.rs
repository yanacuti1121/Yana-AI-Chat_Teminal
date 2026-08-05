// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::{
    capabilities::{compose::ComposeStage, event::EventKind, memory::MemoryKind, Capabilities},
    domain::{Activity, ActivityState, Message, PlanStep, Role, RuntimeInfo, ScopedFile},
    engines::{command::UiCommand, notification::NoticeLevel, workflow::WorkflowState, UiEngines},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Overlay { Scope, Plan }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum SidePanel { Activity, Plan, Memory }
impl SidePanel { pub fn next(self) -> Self { match self { Self::Activity => Self::Plan, Self::Plan => Self::Memory, Self::Memory => Self::Activity } } }

pub struct App {
    pub messages: Vec<Message>, pub activities: Vec<Activity>, pub plan: Vec<PlanStep>, pub scope: Vec<ScopedFile>,
    pub runtime: RuntimeInfo, pub input: String, pub status: String, pub sidebar_visible: bool, pub overlay: Option<Overlay>,
    pub side_panel: SidePanel, pub should_quit: bool, pub engines: UiEngines, pub capabilities: Capabilities,
}

impl App {
    pub fn demo() -> Self {
        let mut engines = UiEngines::default(); engines.context_view.sync(4, true);
        let provider = engines.provider.current().clone();
        let mut capabilities = Capabilities::default();
        capabilities.memory.remember(MemoryKind::Decision, "architecture boundary", "Terminal owns UX; Yana Core remains external", 1, epoch());
        capabilities.memory.remember(MemoryKind::Project, "product direction", "Compose + zero-token memory + sub-agents", 1, epoch()+1);
        capabilities.events.push(EventKind::Think, "Workspace restored from deterministic memory");
        Self {
            messages: vec![
                msg(Role::System, "Workspace ready · Compose, Zero-Memory and Sub-Agent capabilities are active."),
                msg(Role::User, "Thiết kế lại composer nhưng không động vào provider và history."),
                msg(Role::Assistant, "Đã khóa scope UI. Tôi sẽ lập kế hoạch, thực thi, review, test và ghi lại quyết định bằng bằng chứng gốc."),
            ],
            activities: vec![
                activity("Memory restored", "2 original facts · 0 token", ActivityState::Done),
                activity("Compose mode", "Plan → Execute → Review → Test → Reflect", ActivityState::Done),
                activity("Runtime boundary", "UI workspace · external core bridge", ActivityState::Warning),
            ],
            plan: compose_plan(ComposeStage::Plan),
            scope: vec![
                scoped("src/app/mod.rs", 98, 360), scoped("src/capabilities/", 96, 700), scoped("src/ui/mod.rs", 92, 420), scoped("src/engines/", 84, 620),
            ],
            runtime: RuntimeInfo { runtime: provider.name, model: provider.model, context: "4K · memory 0T".into(), local: provider.local },
            input: String::new(), status: "Ready · memory 2 · compose Plan".into(), sidebar_visible: true,
            overlay: None, side_panel: SidePanel::Activity, should_quit: false, engines, capabilities,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code { KeyCode::Char('c') => self.should_quit = true, KeyCode::Char('s') => self.toggle_overlay(Overlay::Scope), KeyCode::Char('p') => self.toggle_overlay(Overlay::Plan), KeyCode::Char('m') => self.side_panel = self.side_panel.next(), _ => {} }
            return;
        }
        match key.code {
            KeyCode::Esc => if self.overlay.is_some() { self.overlay = None } else { self.should_quit = true },
            KeyCode::Tab => { self.sidebar_visible = !self.sidebar_visible; self.engines.layout.set_sidebar(self.sidebar_visible); },
            KeyCode::Enter => if self.overlay.is_some() { self.overlay = None } else { self.submit_input(); },
            KeyCode::Backspace => { self.input.pop(); self.engines.composer.sync(&self.input); },
            KeyCode::Char(character) if self.overlay.is_none() => { self.input.push(character); self.engines.composer.sync(&self.input); },
            _ => {}
        }
    }

    fn submit_input(&mut self) {
        let prompt = self.input.trim().to_owned(); if prompt.is_empty() { return; }
        self.input.clear(); self.engines.composer.clear();
        if let Some(command) = self.engines.command.parse(&prompt) { self.execute_command(command); return; }
        self.messages.push(msg(Role::User, &prompt)); self.engines.chat.record_submission(); self.engines.storage.mark_dirty();
        self.engines.workflow.transition(WorkflowState::Queued); self.capabilities.streaming.start(); self.capabilities.compose.begin(&prompt);
        self.capabilities.events.push(EventKind::Think, format!("Planning: {prompt}")); self.activities.push(activity("Planning", "Compose stage 1/5", ActivityState::Running));
        self.advance_compose(); self.advance_compose();
        self.capabilities.streaming.push_chunk();
        self.messages.push(msg(Role::Assistant, "Kế hoạch đã được dựng từ scope hiện tại. Đây là workflow mô phỏng có trạng thái: Plan → Execute → Review → Test → Reflect. Hành động ghi file vẫn phải đi qua approval/bridge."));
        let approval_id = self.capabilities.approval.request("Apply proposed workspace changes", "Mock mutation request; no host action executed");
        self.capabilities.events.push(EventKind::Act, format!("Approval #{approval_id} requested"));
        self.capabilities.memory.remember(MemoryKind::Working, "current goal", prompt, self.engines.session.id(), epoch());
        self.capabilities.streaming.finish(); self.engines.workflow.transition(WorkflowState::Complete);
        self.activities.push(activity("Awaiting approval", &format!("request #{approval_id}"), ActivityState::Warning));
        self.status = format!("Approval #{approval_id} pending · memory {}", self.capabilities.memory.len());
    }

    fn advance_compose(&mut self) { let stage = self.capabilities.compose.advance(); self.plan = compose_plan(stage); self.capabilities.events.push(EventKind::Verify, format!("Compose advanced to {}", stage.label())); }

    fn execute_command(&mut self, command: UiCommand) {
        match command {
            UiCommand::Help => self.system("/compose /memory [query] /remember <fact> /agents /agent <role> /approve /reject /panel /provider /search /attach /theme /save"),
            UiCommand::Engines => self.system(&format!("14 UI engines + 6 capabilities: {}", UiEngines::names().join(", "))),
            UiCommand::Clear => { self.messages.clear(); self.status = "Transcript cleared".into(); },
            UiCommand::NewSession => { self.engines.session.new_session(); self.messages.clear(); self.system("New session started; project and decision memory remain available."); },
            UiCommand::Provider(name) => if name.is_empty() {
                let labels = self.engines.provider.labels().collect::<Vec<_>>().join(", ");
                self.system(&format!("Providers: {labels}"));
            } else {
                match self.engines.provider.select(&name).cloned() {
                    Ok(profile) => { self.runtime.runtime = profile.name.clone(); self.runtime.model = profile.model.clone(); self.runtime.local = profile.local; self.status = format!("Provider: {}", profile.name); },
                    Err(error) => self.warn(error)
                }
            },
            UiCommand::Search(query) => { let count = self.engines.search.search(&self.messages, &query); self.system(&format!("Transcript search: {count} match(es) for “{query}”.")); },
            UiCommand::Attach(path) => match self.engines.attachment.attach(&path) { Ok(()) => self.system(&format!("Attached: {path}")), Err(error) => self.warn(error) },
            UiCommand::Theme => { self.engines.theme.toggle(); self.system(&format!("Theme: {:?}", self.engines.theme.current())); },
            UiCommand::Render => { self.engines.render.toggle(); self.system(&format!("Render mode: {:?}", self.engines.render.mode())); },
            UiCommand::Save => { self.engines.storage.checkpoint(self.messages.len()); self.system("Session UI checkpoint recorded."); },
            UiCommand::Compose => { self.capabilities.compose.toggle(); self.system(&format!("Compose mode: {}", if self.capabilities.compose.enabled() { "on" } else { "off" })); },
            UiCommand::Memory(query) => self.show_memory(&query),
            UiCommand::Remember(fact) => { if fact.is_empty() { self.warn("usage: /remember <fact>") } else { let id = self.capabilities.memory.remember(MemoryKind::Project, "manual", fact, self.engines.session.id(), epoch()); self.capabilities.events.push(EventKind::Remember, format!("Stored original fact #{id}")); self.system(&format!("Remembered fact #{id} without an LLM summary.")); } },
            UiCommand::Agents => self.system(&format!("Agents: {}", crate::capabilities::agent::AgentEngine::labels().join(", "))),
            UiCommand::Agent(name) => match self.capabilities.agent.select(&name) { Ok(role) => self.system(&format!("Active sub-agent: {}", role.label())), Err(error) => self.warn(error) },
            UiCommand::Approve => self.resolve_approval(true), UiCommand::Reject => self.resolve_approval(false),
            UiCommand::Panel => { self.side_panel = self.side_panel.next(); self.status = format!("Panel: {:?}", self.side_panel); },
            UiCommand::Unknown(name) => self.warn(format!("Unknown command /{name}. Type /help.")),
        }
    }

    fn show_memory(&mut self, query: &str) {
        let message = if query.is_empty() {
            let lines = self.capabilities.memory.recent(6).into_iter().map(|f| format!("#{} [{}] {} → {}", f.id, f.kind.label(), f.subject, f.value)).collect::<Vec<_>>();
            format!("Zero-Memory · {} facts\n{}", self.capabilities.memory.len(), lines.join("\n"))
        } else {
            let matches = self.capabilities.memory.retrieve(query, 6);
            let count = matches.len();
            let lines = matches.iter().map(|f| format!("#{} [{}] {} → {}", f.id, f.kind.label(), f.subject, f.value)).collect::<Vec<_>>();
            format!("Memory query “{query}”: {count} result(s)\n{}", lines.join("\n"))
        };
        self.system(&message);
    }

    fn resolve_approval(&mut self, approved: bool) {
        let Some(request) = self.capabilities.approval.latest().cloned() else { self.warn("no pending approval"); return; };
        self.capabilities.approval.resolve(request.id);
        if approved { while self.capabilities.compose.stage() != ComposeStage::Complete { self.advance_compose(); } self.capabilities.events.push(EventKind::Verify, format!("Approval #{} accepted", request.id)); self.capabilities.memory.remember(MemoryKind::Decision, "approval", format!("Approved: {}", request.title), self.engines.session.id(), epoch()); self.system(&format!("Approved #{}. Mock workflow completed; no host mutation was executed.", request.id)); }
        else { self.capabilities.events.push(EventKind::Warn, format!("Approval #{} rejected", request.id)); self.system(&format!("Rejected #{}. Workflow stopped before mutation.", request.id)); }
    }

    fn toggle_overlay(&mut self, overlay: Overlay) { self.overlay = if self.overlay == Some(overlay) { None } else { Some(overlay) }; }
    fn system(&mut self, content: &str) { self.messages.push(msg(Role::System, content)); self.engines.notification.push(NoticeLevel::Info, content.to_owned()); }
    fn warn(&mut self, content: impl Into<String>) { let content = content.into(); self.messages.push(msg(Role::System, &format!("Warning: {content}"))); self.engines.notification.push(NoticeLevel::Warning, content.clone()); self.status = content; }
}

fn compose_plan(stage: ComposeStage) -> Vec<PlanStep> { [ComposeStage::Plan, ComposeStage::Execute, ComposeStage::Review, ComposeStage::Test, ComposeStage::Reflect].into_iter().map(|item| PlanStep { title: item.label().into(), done: rank(item) < rank(stage) || stage == ComposeStage::Complete, active: item == stage }).collect() }
fn rank(stage: ComposeStage) -> u8 { match stage { ComposeStage::Plan => 0, ComposeStage::Execute => 1, ComposeStage::Review => 2, ComposeStage::Test => 3, ComposeStage::Reflect => 4, ComposeStage::Complete => 5 } }
fn msg(role: Role, content: &str) -> Message { Message { role, content: content.into(), timestamp: timestamp() } }
fn activity(label: &str, detail: &str, state: ActivityState) -> Activity { Activity { label: label.into(), detail: detail.into(), state } }
fn scoped(path: &str, confidence: u8, lines: usize) -> ScopedFile { ScopedFile { path: path.into(), confidence, lines, selected: true } }
fn epoch() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }
fn timestamp() -> String { let seconds = epoch() % 86_400; format!("{:02}:{:02}:{:02}", seconds / 3_600, (seconds % 3_600) / 60, seconds % 60) }
