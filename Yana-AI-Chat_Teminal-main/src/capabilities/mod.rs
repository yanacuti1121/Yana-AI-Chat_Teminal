// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

pub mod agent;
pub mod approval;
pub mod compose;
pub mod event;
pub mod memory;
pub mod streaming;

use agent::AgentEngine;
use approval::ApprovalEngine;
use compose::ComposeEngine;
use event::EventEngine;
use memory::ZeroMemory;
use streaming::StreamingEngine;

#[derive(Debug, Default)]
pub struct Capabilities {
    pub agent: AgentEngine,
    pub approval: ApprovalEngine,
    pub compose: ComposeEngine,
    pub events: EventEngine,
    pub memory: ZeroMemory,
    pub streaming: StreamingEngine,
}
