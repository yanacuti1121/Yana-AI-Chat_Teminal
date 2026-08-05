// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConventionKind {
    Naming,
    ErrorHandling,
    Testing,
    Documentation,
    ModuleLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionEvidence {
    pub kind: ConventionKind,
    pub rule: String,
    pub source: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Convention {
    pub kind: ConventionKind,
    pub rule: String,
    pub support: usize,
    pub confidence_percent: u8,
    pub evidence: Vec<ConventionEvidence>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectDna {
    conventions: Vec<Convention>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnaLimits {
    pub max_files: usize,
    pub max_evidence_per_rule: usize,
    pub minimum_support: usize,
}

impl Default for DnaLimits {
    fn default() -> Self {
        Self {
            max_files: 2_000,
            max_evidence_per_rule: 8,
            minimum_support: 2,
        }
    }
}

impl ProjectDna {
    pub fn infer<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
        limits: DnaLimits,
    ) -> Self {
        let mut observations: BTreeMap<(ConventionKind, String), Vec<ConventionEvidence>> =
            BTreeMap::new();
        let mut visited = 0usize;

        for (path, source) in files {
            if visited >= limits.max_files {
                break;
            }
            visited += 1;
            observe_file(path, source, &mut observations);
        }

        let mut conventions = observations
            .into_iter()
            .filter_map(|((kind, rule), mut evidence)| {
                let support = evidence.len();
                if support < limits.minimum_support {
                    return None;
                }
                evidence.truncate(limits.max_evidence_per_rule.max(1));
                Some(Convention {
                    kind,
                    rule,
                    support,
                    confidence_percent: confidence(support, visited),
                    evidence,
                })
            })
            .collect::<Vec<_>>();

        conventions.sort_by(|left, right| {
            right
                .confidence_percent
                .cmp(&left.confidence_percent)
                .then_with(|| right.support.cmp(&left.support))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.rule.cmp(&right.rule))
        });

        Self { conventions }
    }

    pub fn conventions(&self) -> &[Convention] {
        &self.conventions
    }

    pub fn rules_for(&self, kind: ConventionKind) -> impl Iterator<Item = &Convention> {
        self.conventions.iter().filter(move |rule| rule.kind == kind)
    }

    pub fn conflicts(&self, proposed_text: &str) -> Vec<&Convention> {
        self.conventions
            .iter()
            .filter(|convention| conflicts_with(convention, proposed_text))
            .collect()
    }
}

fn observe_file(
    path: &str,
    source: &str,
    observations: &mut BTreeMap<(ConventionKind, String), Vec<ConventionEvidence>>,
) {
    let mut seen_in_file = BTreeSet::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        let line_number = index + 1;

        let candidates = [
            detect_naming(line),
            detect_error_handling(line),
            detect_testing(line),
            detect_documentation(line),
            detect_module_layout(line),
        ];

        for (kind, rule) in candidates.into_iter().flatten() {
            let key = (kind, rule.to_owned());
            if seen_in_file.insert(key.clone()) {
                observations.entry(key).or_default().push(ConventionEvidence {
                    kind,
                    rule: rule.to_owned(),
                    source: path.to_owned(),
                    line: line_number,
                });
            }
        }
    }
}

fn detect_naming(line: &str) -> Option<(ConventionKind, &'static str)> {
    if line.starts_with("pub struct ") || line.starts_with("pub enum ") || line.starts_with("pub trait ") {
        return Some((ConventionKind::Naming, "public types use PascalCase"));
    }
    if line.starts_with("pub fn ") || line.starts_with("fn ") || line.starts_with("async fn ") {
        return Some((ConventionKind::Naming, "functions use snake_case"));
    }
    None
}

fn detect_error_handling(line: &str) -> Option<(ConventionKind, &'static str)> {
    if line.contains("Result<") || line.contains("Result<(),") || line.contains("Result<Self") {
        return Some((ConventionKind::ErrorHandling, "fallible operations return Result"));
    }
    if line.contains(".unwrap()") || line.contains(".expect(") {
        return Some((ConventionKind::ErrorHandling, "unwrap-like calls are present"));
    }
    None
}

fn detect_testing(line: &str) -> Option<(ConventionKind, &'static str)> {
    if line == "#[test]" {
        return Some((ConventionKind::Testing, "unit tests use #[test]"));
    }
    if line.starts_with("mod tests") || line.starts_with("#[cfg(test)]") {
        return Some((ConventionKind::Testing, "tests live beside implementation"));
    }
    None
}

fn detect_documentation(line: &str) -> Option<(ConventionKind, &'static str)> {
    if line.starts_with("///") || line.starts_with("//!") {
        return Some((ConventionKind::Documentation, "Rustdoc documents public behavior"));
    }
    None
}

fn detect_module_layout(line: &str) -> Option<(ConventionKind, &'static str)> {
    if line.starts_with("mod ") || line.starts_with("pub mod ") {
        return Some((ConventionKind::ModuleLayout, "modules are declared explicitly"));
    }
    None
}

fn confidence(support: usize, visited_files: usize) -> u8 {
    if visited_files == 0 {
        return 0;
    }
    ((support.saturating_mul(100) / visited_files).min(100)) as u8
}

fn conflicts_with(convention: &Convention, proposed_text: &str) -> bool {
    match convention.rule.as_str() {
        "fallible operations return Result" => {
            proposed_text.contains(".unwrap()") || proposed_text.contains(".expect(")
        }
        "functions use snake_case" => proposed_text
            .lines()
            .any(|line| line.trim_start().starts_with("fn ") && line.contains(char::is_uppercase)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_only_rules_with_enough_support() {
        let dna = ProjectDna::infer(
            [
                ("a.rs", "pub fn load_file() -> Result<(), Error> { Ok(()) }"),
                ("b.rs", "pub fn save_file() -> Result<(), Error> { Ok(()) }"),
                ("c.rs", "pub struct App;"),
            ],
            DnaLimits::default(),
        );

        assert!(dna
            .conventions()
            .iter()
            .any(|rule| rule.rule == "functions use snake_case"));
        assert!(dna
            .conventions()
            .iter()
            .any(|rule| rule.rule == "fallible operations return Result"));
        assert!(!dna
            .conventions()
            .iter()
            .any(|rule| rule.rule == "public types use PascalCase"));
    }

    #[test]
    fn ordering_is_deterministic() {
        let first = ProjectDna::infer(
            [("b.rs", "fn beta() {}"), ("a.rs", "fn alpha() {}")],
            DnaLimits::default(),
        );
        let second = ProjectDna::infer(
            [("a.rs", "fn alpha() {}"), ("b.rs", "fn beta() {}")],
            DnaLimits::default(),
        );
        assert_eq!(first, second);
    }

    #[test]
    fn detects_proposals_that_conflict_with_error_policy() {
        let dna = ProjectDna::infer(
            [
                ("a.rs", "fn load() -> Result<(), Error> { Ok(()) }"),
                ("b.rs", "fn save() -> Result<(), Error> { Ok(()) }"),
            ],
            DnaLimits::default(),
        );
        let conflicts = dna.conflicts("let value = read().unwrap();");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].rule, "fallible operations return Result");
    }

    #[test]
    fn evidence_is_bounded() {
        let files = (0..20)
            .map(|index| (format!("{index}.rs"), "fn value() {}".to_owned()))
            .collect::<Vec<_>>();
        let dna = ProjectDna::infer(
            files.iter().map(|(path, source)| (path.as_str(), source.as_str())),
            DnaLimits {
                max_evidence_per_rule: 3,
                ..DnaLimits::default()
            },
        );
        assert_eq!(dna.conventions()[0].evidence.len(), 3);
    }
}
