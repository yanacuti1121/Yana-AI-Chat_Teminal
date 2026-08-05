// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use crate::workspace_io::{WorkspaceIo, WorkspaceIoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextHunk {
    pub expected: String,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPatchPlan {
    pub path: PathBuf,
    pub original: String,
    pub updated: String,
    pub hunks: usize,
}

impl TextPatchPlan {
    pub fn prepare(
        workspace: &WorkspaceIo,
        path: impl Into<PathBuf>,
        hunks: &[TextHunk],
    ) -> Result<Self, TextPatchError> {
        let path = path.into();
        let original = workspace.read_text(&path)?;
        let mut updated = original.clone();

        for (index, hunk) in hunks.iter().enumerate() {
            if hunk.expected.is_empty() {
                return Err(TextPatchError::EmptyExpected { index });
            }
            let matches = updated.match_indices(&hunk.expected).count();
            if matches == 0 {
                return Err(TextPatchError::ContextMissing { index });
            }
            if matches > 1 {
                return Err(TextPatchError::ContextAmbiguous { index, matches });
            }
            updated = updated.replacen(&hunk.expected, &hunk.replacement, 1);
        }

        Ok(Self {
            path,
            original,
            updated,
            hunks: hunks.len(),
        })
    }

    pub fn changed(&self) -> bool {
        self.original != self.updated
    }
}

#[derive(Debug)]
pub enum TextPatchError {
    Workspace(WorkspaceIoError),
    EmptyExpected { index: usize },
    ContextMissing { index: usize },
    ContextAmbiguous { index: usize, matches: usize },
}

impl From<WorkspaceIoError> for TextPatchError {
    fn from(error: WorkspaceIoError) -> Self {
        Self::Workspace(error)
    }
}

impl std::fmt::Display for TextPatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace(error) => write!(formatter, "{error}"),
            Self::EmptyExpected { index } => write!(formatter, "patch hunk {index} has empty expected context"),
            Self::ContextMissing { index } => write!(formatter, "patch hunk {index} context was not found"),
            Self::ContextAmbiguous { index, matches } => write!(formatter, "patch hunk {index} matched {matches} locations"),
        }
    }
}

impl std::error::Error for TextPatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_context_is_rejected() {
        let root = std::env::current_dir().unwrap();
        let workspace = WorkspaceIo::open(root).unwrap();
        let result = TextPatchPlan::prepare(
            &workspace,
            "Cargo.toml",
            &[TextHunk {
                expected: "version".into(),
                replacement: "release".into(),
            }],
        );
        assert!(matches!(result, Err(TextPatchError::ContextAmbiguous { .. })));
    }
}
