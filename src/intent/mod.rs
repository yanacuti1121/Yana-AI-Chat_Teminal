// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentKind {
    Explore,
    Explain,
    Refactor,
    Fix,
    Test,
    Review,
    Operate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent {
    pub kind: IntentKind,
    pub risk: IntentRisk,
    pub needs_scope: bool,
    pub needs_approval: bool,
    pub suggested_scope: Vec<String>,
}

#[derive(Debug, Default)]
pub struct IntentEngine;

impl IntentEngine {
    pub fn classify(prompt: &str) -> Intent {
        let normalized = prompt.to_lowercase();

        let kind = if contains_any(&normalized, &["review", "audit", "kiểm tra code"]) {
            IntentKind::Review
        } else if contains_any(&normalized, &["test", "cargo test", "pytest", "kiểm thử"]) {
            IntentKind::Test
        } else if contains_any(&normalized, &["fix", "bug", "lỗi", "sửa lỗi"]) {
            IntentKind::Fix
        } else if contains_any(&normalized, &["refactor", "thiết kế lại", "đổi giao diện"]) {
            IntentKind::Refactor
        } else if contains_any(&normalized, &["run", "deploy", "commit", "push", "chạy lệnh"]) {
            IntentKind::Operate
        } else if contains_any(&normalized, &["explain", "why", "giải thích", "tại sao"]) {
            IntentKind::Explain
        } else {
            IntentKind::Explore
        };

        let risk = match kind {
            IntentKind::Operate => IntentRisk::High,
            IntentKind::Refactor | IntentKind::Fix | IntentKind::Test => IntentRisk::Medium,
            IntentKind::Explore | IntentKind::Explain | IntentKind::Review => IntentRisk::Low,
        };

        let suggested_scope = infer_scope(&normalized);
        Intent {
            kind,
            risk,
            needs_scope: !matches!(kind, IntentKind::Explain),
            needs_approval: matches!(
                kind,
                IntentKind::Refactor | IntentKind::Fix | IntentKind::Operate
            ),
            suggested_scope,
        }
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn infer_scope(prompt: &str) -> Vec<String> {
    let mut scope = Vec::new();
    if contains_any(prompt, &["ui", "giao diện", "composer", "theme"]) {
        scope.push("src/ui".into());
    }
    if contains_any(prompt, &["memory", "bộ nhớ"]) {
        scope.push("src/memory".into());
    }
    if contains_any(prompt, &["workspace", "file", "repo"]) {
        scope.push("src/workspace".into());
    }
    if contains_any(prompt, &["guard", "permission", "an toàn"]) {
        scope.push("src/guard".into());
    }
    scope
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_ui_refactor_with_scope() {
        let intent = IntentEngine::classify("Thiết kế lại giao diện composer");
        assert_eq!(intent.kind, IntentKind::Refactor);
        assert_eq!(intent.risk, IntentRisk::Medium);
        assert!(intent.needs_approval);
        assert_eq!(intent.suggested_scope, vec!["src/ui"]);
    }

    #[test]
    fn classifies_explanation_as_low_risk() {
        let intent = IntentEngine::classify("Giải thích tại sao guard chặn lệnh này");
        assert_eq!(intent.kind, IntentKind::Explain);
        assert_eq!(intent.risk, IntentRisk::Low);
        assert!(!intent.needs_approval);
    }
}
