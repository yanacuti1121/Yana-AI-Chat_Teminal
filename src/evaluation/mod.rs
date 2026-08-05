// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvaluationDimension {
    Correctness,
    Safety,
    ScopeDiscipline,
    EvidenceQuality,
    Efficiency,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionScore {
    pub dimension: EvaluationDimension,
    pub earned: u32,
    pub possible: u32,
    pub evidence: Vec<String>,
}

impl DimensionScore {
    pub fn ratio_per_mille(&self) -> u16 {
        if self.possible == 0 {
            return 0;
        }
        ((u64::from(self.earned.min(self.possible)) * 1_000) / u64::from(self.possible)) as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEvaluation {
    pub task_id: String,
    pub dimensions: BTreeMap<EvaluationDimension, DimensionScore>,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub files_read: u32,
    pub files_changed: u32,
    pub context_bytes: u64,
    pub provider_input_tokens: u64,
    pub provider_output_tokens: u64,
}

impl TaskEvaluation {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            dimensions: BTreeMap::new(),
            tests_passed: 0,
            tests_failed: 0,
            files_read: 0,
            files_changed: 0,
            context_bytes: 0,
            provider_input_tokens: 0,
            provider_output_tokens: 0,
        }
    }

    pub fn record(&mut self, score: DimensionScore) {
        self.dimensions.insert(score.dimension, score);
    }

    pub fn overall_per_mille(&self) -> u16 {
        if self.dimensions.is_empty() {
            return 0;
        }
        let total: u32 = self
            .dimensions
            .values()
            .map(|score| u32::from(score.ratio_per_mille()))
            .sum();
        (total / self.dimensions.len() as u32) as u16
    }

    pub fn passed(&self, minimum_per_mille: u16) -> bool {
        self.tests_failed == 0 && self.overall_per_mille() >= minimum_per_mille
    }

    pub fn token_total(&self) -> u64 {
        self.provider_input_tokens
            .saturating_add(self.provider_output_tokens)
    }

    pub fn scope_amplification_per_mille(&self) -> u32 {
        if self.files_changed == 0 {
            return 0;
        }
        self.files_read.saturating_mul(1_000) / self.files_changed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationSuite {
    tasks: Vec<TaskEvaluation>,
}

impl EvaluationSuite {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn push(&mut self, task: TaskEvaluation) {
        self.tasks.push(task);
    }

    pub fn tasks(&self) -> &[TaskEvaluation] {
        &self.tasks
    }

    pub fn pass_rate_per_mille(&self, minimum_per_mille: u16) -> u16 {
        if self.tasks.is_empty() {
            return 0;
        }
        let passed = self
            .tasks
            .iter()
            .filter(|task| task.passed(minimum_per_mille))
            .count();
        ((passed * 1_000) / self.tasks.len()) as u16
    }

    pub fn total_tokens(&self) -> u64 {
        self.tasks.iter().map(TaskEvaluation::token_total).sum()
    }
}

impl Default for EvaluationSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_requires_tests_and_threshold() {
        let mut task = TaskEvaluation::new("task-1");
        task.record(DimensionScore {
            dimension: EvaluationDimension::Correctness,
            earned: 9,
            possible: 10,
            evidence: vec!["tests passed".into()],
        });
        assert!(task.passed(850));
        task.tests_failed = 1;
        assert!(!task.passed(850));
    }

    #[test]
    fn suite_reports_deterministic_pass_rate() {
        let mut good = TaskEvaluation::new("good");
        good.record(DimensionScore {
            dimension: EvaluationDimension::Safety,
            earned: 10,
            possible: 10,
            evidence: Vec::new(),
        });
        let bad = TaskEvaluation::new("bad");
        let mut suite = EvaluationSuite::new();
        suite.push(good);
        suite.push(bad);
        assert_eq!(suite.pass_rate_per_mille(900), 500);
    }
}
