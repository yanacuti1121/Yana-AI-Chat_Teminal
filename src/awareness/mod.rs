// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Notice,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceFacts {
    pub changed_files: usize,
    pub failing_tests: usize,
    pub pending_decisions: usize,
    pub index_stale: bool,
    pub context_percent: u8,
    pub model_tools_available: bool,
}

#[derive(Debug, Default)]
pub struct Awareness;

impl Awareness {
    pub fn inspect(facts: &WorkspaceFacts) -> Vec<Signal> {
        let mut signals = Vec::new();

        if facts.failing_tests > 0 {
            signals.push(Signal {
                severity: Severity::Critical,
                title: "Tests are failing".into(),
                detail: format!("{} failing test groups", facts.failing_tests),
                action: Some("Review failures before applying more changes".into()),
            });
        }
        if facts.index_stale {
            signals.push(Signal {
                severity: Severity::Warning,
                title: "Workspace index is stale".into(),
                detail: "Repository changes occurred after the latest Atlas index".into(),
                action: Some("Refresh Atlas incrementally".into()),
            });
        }
        if facts.context_percent >= 85 {
            signals.push(Signal {
                severity: Severity::Warning,
                title: "Context pressure is high".into(),
                detail: format!("{}% of the active context budget is used", facts.context_percent),
                action: Some("Compact or narrow scope before continuing".into()),
            });
        }
        if !facts.model_tools_available {
            signals.push(Signal {
                severity: Severity::Notice,
                title: "Model has no native tool calling".into(),
                detail: "Yana must use a constrained compatibility adapter".into(),
                action: None,
            });
        }
        if facts.changed_files > 0 {
            signals.push(Signal {
                severity: Severity::Info,
                title: "Workspace has uncommitted changes".into(),
                detail: format!("{} files changed", facts.changed_files),
                action: None,
            });
        }
        if facts.pending_decisions > 0 {
            signals.push(Signal {
                severity: Severity::Notice,
                title: "Architecture decisions need review".into(),
                detail: format!("{} pending decisions", facts.pending_decisions),
                action: Some("Resolve decision conflicts before broad refactors".into()),
            });
        }

        signals.sort_by(|left, right| right.severity.cmp(&left.severity));
        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prioritizes_critical_workspace_state() {
        let signals = Awareness::inspect(&WorkspaceFacts {
            changed_files: 2,
            failing_tests: 1,
            pending_decisions: 0,
            index_stale: true,
            context_percent: 92,
            model_tools_available: true,
        });

        assert_eq!(signals.first().unwrap().severity, Severity::Critical);
        assert!(signals.iter().any(|signal| signal.title == "Workspace index is stale"));
    }
}
