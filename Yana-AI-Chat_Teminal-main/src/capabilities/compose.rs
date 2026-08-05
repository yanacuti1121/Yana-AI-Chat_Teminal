// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeStage { Plan, Execute, Review, Test, Reflect, Complete }

impl ComposeStage {
    pub fn label(self) -> &'static str {
        match self { Self::Plan => "Plan", Self::Execute => "Execute", Self::Review => "Review", Self::Test => "Test", Self::Reflect => "Reflect", Self::Complete => "Complete" }
    }
}

#[derive(Debug)]
pub struct ComposeEngine {
    enabled: bool,
    stage: ComposeStage,
    goal: String,
}

impl Default for ComposeEngine {
    fn default() -> Self { Self { enabled: true, stage: ComposeStage::Plan, goal: String::new() } }
}

impl ComposeEngine {
    pub fn toggle(&mut self) { self.enabled = !self.enabled; }
    pub fn enabled(&self) -> bool { self.enabled }
    pub fn begin(&mut self, goal: impl Into<String>) { self.goal = goal.into(); self.stage = ComposeStage::Plan; }
    pub fn advance(&mut self) -> ComposeStage {
        self.stage = match self.stage { ComposeStage::Plan => ComposeStage::Execute, ComposeStage::Execute => ComposeStage::Review, ComposeStage::Review => ComposeStage::Test, ComposeStage::Test => ComposeStage::Reflect, ComposeStage::Reflect | ComposeStage::Complete => ComposeStage::Complete };
        self.stage
    }
    pub fn stage(&self) -> ComposeStage { self.stage }
    pub fn goal(&self) -> &str { &self.goal }
}
