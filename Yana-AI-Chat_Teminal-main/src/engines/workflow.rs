// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowState {
    Idle,
    Queued,
    Rendering,
    Complete,
}

#[derive(Debug)]
pub struct WorkflowEngine {
    state: WorkflowState,
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self {
            state: WorkflowState::Idle,
        }
    }
}

impl WorkflowEngine {
    pub fn transition(&mut self, state: WorkflowState) {
        self.state = state;
    }

    pub fn state(&self) -> WorkflowState {
        self.state
    }
}
