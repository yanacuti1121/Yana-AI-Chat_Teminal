// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io, path::PathBuf};

use crate::{
    patch::{PatchError, PatchPreview, WritePlan},
    rollback::{RollbackError, RollbackSnapshot, RollbackStore},
    text_patch::{TextPatchError, TextPatchPlan},
    workspace_diff::DiffPreview,
    workspace_io::WorkspaceIo,
    workspace_lock::{LockError, WorkspaceLocks},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationReceipt {
    pub path: PathBuf,
    pub diff: DiffPreview,
    pub rollback: RollbackSnapshot,
}

pub struct WorkspaceMutator<'a> {
    workspace: &'a WorkspaceIo,
    rollback: &'a RollbackStore,
    locks: &'a mut WorkspaceLocks,
}

impl<'a> WorkspaceMutator<'a> {
    pub fn new(
        workspace: &'a WorkspaceIo,
        rollback: &'a RollbackStore,
        locks: &'a mut WorkspaceLocks,
    ) -> Self {
        Self {
            workspace,
            rollback,
            locks,
        }
    }

    pub fn apply_text_patch(
        &mut self,
        plan: TextPatchPlan,
    ) -> Result<MutationReceipt, WorkspaceMutationError> {
        let lease = self.locks.acquire(plan.path.clone())?;
        let snapshot = self.rollback.capture(self.workspace, &plan.path)?;
        let diff = DiffPreview::between(&plan.original, &plan.updated);
        let write = WritePlan::prepare(self.workspace, plan.path.clone(), plan.updated)?;
        let result = write.apply(self.workspace);
        self.locks.release(lease);
        result?;
        Ok(MutationReceipt {
            path: plan.path,
            diff,
            rollback: snapshot,
        })
    }

    pub fn restore(
        &mut self,
        snapshot: &RollbackSnapshot,
    ) -> Result<(), WorkspaceMutationError> {
        let lease = self.locks.acquire(snapshot.original.clone())?;
        let result = self.rollback.restore(self.workspace, snapshot);
        self.locks.release(lease);
        result?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum WorkspaceMutationError {
    Lock(LockError),
    Rollback(RollbackError),
    Patch(PatchError),
    TextPatch(TextPatchError),
    Io(io::Error),
}

impl From<LockError> for WorkspaceMutationError {
    fn from(error: LockError) -> Self {
        Self::Lock(error)
    }
}
impl From<RollbackError> for WorkspaceMutationError {
    fn from(error: RollbackError) -> Self {
        Self::Rollback(error)
    }
}
impl From<PatchError> for WorkspaceMutationError {
    fn from(error: PatchError) -> Self {
        Self::Patch(error)
    }
}
impl From<TextPatchError> for WorkspaceMutationError {
    fn from(error: TextPatchError) -> Self {
        Self::TextPatch(error)
    }
}

impl std::fmt::Display for WorkspaceMutationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lock(error) => write!(formatter, "{error}"),
            Self::Rollback(error) => write!(formatter, "{error}"),
            Self::Patch(error) => write!(formatter, "{error}"),
            Self::TextPatch(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "workspace mutation I/O error: {error}"),
        }
    }
}

impl std::error::Error for WorkspaceMutationError {}
