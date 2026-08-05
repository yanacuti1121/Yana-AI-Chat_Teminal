// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectionInput {
    pub outcome: Outcome,
    pub confidence: u8,
    pub files_read: usize,
    pub files_modified: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub scope_expansions: usize,
    pub unnecessary_reads: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reflection {
    pub outcome: Outcome,
    pub confidence: u8,
    pub strengths: Vec<String>,
    pub concerns: Vec<String>,
    pub next_time: Vec<String>,
}

#[derive(Debug, Default)]
pub struct ReflectionEngine;

impl ReflectionEngine {
    pub fn evaluate(input: ReflectionInput) -> Reflection {
        let mut strengths = Vec::new();
        let mut concerns = Vec::new();
        let mut next_time = Vec::new();

        if input.tests_failed == 0 && input.tests_passed > 0 {
            strengths.push(format!("{} verification checks passed", input.tests_passed));
        }
        if input.scope_expansions == 0 {
            strengths.push("scope remained stable".into());
        }
        if input.files_modified <= input.files_read {
            strengths.push("changes stayed inside observed files".into());
        }

        if input.tests_failed > 0 {
            concerns.push(format!("{} verification checks failed", input.tests_failed));
            next_time.push("inspect failing evidence before declaring completion".into());
        }
        if input.unnecessary_reads > 0 {
            concerns.push(format!("{} reads were not needed", input.unnecessary_reads));
            next_time.push("rank candidate files before reading deeply".into());
        }
        if input.scope_expansions > 1 {
            concerns.push("scope expanded repeatedly".into());
            next_time.push("split the task or negotiate a tighter boundary".into());
        }
        if input.confidence < 60 {
            concerns.push("confidence remained low".into());
            next_time.push("collect stronger evidence before applying changes".into());
        }

        if next_time.is_empty() {
            next_time.push("reuse this workflow for similar tasks".into());
        }

        Reflection {
            outcome: input.outcome,
            confidence: input.confidence.min(100),
            strengths,
            concerns,
            next_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_clean_success() {
        let reflection = ReflectionEngine::evaluate(ReflectionInput {
            outcome: Outcome::Success,
            confidence: 93,
            files_read: 4,
            files_modified: 2,
            tests_passed: 18,
            tests_failed: 0,
            scope_expansions: 0,
            unnecessary_reads: 0,
        });

        assert!(reflection.concerns.is_empty());
        assert!(reflection
            .strengths
            .iter()
            .any(|item| item.contains("scope remained stable")));
    }

    #[test]
    fn suggests_improvement_for_wasteful_run() {
        let reflection = ReflectionEngine::evaluate(ReflectionInput {
            outcome: Outcome::Partial,
            confidence: 48,
            files_read: 20,
            files_modified: 1,
            tests_passed: 2,
            tests_failed: 1,
            scope_expansions: 3,
            unnecessary_reads: 7,
        });

        assert!(reflection.concerns.len() >= 3);
        assert!(reflection.next_time.len() >= 3);
    }
}
