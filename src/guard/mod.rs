// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use crate::forge::{ActionKind, ActionRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    ReadOnly,
    WorkspaceWrite,
    Execute,
    Dangerous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardDecision {
    Allow,
    RequireApproval(String),
    Deny(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardReport {
    pub decision: GuardDecision,
    pub reason: String,
    pub risk: RiskLevel,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Guard {
    permission: PermissionLevel,
    protected_paths: Vec<String>,
}

impl Guard {
    pub fn new(permission: PermissionLevel) -> Self {
        Self {
            permission,
            protected_paths: vec![
                ".git".into(),
                ".env".into(),
                "secrets".into(),
                "credentials".into(),
            ],
        }
    }

    pub fn permission(&self) -> PermissionLevel {
        self.permission
    }

    pub fn set_permission(&mut self, permission: PermissionLevel) {
        self.permission = permission;
    }

    pub fn protect(&mut self, path: impl Into<String>) {
        let path = path.into();
        if !self.protected_paths.contains(&path) {
            self.protected_paths.push(path);
        }
    }

    pub fn evaluate(&self, action: &ActionRequest) -> GuardDecision {
        self.explain(action).decision
    }

    pub fn explain(&self, action: &ActionRequest) -> GuardReport {
        if self.is_protected(&action.target) {
            let reason = format!(
                "target is protected by workspace policy: {}",
                action.target
            );
            return GuardReport {
                decision: GuardDecision::Deny(reason.clone()),
                reason,
                risk: RiskLevel::High,
                suggestion: Some("operate on a non-protected workspace path instead".into()),
            };
        }

        match action.kind {
            ActionKind::Read | ActionKind::Search => GuardReport {
                decision: GuardDecision::Allow,
                reason: "read-only workspace observation".into(),
                risk: RiskLevel::Low,
                suggestion: None,
            },
            ActionKind::Create | ActionKind::Patch | ActionKind::Rename => {
                if self.permission >= PermissionLevel::WorkspaceWrite {
                    GuardReport {
                        decision: GuardDecision::RequireApproval("workspace mutation".into()),
                        reason: "the action changes workspace content".into(),
                        risk: RiskLevel::Medium,
                        suggestion: Some("review the dry-run preview before approval".into()),
                    }
                } else {
                    GuardReport {
                        decision: GuardDecision::Deny("write permission is disabled".into()),
                        reason: "the current session is read-only".into(),
                        risk: RiskLevel::Medium,
                        suggestion: Some("raise permission to WorkspaceWrite for this session".into()),
                    }
                }
            }
            ActionKind::Run => {
                if self.permission >= PermissionLevel::Execute {
                    GuardReport {
                        decision: GuardDecision::RequireApproval("command execution".into()),
                        reason: "the action starts a host process".into(),
                        risk: RiskLevel::High,
                        suggestion: Some("inspect the exact command and working directory".into()),
                    }
                } else {
                    GuardReport {
                        decision: GuardDecision::Deny("execute permission is disabled".into()),
                        reason: "the session cannot start host processes".into(),
                        risk: RiskLevel::High,
                        suggestion: Some("use dry-run or raise permission to Execute".into()),
                    }
                }
            }
            ActionKind::Git | ActionKind::Delete => {
                if self.permission >= PermissionLevel::Dangerous {
                    GuardReport {
                        decision: GuardDecision::RequireApproval("high-impact action".into()),
                        reason: "the action can remove data or alter repository history".into(),
                        risk: RiskLevel::High,
                        suggestion: Some("create a snapshot and review impact first".into()),
                    }
                } else {
                    GuardReport {
                        decision: GuardDecision::Deny("dangerous permission is disabled".into()),
                        reason: "high-impact actions require the highest permission level".into(),
                        risk: RiskLevel::High,
                        suggestion: Some("prefer a reversible alternative".into()),
                    }
                }
            }
        }
    }

    fn is_protected(&self, target: &str) -> bool {
        self.protected_paths.iter().any(|protected| {
            target == protected
                || target.starts_with(&format!("{protected}/"))
                || target.contains(&format!("/{protected}/"))
        })
    }
}

impl Default for Guard {
    fn default() -> Self {
        Self::new(PermissionLevel::ReadOnly)
    }
}

#[cfg(test)]
mod tests {
    use crate::forge::{ActionKind, ActionRequest, ActionStatus};

    use super::*;

    fn request(kind: ActionKind, target: &str) -> ActionRequest {
        ActionRequest {
            id: 1,
            kind,
            target: target.into(),
            reason: "test".into(),
            status: ActionStatus::Proposed,
        }
    }

    #[test]
    fn read_only_allows_read_and_denies_write() {
        let guard = Guard::default();
        assert_eq!(
            guard.evaluate(&request(ActionKind::Read, "src/lib.rs")),
            GuardDecision::Allow
        );
        assert!(matches!(
            guard.evaluate(&request(ActionKind::Patch, "src/lib.rs")),
            GuardDecision::Deny(_)
        ));
    }

    #[test]
    fn protected_paths_are_always_denied() {
        let guard = Guard::new(PermissionLevel::Dangerous);
        let report = guard.explain(&request(ActionKind::Read, ".env"));
        assert!(matches!(report.decision, GuardDecision::Deny(_)));
        assert_eq!(report.risk, RiskLevel::High);
        assert!(report.suggestion.is_some());
    }
}
