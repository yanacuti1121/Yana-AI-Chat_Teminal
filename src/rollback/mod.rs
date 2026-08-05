// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::workspace_io::{WorkspaceIo, WorkspaceIoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackSnapshot {
    pub original: PathBuf,
    pub snapshot: PathBuf,
    pub existed: bool,
}

#[derive(Debug, Clone)]
pub struct RollbackStore {
    root: PathBuf,
}

impl RollbackStore {
    pub fn open(workspace: &WorkspaceIo) -> Result<Self, RollbackError> {
        let root = workspace.root().join(".yana").join("rollback");
        fs::create_dir_all(&root).map_err(RollbackError::Io)?;
        Ok(Self { root })
    }

    pub fn capture(
        &self,
        workspace: &WorkspaceIo,
        relative: impl AsRef<Path>,
    ) -> Result<RollbackSnapshot, RollbackError> {
        let relative = relative.as_ref();
        let destination = workspace.resolve_for_write(relative)?;
        let existed = destination.is_file();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| RollbackError::Clock)?
            .as_nanos();
        let encoded = relative
            .to_string_lossy()
            .replace(['/', '\\'], "__");
        let snapshot = self.root.join(format!("{stamp}-{encoded}.bak"));

        if existed {
            fs::copy(&destination, &snapshot).map_err(RollbackError::Io)?;
        } else {
            fs::write(&snapshot, []).map_err(RollbackError::Io)?;
        }

        Ok(RollbackSnapshot {
            original: relative.to_path_buf(),
            snapshot,
            existed,
        })
    }

    pub fn restore(
        &self,
        workspace: &WorkspaceIo,
        snapshot: &RollbackSnapshot,
    ) -> Result<(), RollbackError> {
        let destination = workspace.resolve_for_write(&snapshot.original)?;
        if snapshot.existed {
            let parent = destination
                .parent()
                .ok_or_else(|| RollbackError::MissingParent(destination.clone()))?;
            fs::create_dir_all(parent).map_err(RollbackError::Io)?;
            fs::copy(&snapshot.snapshot, &destination).map_err(RollbackError::Io)?;
        } else if destination.exists() {
            fs::remove_file(destination).map_err(RollbackError::Io)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RollbackError {
    Workspace(WorkspaceIoError),
    Io(io::Error),
    MissingParent(PathBuf),
    Clock,
}

impl From<WorkspaceIoError> for RollbackError {
    fn from(error: WorkspaceIoError) -> Self {
        Self::Workspace(error)
    }
}

impl std::fmt::Display for RollbackError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "rollback I/O error: {error}"),
            Self::MissingParent(path) => write!(formatter, "rollback target has no parent: {}", path.display()),
            Self::Clock => write!(formatter, "system clock is before the Unix epoch"),
        }
    }
}

impl std::error::Error for RollbackError {}
