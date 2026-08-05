// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum EventKind { Think, Read, Act, Verify, Remember, Warn }
#[derive(Debug, Clone, PartialEq, Eq)] pub struct WorkspaceEvent { pub kind: EventKind, pub text: String }
#[derive(Debug, Default)] pub struct EventEngine { events: VecDeque<WorkspaceEvent> }
impl EventEngine {
    pub fn push(&mut self, kind: EventKind, text: impl Into<String>) { self.events.push_back(WorkspaceEvent { kind, text: text.into() }); while self.events.len() > 32 { self.events.pop_front(); } }
    pub fn recent(&self, limit: usize) -> Vec<&WorkspaceEvent> { self.events.iter().rev().take(limit).collect() }
}
