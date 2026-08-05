// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io, path::PathBuf};

use crate::workspace_io::{WorkspaceIo, WorkspaceIoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchPreview {
    pub path: PathBuf,
    pub before_bytes: usize,
    pub after_bytes: usize,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritePlan {
    pub path: PathBuf,
    pub content: String,
    pub preview: PatchPreview,
}

impl WritePlan {
    pub fn prepare(
        workspace: &WorkspaceIo,
        relative: impl Into<PathBuf>,
        content: impl Into<String>,
    ) -> Result<Self, PatchError> {
        let relative = relative.into();
        let path = workspace.resolve_for_write(&relative)?;
        let content = content.into();
        let before = match workspace.read_text(&relative) {
            Ok(existing) => existing,
            Err(WorkspaceIoError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                String::new()
            }
            Err(error) => return Err(error.into()),
        };

        Ok(Self {
            path: relative,
            preview: PatchPreview {
                path,
                before_bytes: before.len(),
                after_bytes: content.len(),
                changed: before != content,
            },
            content,
        })
    }

    pub fn apply(self, workspace: &WorkspaceIo) -> Result<PatchPreview, PatchError> {
        let destination = workspace.resolve_for_write(&self.path)?;
        let parent = destination
            .parent()
            .ok_or_else(|| PatchError::MissingParent(destination.clone()))?;
        fs::create_dir_all(parent).map_err(PatchError::Io)?;

        let temporary = destination.with_extension("yana.tmp");
        fs::write(&temporary, self.content.as_bytes()).map_err(PatchError::Io)?;
        fs::rename(&temporary, &destination).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            PatchError::Io(error)
        })?;

        Ok(self.preview)
    }
}

#[derive(Debug)]
pub enum PatchError {
    Workspace(WorkspaceIoError),
    Io(io::Error),
    MissingParent(PathBuf),
}

impl From<WorkspaceIoError> for PatchError {
    fn from(error: WorkspaceIoError) -> Self {
        Self::Workspace(error)
    }
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "patch I/O error: {error}"),
            Self::MissingParent(path) => write!(formatter, "patch target has no parent: {}", path.display()),
        }
    }
}

impl std::error::Error for PatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_reports_no_change_for_same_content() {
        let root = std::env::current_dir().unwrap();
        let workspace = WorkspaceIo::open(root).unwrap();
        let current = workspace.read_text("Cargo.toml").unwrap();
        let plan = WritePlan::prepare(&workspace, "Cargo.toml", current).unwrap();
        assert!(!plan.preview.changed);
    }
}
