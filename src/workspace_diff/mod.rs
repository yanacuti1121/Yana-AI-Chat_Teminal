// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    Context(String),
    Removed(String),
    Added(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffPreview {
    pub lines: Vec<DiffLine>,
    pub additions: usize,
    pub deletions: usize,
}

impl DiffPreview {
    pub fn between(before: &str, after: &str) -> Self {
        let before_lines = before.lines().collect::<Vec<_>>();
        let after_lines = after.lines().collect::<Vec<_>>();
        let mut prefix = 0;
        while prefix < before_lines.len()
            && prefix < after_lines.len()
            && before_lines[prefix] == after_lines[prefix]
        {
            prefix += 1;
        }

        let mut before_suffix = before_lines.len();
        let mut after_suffix = after_lines.len();
        while before_suffix > prefix
            && after_suffix > prefix
            && before_lines[before_suffix - 1] == after_lines[after_suffix - 1]
        {
            before_suffix -= 1;
            after_suffix -= 1;
        }

        let mut lines = Vec::new();
        if prefix > 0 {
            lines.push(DiffLine::Context(before_lines[prefix - 1].to_owned()));
        }
        for line in &before_lines[prefix..before_suffix] {
            lines.push(DiffLine::Removed((*line).to_owned()));
        }
        for line in &after_lines[prefix..after_suffix] {
            lines.push(DiffLine::Added((*line).to_owned()));
        }
        if before_suffix < before_lines.len() {
            lines.push(DiffLine::Context(before_lines[before_suffix].to_owned()));
        }

        let additions = lines.iter().filter(|line| matches!(line, DiffLine::Added(_))).count();
        let deletions = lines.iter().filter(|line| matches!(line, DiffLine::Removed(_))).count();
        Self {
            lines,
            additions,
            deletions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_additions_and_deletions() {
        let diff = DiffPreview::between("a\nb\nc\n", "a\nx\nc\n");
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);
    }
}
