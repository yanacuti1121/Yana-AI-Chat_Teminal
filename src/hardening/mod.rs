// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HardeningCheck {
    Formatting,
    Clippy,
    Tests,
    LockedBuild,
    LicenseHeaders,
    DependencyAudit,
    SecretScan,
    RecoveryExercise,
    RollbackExercise,
    ProviderContract,
    KnowledgeDeterminism,
    CrossPlatformBuild,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub check: HardeningCheck,
    pub state: CheckState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardeningReport {
    pub results: Vec<CheckResult>,
}

impl HardeningReport {
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }

    pub fn record(&mut self, result: CheckResult) {
        if let Some(existing) = self
            .results
            .iter_mut()
            .find(|entry| entry.check == result.check)
        {
            *existing = result;
        } else {
            self.results.push(result);
            self.results.sort_by_key(|entry| entry.check);
        }
    }

    pub fn release_ready(&self) -> bool {
        REQUIRED_CHECKS.iter().all(|required| {
            self.results
                .iter()
                .any(|result| result.check == *required && result.state == CheckState::Passed)
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = &CheckResult> {
        self.results
            .iter()
            .filter(|result| result.state == CheckState::Failed)
    }

    pub fn missing_required(&self) -> Vec<HardeningCheck> {
        REQUIRED_CHECKS
            .iter()
            .copied()
            .filter(|required| {
                !self
                    .results
                    .iter()
                    .any(|result| result.check == *required && result.state == CheckState::Passed)
            })
            .collect()
    }
}

impl Default for HardeningReport {
    fn default() -> Self {
        Self::new()
    }
}

pub const REQUIRED_CHECKS: &[HardeningCheck] = &[
    HardeningCheck::Formatting,
    HardeningCheck::Clippy,
    HardeningCheck::Tests,
    HardeningCheck::LockedBuild,
    HardeningCheck::LicenseHeaders,
    HardeningCheck::SecretScan,
    HardeningCheck::RecoveryExercise,
    HardeningCheck::RollbackExercise,
    HardeningCheck::ProviderContract,
    HardeningCheck::KnowledgeDeterminism,
    HardeningCheck::CrossPlatformBuild,
    HardeningCheck::Documentation,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseDecision {
    pub allowed: bool,
    pub reasons: Vec<String>,
}

impl ReleaseDecision {
    pub fn from_report(report: &HardeningReport) -> Self {
        let mut reasons = Vec::new();
        for failure in report.failures() {
            reasons.push(format!("{:?} failed: {}", failure.check, failure.evidence));
        }
        for missing in report.missing_required() {
            reasons.push(format!("required check has not passed: {missing:?}"));
        }
        Self {
            allowed: reasons.is_empty(),
            reasons,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_report_cannot_release() {
        let mut report = HardeningReport::new();
        report.record(CheckResult {
            check: HardeningCheck::Tests,
            state: CheckState::Passed,
            evidence: "all tests passed".into(),
        });
        let decision = ReleaseDecision::from_report(&report);
        assert!(!decision.allowed);
        assert!(!decision.reasons.is_empty());
    }

    #[test]
    fn replacing_result_is_deterministic() {
        let mut report = HardeningReport::new();
        report.record(CheckResult {
            check: HardeningCheck::Tests,
            state: CheckState::Failed,
            evidence: "one failure".into(),
        });
        report.record(CheckResult {
            check: HardeningCheck::Tests,
            state: CheckState::Passed,
            evidence: "fixed".into(),
        });
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].state, CheckState::Passed);
    }
}
