// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use crate::{
    forge::{ActionKind, ActionPreview, ActionRequest, Forge, ForgeError},
    guard::{Guard, GuardDecision, GuardReport, PermissionLevel},
    harbor::{Harbor, SessionSnapshot},
    intent::{Intent, IntentEngine},
    journal::{Journal, JournalKind},
    lens::{Evidence, Lens, LensError},
    reflection::{Reflection, ReflectionEngine, ReflectionInput},
};

#[derive(Debug)]
pub struct ActionProposal {
    pub id: u64,
    pub decision: GuardDecision,
    pub report: GuardReport,
    pub preview: ActionPreview,
}

#[derive(Debug)]
pub struct OperatorCore {
    forge: Forge,
    guard: Guard,
    lens: Lens,
    harbor: Harbor,
    journal: Journal,
}

impl OperatorCore {
    pub fn new(permission: PermissionLevel) -> Self {
        Self {
            forge: Forge::new(),
            guard: Guard::new(permission),
            lens: Lens::new(),
            harbor: Harbor::new(),
            journal: Journal::new(),
        }
    }

    pub fn analyze_intent(&mut self, timestamp: u64, prompt: &str) -> Intent {
        let intent = IntentEngine::classify(prompt);
        self.journal.record(
            timestamp,
            JournalKind::Think,
            "intent classified",
            format!("{:?} · {:?}", intent.kind, intent.risk),
        );
        intent
    }

    pub fn propose(
        &mut self,
        timestamp: u64,
        kind: ActionKind,
        target: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<ActionProposal, OperatorError> {
        let id = self.forge.propose(kind, target, reason)?;
        let action = self
            .forge
            .action(id)
            .expect("newly proposed action must exist");
        let report = self.guard.explain(action);
        let decision = report.decision.clone();
        let preview = self.forge.preview(id)?;

        self.journal.record(
            timestamp,
            JournalKind::Plan,
            "action preview prepared",
            preview.summary.clone(),
        );

        let (journal_kind, summary) = match &decision {
            GuardDecision::Allow => (JournalKind::Decide, "action allowed"),
            GuardDecision::RequireApproval(_) => {
                (JournalKind::Warning, "action awaiting approval")
            }
            GuardDecision::Deny(_) => (JournalKind::Warning, "action denied"),
        };

        self.journal.record(
            timestamp,
            journal_kind,
            summary,
            format!("#{id} · {}", report.reason),
        );

        if matches!(&decision, GuardDecision::Deny(_)) {
            self.forge.reject(id)?;
        }

        Ok(ActionProposal {
            id,
            decision,
            report,
            preview,
        })
    }

    pub fn approve(&mut self, timestamp: u64, id: u64) -> Result<(), OperatorError> {
        let action = self.forge.action(id).ok_or(ForgeError::UnknownAction(id))?;
        match self.guard.evaluate(action) {
            GuardDecision::Deny(reason) => return Err(OperatorError::Denied(reason)),
            GuardDecision::Allow | GuardDecision::RequireApproval(_) => {}
        }

        self.forge.approve(id)?;
        self.journal.record(
            timestamp,
            JournalKind::Decide,
            "action approved",
            format!("action #{id}"),
        );
        Ok(())
    }

    pub fn complete(
        &mut self,
        timestamp: u64,
        id: u64,
        evidence: Option<Evidence>,
    ) -> Result<(), OperatorError> {
        self.forge.complete(id)?;
        if let Some(evidence) = evidence {
            self.lens.collect(evidence);
        }
        self.journal.record(
            timestamp,
            JournalKind::Verify,
            "action completed",
            format!("action #{id}"),
        );
        Ok(())
    }

    pub fn reflect(&mut self, timestamp: u64, input: ReflectionInput) -> Reflection {
        let reflection = ReflectionEngine::evaluate(input);
        let detail = format!(
            "outcome={:?} confidence={} strengths={} concerns={} next={}",
            reflection.outcome,
            reflection.confidence,
            reflection.strengths.len(),
            reflection.concerns.len(),
            reflection.next_time.len()
        );
        self.journal
            .record(timestamp, JournalKind::Reflect, "task reflection", detail);
        reflection
    }

    pub fn attach_evidence(&mut self, evidence: Evidence) {
        self.lens.collect(evidence);
    }

    pub fn snapshot_session(
        &mut self,
        id: impl Into<String>,
        title: impl Into<String>,
        workspace: impl Into<String>,
        active_scope: Vec<String>,
        created_at: u64,
    ) {
        let pending_actions = self
            .forge
            .pending()
            .map(|action| action.id)
            .collect::<Vec<_>>();

        self.harbor.dock(SessionSnapshot {
            id: id.into(),
            title: title.into(),
            workspace: workspace.into(),
            active_scope,
            pending_actions,
            created_at,
        });
    }

    pub fn action(&self, id: u64) -> Option<&ActionRequest> {
        self.forge.action(id)
    }

    pub fn forge(&self) -> &Forge {
        &self.forge
    }

    pub fn guard(&self) -> &Guard {
        &self.guard
    }

    pub fn lens(&self) -> &Lens {
        &self.lens
    }

    pub fn harbor(&self) -> &Harbor {
        &self.harbor
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }
}

#[derive(Debug)]
pub enum OperatorError {
    Forge(ForgeError),
    Lens(LensError),
    Denied(String),
}

impl From<ForgeError> for OperatorError {
    fn from(error: ForgeError) -> Self {
        Self::Forge(error)
    }
}

impl From<LensError> for OperatorError {
    fn from(error: LensError) -> Self {
        Self::Lens(error)
    }
}

impl std::fmt::Display for OperatorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forge(error) => write!(formatter, "{error}"),
            Self::Lens(error) => write!(formatter, "{error}"),
            Self::Denied(reason) => write!(formatter, "action denied: {reason}"),
        }
    }
}

impl std::error::Error for OperatorError {}

#[cfg(test)]
mod tests {
    use crate::{
        forge::{ActionKind, ActionStatus, ImpactLevel},
        guard::{GuardDecision, PermissionLevel},
        intent::IntentKind,
        lens::Evidence,
        reflection::{Outcome, ReflectionInput},
    };

    use super::*;

    #[test]
    fn write_action_requires_approval_then_records_evidence() {
        let mut operator = OperatorCore::new(PermissionLevel::WorkspaceWrite);
        let proposal = operator
            .propose(
                1,
                ActionKind::Patch,
                "src/ui/mod.rs",
                "apply Sky Lake theme",
            )
            .unwrap();
        assert!(matches!(
            &proposal.decision,
            GuardDecision::RequireApproval(_)
        ));
        assert_eq!(proposal.preview.impact, ImpactLevel::Medium);

        operator.approve(2, proposal.id).unwrap();
        operator
            .complete(
                3,
                proposal.id,
                Some(Evidence::new("src/ui/mod.rs", 1, 20, "patch verified", 95).unwrap()),
            )
            .unwrap();

        assert_eq!(
            operator.action(proposal.id).unwrap().status,
            ActionStatus::Completed
        );
        assert_eq!(operator.lens().all().len(), 1);
    }

    #[test]
    fn dangerous_action_is_denied_without_permission() {
        let mut operator = OperatorCore::new(PermissionLevel::WorkspaceWrite);
        let proposal = operator
            .propose(1, ActionKind::Delete, "src/old.rs", "cleanup")
            .unwrap();
        assert!(matches!(&proposal.decision, GuardDecision::Deny(_)));
        assert!(proposal.report.suggestion.is_some());
        assert_eq!(
            operator.action(proposal.id).unwrap().status,
            ActionStatus::Rejected
        );
    }

    #[test]
    fn intent_and_reflection_are_recorded() {
        let mut operator = OperatorCore::new(PermissionLevel::WorkspaceWrite);
        let intent = operator.analyze_intent(1, "Thiết kế lại giao diện composer");
        assert_eq!(intent.kind, IntentKind::Refactor);

        let reflection = operator.reflect(
            2,
            ReflectionInput {
                outcome: Outcome::Success,
                confidence: 91,
                files_read: 4,
                files_modified: 2,
                tests_passed: 12,
                tests_failed: 0,
                scope_expansions: 0,
                unnecessary_reads: 0,
            },
        );
        assert!(reflection.concerns.is_empty());
        assert_eq!(operator.journal().reflection().count(), 1);
    }
}
