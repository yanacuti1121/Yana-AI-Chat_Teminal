// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole { Main, Planner, Builder, Reviewer, Tester }
impl AgentRole { pub fn label(self) -> &'static str { match self { Self::Main => "Main", Self::Planner => "Planner", Self::Builder => "Builder", Self::Reviewer => "Reviewer", Self::Tester => "Tester" } } }

#[derive(Debug)]
pub struct AgentEngine { active: AgentRole, calls: usize }
impl Default for AgentEngine { fn default() -> Self { Self { active: AgentRole::Main, calls: 0 } } }
impl AgentEngine {
    pub fn select(&mut self, name: &str) -> Result<AgentRole, String> {
        let role = match name.trim().to_ascii_lowercase().as_str() { "main" => AgentRole::Main, "plan" | "planner" => AgentRole::Planner, "build" | "builder" => AgentRole::Builder, "review" | "reviewer" => AgentRole::Reviewer, "test" | "tester" => AgentRole::Tester, _ => return Err(format!("unknown agent: {name}")) };
        self.active = role; self.calls += 1; Ok(role)
    }
    pub fn active(&self) -> AgentRole { self.active }
    pub fn calls(&self) -> usize { self.calls }
    pub fn labels() -> [&'static str; 5] { ["main", "planner", "builder", "reviewer", "tester"] }
}
