// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    io,
    path::PathBuf,
};

use crate::{
    rollback::{RollbackError, RollbackSnapshot, RollbackStore},
    workspace_io::{WorkspaceIo, WorkspaceIoError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl RenamePlan {
    pub fn prepare(
        workspace: &WorkspaceIo,
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>,
    ) -> Result<Self, FilePlanError> {
        let from = from.into();
        let to = to.into();
        let source = workspace.resolve_existing_public(&from)?;
        if !source.is_file() {
            return Err(FilePlanError::SourceNotFile(from));
        }
        let destination = workspace.resolve_for_write(&to)?;
        if destination.exists() {
            return Err(FilePlanError::DestinationExists(to));
        }
        Ok(Self { from, to })
    }

    pub fn apply(
        self,
        workspace: &WorkspaceIo,
        rollback: &RollbackStore,
    ) -> Result<RollbackSnapshot, FilePlanError> {
        let snapshot = rollback.capture(workspace, &self.from)?;
        let source = workspace.resolve_existing_public(&self.from)?;
        let destination = workspace.resolve_for_write(&self.to)?;
        let parent = destination
            .parent()
            .ok_or_else(|| FilePlanError::MissingParent(destination.clone()))?;
        fs::create_dir_all(parent).map_err(FilePlanError::Io)?;
        fs::rename(source, destination).map_err(FilePlanError::Io)?;
        Ok(snapshot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlan {
    pub path: PathBuf,
}

impl DeletePlan {
    pub fn prepare(
        workspace: &WorkspaceIo,
        path: impl Into<PathBuf>,
    ) -> Result<Self, FilePlanError> {
        let path = path.into();
        let resolved = workspace.resolve_existing_public(&path)?;
        if !resolved.is_file() {
            return Err(FilePlanError::SourceNotFile(path));
        }
        Ok(Self { path })
    }

    pub fn apply(
        self,
        workspace: &WorkspaceIo,
        rollback: &RollbackStore,
    ) -> Result<RollbackSnapshot, FilePlanError> {
        let snapshot = rollback.capture(workspace, &self.path)?;
        let resolved = workspace.resolve_existing_public(&self.path)?;
        fs::remove_file(resolved).map_err(FilePlanError::Io)?;
        Ok(snapshot)
    }
}

#[derive(Debug)]
pub enum FilePlanError {
    Workspace(WorkspaceIoError),
    Rollback(RollbackError),
    Io(io::Error),
    SourceNotFile(PathBuf),
    DestinationExists(PathBuf),
    MissingParent(PathBuf),
}

impl From<WorkspaceIoError> for FilePlanError {
    fn from(error: WorkspaceIoError) -> Self {
        Self::Workspace(error)
    }
}

impl From<RollbackError> for FilePlanError {
    fn from(error: RollbackError) -> Self {
        Self::Rollback(error)
    }
}

impl std::fmt::Display for FilePlanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workspace(error) => write!(formatter, "{error}"),
            Self::Rollback(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "file plan I/O error: {error}"),
            Self::SourceNotFile(path) => write!(formatter, "source is not a file: {}", path.display()),
            Self::DestinationExists(path) => write!(formatter, "destination already exists: {}", path.display()),
            Self::MissingParent(path) => write!(formatter, "destination has no parent: {}", path.display()),
        }
    }
}

impl std::error::Error for FilePlanError {}
