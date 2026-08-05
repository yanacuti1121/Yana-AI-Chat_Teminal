// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use crate::gateway::{Capability, Gateway, GatewayError, RouteDecision, RouteRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    Planner,
    Builder,
    Reviewer,
    Researcher,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTask {
    pub id: u64,
    pub role: AgentRole,
    pub summary: String,
    pub required_capabilities: Vec<Capability>,
    pub prefer_local: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub task_id: u64,
    pub role: AgentRole,
    pub route: RouteDecision,
}

#[derive(Debug, Default)]
pub struct Conductor {
    next_task_id: u64,
    assignments: Vec<Assignment>,
}

impl Conductor {
    pub fn new() -> Self {
        Self {
            next_task_id: 1,
            assignments: Vec::new(),
        }
    }

    pub fn task(
        &mut self,
        role: AgentRole,
        summary: impl Into<String>,
        required_capabilities: Vec<Capability>,
        prefer_local: bool,
    ) -> AgentTask {
        let task = AgentTask {
            id: self.next_task_id,
            role,
            summary: summary.into(),
            required_capabilities,
            prefer_local,
        };
        self.next_task_id += 1;
        task
    }

    pub fn assign(
        &mut self,
        gateway: &Gateway,
        task: &AgentTask,
    ) -> Result<&Assignment, GatewayError> {
        let route = gateway.route(&RouteRequest {
            required: task.required_capabilities.clone(),
            prefer_local: task.prefer_local,
            preferred_provider: None,
        })?;
        self.assignments.push(Assignment {
            task_id: task.id,
            role: task.role,
            route,
        });
        Ok(self.assignments.last().expect("assignment just inserted"))
    }

    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }
}

#[cfg(test)]
mod tests {
    use crate::gateway::{ProviderManifest, Transport};

    use super::*;

    #[test]
    fn assigns_builder_to_tool_capable_local_provider() {
        let mut gateway = Gateway::new();
        gateway
            .register(ProviderManifest {
                id: "ollama-tools".into(),
                display_name: "Ollama Tools".into(),
                transport: Transport::Ollama,
                local: true,
                capabilities: vec![Capability::Chat, Capability::ToolCalling],
                default_model: Some("qwen-coder".into()),
            })
            .unwrap();

        let mut conductor = Conductor::new();
        let task = conductor.task(
            AgentRole::Builder,
            "apply workspace patch",
            vec![Capability::Chat, Capability::ToolCalling],
            true,
        );
        let assignment = conductor.assign(&gateway, &task).unwrap();
        assert_eq!(assignment.route.provider_id, "ollama-tools");
    }
}
