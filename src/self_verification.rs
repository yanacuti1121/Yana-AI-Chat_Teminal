// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use crate::project_dna::ProjectDna;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationStage {
    Compile,
    Tests,
    StaticAnalysis,
    ProjectDna,
    Knowledge,
    Scope,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationEvidence {
    pub source: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationCheck {
    pub stage: VerificationStage,
    pub name: String,
    pub status: CheckStatus,
    pub weight: u8,
    pub evidence: Vec<VerificationEvidence>,
}

impl VerificationCheck {
    pub fn passed(stage: VerificationStage, name: impl Into<String>, weight: u8) -> Self {
        Self {
            stage,
            name: name.into(),
            status: CheckStatus::Passed,
            weight: weight.max(1),
            evidence: Vec::new(),
        }
    }

    pub fn warning(stage: VerificationStage, name: impl Into<String>, weight: u8) -> Self {
        Self {
            stage,
            name: name.into(),
            status: CheckStatus::Warning,
            weight: weight.max(1),
            evidence: Vec::new(),
        }
    }

    pub fn failed(stage: VerificationStage, name: impl Into<String>, weight: u8) -> Self {
        Self {
            stage,
            name: name.into(),
            status: CheckStatus::Failed,
            weight: weight.max(1),
            evidence: Vec::new(),
        }
    }

    pub fn with_evidence(
        mut self,
        source: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        self.evidence.push(VerificationEvidence {
            source: source.into(),
            detail: detail.into(),
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationInput {
    pub changed_files: Vec<String>,
    pub proposed_text: BTreeMap<String, String>,
    pub expected_tests: BTreeSet<String>,
    pub executed_tests: BTreeMap<String, bool>,
    pub compile_passed: Option<bool>,
    pub static_analysis_passed: Option<bool>,
    pub knowledge_evidence_ids: BTreeSet<String>,
    pub rollback_snapshot_present: bool,
}

impl Default for VerificationInput {
    fn default() -> Self {
        Self {
            changed_files: Vec::new(),
            proposed_text: BTreeMap::new(),
            expected_tests: BTreeSet::new(),
            executed_tests: BTreeMap::new(),
            compile_passed: None,
            static_analysis_passed: None,
            knowledge_evidence_ids: BTreeSet::new(),
            rollback_snapshot_present: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerificationPolicy {
    pub minimum_confidence_percent: u8,
    pub require_compile: bool,
    pub require_static_analysis: bool,
    pub require_tests_for_code_changes: bool,
    pub require_knowledge_evidence: bool,
    pub require_rollback_snapshot: bool,
    pub max_changed_files: usize,
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            minimum_confidence_percent: 80,
            require_compile: true,
            require_static_analysis: true,
            require_tests_for_code_changes: true,
            require_knowledge_evidence: true,
            require_rollback_snapshot: true,
            max_changed_files: 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    checks: Vec<VerificationCheck>,
    pub confidence_percent: u8,
    pub passed: bool,
    pub blocking_failures: usize,
    pub warnings: usize,
}

impl VerificationReport {
    pub fn checks(&self) -> &[VerificationCheck] {
        &self.checks
    }

    pub fn by_stage(&self, stage: VerificationStage) -> impl Iterator<Item = &VerificationCheck> {
        self.checks.iter().filter(move |check| check.stage == stage)
    }
}

pub struct SelfVerifier;

impl SelfVerifier {
    pub fn verify(
        input: &VerificationInput,
        dna: Option<&ProjectDna>,
        policy: VerificationPolicy,
    ) -> VerificationReport {
        let mut checks = Vec::new();
        checks.push(check_scope(input, policy));
        checks.push(check_compile(input, policy));
        checks.push(check_tests(input, policy));
        checks.push(check_static_analysis(input, policy));
        checks.push(check_project_dna(input, dna));
        checks.push(check_knowledge(input, policy));
        checks.push(check_recovery(input, policy));

        checks.sort_by(|left, right| {
            left.stage
                .cmp(&right.stage)
                .then_with(|| left.name.cmp(&right.name))
        });

        let confidence_percent = confidence(&checks);
        let blocking_failures = checks
            .iter()
            .filter(|check| check.status == CheckStatus::Failed)
            .count();
        let warnings = checks
            .iter()
            .filter(|check| check.status == CheckStatus::Warning)
            .count();
        let passed = blocking_failures == 0
            && confidence_percent >= policy.minimum_confidence_percent;

        VerificationReport {
            checks,
            confidence_percent,
            passed,
            blocking_failures,
            warnings,
        }
    }
}

fn check_scope(input: &VerificationInput, policy: VerificationPolicy) -> VerificationCheck {
    if input.changed_files.len() > policy.max_changed_files {
        return VerificationCheck::failed(
            VerificationStage::Scope,
            "changed file count stays within approved scope",
            20,
        )
        .with_evidence(
            "workspace",
            format!(
                "{} changed files exceeds limit {}",
                input.changed_files.len(),
                policy.max_changed_files
            ),
        );
    }

    VerificationCheck::passed(
        VerificationStage::Scope,
        "changed file count stays within approved scope",
        20,
    )
    .with_evidence(
        "workspace",
        format!("{} changed files", input.changed_files.len()),
    )
}

fn check_compile(input: &VerificationInput, policy: VerificationPolicy) -> VerificationCheck {
    match input.compile_passed {
        Some(true) => VerificationCheck::passed(
            VerificationStage::Compile,
            "workspace compiles",
            20,
        ),
        Some(false) => VerificationCheck::failed(
            VerificationStage::Compile,
            "workspace compiles",
            20,
        ),
        None if policy.require_compile => VerificationCheck::failed(
            VerificationStage::Compile,
            "workspace compiles",
            20,
        )
        .with_evidence("verification", "compile result is missing"),
        None => VerificationCheck {
            stage: VerificationStage::Compile,
            name: "workspace compiles".into(),
            status: CheckStatus::Skipped,
            weight: 20,
            evidence: Vec::new(),
        },
    }
}

fn check_tests(input: &VerificationInput, policy: VerificationPolicy) -> VerificationCheck {
    let code_changed = input.changed_files.iter().any(|path| {
        path.ends_with(".rs")
            || path.ends_with(".py")
            || path.ends_with(".js")
            || path.ends_with(".ts")
            || path.ends_with(".tsx")
    });

    let missing = input
        .expected_tests
        .iter()
        .filter(|test| !input.executed_tests.contains_key(*test))
        .cloned()
        .collect::<Vec<_>>();
    let failed = input
        .executed_tests
        .iter()
        .filter_map(|(name, passed)| (!*passed).then_some(name.clone()))
        .collect::<Vec<_>>();

    if !failed.is_empty() {
        return VerificationCheck::failed(VerificationStage::Tests, "required tests pass", 20)
            .with_evidence("tests", format!("failed: {}", failed.join(", ")));
    }
    if !missing.is_empty() {
        return VerificationCheck::failed(VerificationStage::Tests, "required tests pass", 20)
            .with_evidence("tests", format!("not executed: {}", missing.join(", ")));
    }
    if code_changed && policy.require_tests_for_code_changes && input.executed_tests.is_empty() {
        return VerificationCheck::failed(VerificationStage::Tests, "required tests pass", 20)
            .with_evidence("tests", "code changed but no tests were executed");
    }

    VerificationCheck::passed(VerificationStage::Tests, "required tests pass", 20)
        .with_evidence("tests", format!("{} tests executed", input.executed_tests.len()))
}

fn check_static_analysis(
    input: &VerificationInput,
    policy: VerificationPolicy,
) -> VerificationCheck {
    match input.static_analysis_passed {
        Some(true) => VerificationCheck::passed(
            VerificationStage::StaticAnalysis,
            "static analysis passes",
            10,
        ),
        Some(false) => VerificationCheck::failed(
            VerificationStage::StaticAnalysis,
            "static analysis passes",
            10,
        ),
        None if policy.require_static_analysis => VerificationCheck::failed(
            VerificationStage::StaticAnalysis,
            "static analysis passes",
            10,
        )
        .with_evidence("verification", "static analysis result is missing"),
        None => VerificationCheck {
            stage: VerificationStage::StaticAnalysis,
            name: "static analysis passes".into(),
            status: CheckStatus::Skipped,
            weight: 10,
            evidence: Vec::new(),
        },
    }
}

fn check_project_dna(
    input: &VerificationInput,
    dna: Option<&ProjectDna>,
) -> VerificationCheck {
    let Some(dna) = dna else {
        return VerificationCheck {
            stage: VerificationStage::ProjectDna,
            name: "proposal follows established Project DNA".into(),
            status: CheckStatus::Skipped,
            weight: 10,
            evidence: Vec::new(),
        };
    };

    let mut conflicts = Vec::new();
    for (path, text) in &input.proposed_text {
        for convention in dna.conflicts(text) {
            conflicts.push(format!("{path}: {}", convention.rule));
        }
    }
    conflicts.sort();
    conflicts.dedup();

    if conflicts.is_empty() {
        VerificationCheck::passed(
            VerificationStage::ProjectDna,
            "proposal follows established Project DNA",
            10,
        )
    } else {
        VerificationCheck::warning(
            VerificationStage::ProjectDna,
            "proposal follows established Project DNA",
            10,
        )
        .with_evidence("project-dna", conflicts.join("; "))
    }
}

fn check_knowledge(
    input: &VerificationInput,
    policy: VerificationPolicy,
) -> VerificationCheck {
    if policy.require_knowledge_evidence && input.knowledge_evidence_ids.is_empty() {
        return VerificationCheck::failed(
            VerificationStage::Knowledge,
            "changes retain evidence linkage",
            10,
        )
        .with_evidence("knowledge", "no evidence IDs were attached");
    }

    VerificationCheck::passed(
        VerificationStage::Knowledge,
        "changes retain evidence linkage",
        10,
    )
    .with_evidence(
        "knowledge",
        format!("{} evidence IDs", input.knowledge_evidence_ids.len()),
    )
}

fn check_recovery(input: &VerificationInput, policy: VerificationPolicy) -> VerificationCheck {
    if policy.require_rollback_snapshot && !input.rollback_snapshot_present {
        return VerificationCheck::failed(
            VerificationStage::Recovery,
            "rollback snapshot exists",
            10,
        );
    }

    VerificationCheck::passed(
        VerificationStage::Recovery,
        "rollback snapshot exists",
        10,
    )
}

fn confidence(checks: &[VerificationCheck]) -> u8 {
    let total_weight: usize = checks
        .iter()
        .filter(|check| check.status != CheckStatus::Skipped)
        .map(|check| usize::from(check.weight))
        .sum();
    if total_weight == 0 {
        return 0;
    }

    let earned: usize = checks
        .iter()
        .filter(|check| check.status != CheckStatus::Skipped)
        .map(|check| {
            let weight = usize::from(check.weight);
            match check.status {
                CheckStatus::Passed => weight,
                CheckStatus::Warning => weight / 2,
                CheckStatus::Failed | CheckStatus::Skipped => 0,
            }
        })
        .sum();

    ((earned * 100) / total_weight).min(100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_dna::{DnaLimits, ProjectDna};

    fn valid_input() -> VerificationInput {
        VerificationInput {
            changed_files: vec!["src/lib.rs".into()],
            proposed_text: BTreeMap::from([(
                "src/lib.rs".into(),
                "fn load() -> Result<(), Error> { Ok(()) }".into(),
            )]),
            expected_tests: BTreeSet::from(["unit".into()]),
            executed_tests: BTreeMap::from([("unit".into(), true)]),
            compile_passed: Some(true),
            static_analysis_passed: Some(true),
            knowledge_evidence_ids: BTreeSet::from(["evidence-1".into()]),
            rollback_snapshot_present: true,
        }
    }

    #[test]
    fn complete_evidence_backed_verification_passes() {
        let report = SelfVerifier::verify(&valid_input(), None, VerificationPolicy::default());
        assert!(report.passed);
        assert_eq!(report.confidence_percent, 100);
        assert_eq!(report.blocking_failures, 0);
    }

    #[test]
    fn missing_tests_block_code_changes() {
        let mut input = valid_input();
        input.expected_tests.clear();
        input.executed_tests.clear();
        let report = SelfVerifier::verify(&input, None, VerificationPolicy::default());
        assert!(!report.passed);
        assert!(report
            .by_stage(VerificationStage::Tests)
            .any(|check| check.status == CheckStatus::Failed));
    }

    #[test]
    fn failed_compile_is_always_blocking() {
        let mut input = valid_input();
        input.compile_passed = Some(false);
        let report = SelfVerifier::verify(&input, None, VerificationPolicy::default());
        assert!(!report.passed);
        assert!(report.blocking_failures > 0);
    }

    #[test]
    fn project_dna_conflict_reduces_confidence_without_bypassing_policy() {
        let dna = ProjectDna::infer(
            [
                ("a.rs", "fn load() -> Result<(), Error> { Ok(()) }"),
                ("b.rs", "fn save() -> Result<(), Error> { Ok(()) }"),
            ],
            DnaLimits::default(),
        );
        let mut input = valid_input();
        input.proposed_text.insert(
            "src/lib.rs".into(),
            "fn load() { read().unwrap(); }".into(),
        );
        let report = SelfVerifier::verify(&input, Some(&dna), VerificationPolicy::default());
        assert!(report.warnings > 0);
        assert!(report.confidence_percent < 100);
    }

    #[test]
    fn excessive_scope_is rejected() {
        let mut input = valid_input();
        input.changed_files = (0..25).map(|index| format!("src/{index}.rs")).collect();
        let report = SelfVerifier::verify(&input, None, VerificationPolicy::default());
        assert!(!report.passed);
        assert!(report
            .by_stage(VerificationStage::Scope)
            .any(|check| check.status == CheckStatus::Failed));
    }

    #[test]
    fn report_order_is_deterministic() {
        let first = SelfVerifier::verify(&valid_input(), None, VerificationPolicy::default());
        let second = SelfVerifier::verify(&valid_input(), None, VerificationPolicy::default());
        assert_eq!(first, second);
    }
}
