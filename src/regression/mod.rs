// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricKind {
    StartupMillis,
    AtlasIndexMillis,
    RetrievalMillis,
    PatchMillis,
    PeakMemoryBytes,
    ContextBytes,
    ProviderTokens,
    TaskScorePerMille,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LowerIsBetter,
    HigherIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegressionRule {
    pub direction: Direction,
    pub maximum_regression_per_mille: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricSample {
    pub baseline: u64,
    pub candidate: u64,
}

impl MetricSample {
    pub fn regression_per_mille(self, direction: Direction) -> u32 {
        if self.baseline == 0 {
            return if self.candidate == 0 { 0 } else { 1_000 };
        }
        let regression = match direction {
            Direction::LowerIsBetter => self.candidate.saturating_sub(self.baseline),
            Direction::HigherIsBetter => self.baseline.saturating_sub(self.candidate),
        };
        ((u128::from(regression) * 1_000) / u128::from(self.baseline)) as u32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressionFinding {
    pub metric: MetricKind,
    pub baseline: u64,
    pub candidate: u64,
    pub regression_per_mille: u32,
    pub allowed_per_mille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressionReport {
    pub findings: Vec<RegressionFinding>,
}

impl RegressionReport {
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct RegressionGate {
    rules: BTreeMap<MetricKind, RegressionRule>,
}

impl RegressionGate {
    pub fn new() -> Self {
        let mut rules = BTreeMap::new();
        rules.insert(
            MetricKind::StartupMillis,
            RegressionRule { direction: Direction::LowerIsBetter, maximum_regression_per_mille: 100 },
        );
        rules.insert(
            MetricKind::AtlasIndexMillis,
            RegressionRule { direction: Direction::LowerIsBetter, maximum_regression_per_mille: 150 },
        );
        rules.insert(
            MetricKind::RetrievalMillis,
            RegressionRule { direction: Direction::LowerIsBetter, maximum_regression_per_mille: 100 },
        );
        rules.insert(
            MetricKind::PeakMemoryBytes,
            RegressionRule { direction: Direction::LowerIsBetter, maximum_regression_per_mille: 100 },
        );
        rules.insert(
            MetricKind::TaskScorePerMille,
            RegressionRule { direction: Direction::HigherIsBetter, maximum_regression_per_mille: 20 },
        );
        Self { rules }
    }

    pub fn with_rule(mut self, metric: MetricKind, rule: RegressionRule) -> Self {
        self.rules.insert(metric, rule);
        self
    }

    pub fn evaluate(
        &self,
        samples: &BTreeMap<MetricKind, MetricSample>,
    ) -> RegressionReport {
        let mut findings = Vec::new();
        for (metric, sample) in samples {
            let Some(rule) = self.rules.get(metric) else { continue };
            let regression = sample.regression_per_mille(rule.direction);
            if regression > u32::from(rule.maximum_regression_per_mille) {
                findings.push(RegressionFinding {
                    metric: *metric,
                    baseline: sample.baseline,
                    candidate: sample.candidate,
                    regression_per_mille: regression,
                    allowed_per_mille: rule.maximum_regression_per_mille,
                });
            }
        }
        RegressionReport { findings }
    }
}

impl Default for RegressionGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_large_latency_regression() {
        let gate = RegressionGate::new();
        let samples = BTreeMap::from([(
            MetricKind::StartupMillis,
            MetricSample { baseline: 100, candidate: 120 },
        )]);
        let report = gate.evaluate(&samples);
        assert!(!report.passed());
        assert_eq!(report.findings[0].regression_per_mille, 200);
    }

    #[test]
    fn accepts_improved_task_score() {
        let gate = RegressionGate::new();
        let samples = BTreeMap::from([(
            MetricKind::TaskScorePerMille,
            MetricSample { baseline: 850, candidate: 900 },
        )]);
        assert!(gate.evaluate(&samples).passed());
    }
}
