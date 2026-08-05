// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState { Prepared, Applying, Verifying, Committed, RolledBack, Failed }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionStep {
    pub path: PathBuf,
    pub rollback_id: String,
    pub applied: bool,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceTransaction {
    pub id: String,
    pub state: TransactionState,
    steps: Vec<TransactionStep>,
}

impl WorkspaceTransaction {
    pub fn new(id: impl Into<String>) -> Self { Self { id: id.into(), state: TransactionState::Prepared, steps: Vec::new() } }
    pub fn add_step(&mut self, path: PathBuf, rollback_id: impl Into<String>) -> Result<(), TransactionError> {
        if self.state != TransactionState::Prepared { return Err(TransactionError::InvalidState); }
        if self.steps.iter().any(|step| step.path == path) { return Err(TransactionError::DuplicatePath(path)); }
        self.steps.push(TransactionStep { path, rollback_id: rollback_id.into(), applied: false, verified: false });
        Ok(())
    }
    pub fn begin_apply(&mut self) -> Result<(), TransactionError> { self.transition(TransactionState::Prepared, TransactionState::Applying) }
    pub fn mark_applied(&mut self, path: &PathBuf) -> Result<(), TransactionError> {
        if self.state != TransactionState::Applying { return Err(TransactionError::InvalidState); }
        let step = self.steps.iter_mut().find(|step| &step.path == path).ok_or_else(|| TransactionError::UnknownPath(path.clone()))?;
        step.applied = true;
        Ok(())
    }
    pub fn begin_verify(&mut self) -> Result<(), TransactionError> {
        if !self.steps.iter().all(|step| step.applied) { return Err(TransactionError::IncompleteApply); }
        self.transition(TransactionState::Applying, TransactionState::Verifying)
    }
    pub fn mark_verified(&mut self, path: &PathBuf) -> Result<(), TransactionError> {
        if self.state != TransactionState::Verifying { return Err(TransactionError::InvalidState); }
        let step = self.steps.iter_mut().find(|step| &step.path == path).ok_or_else(|| TransactionError::UnknownPath(path.clone()))?;
        step.verified = true;
        Ok(())
    }
    pub fn commit(&mut self) -> Result<(), TransactionError> {
        if !self.steps.iter().all(|step| step.verified) { return Err(TransactionError::IncompleteVerification); }
        self.transition(TransactionState::Verifying, TransactionState::Committed)
    }
    pub fn rollback(&mut self) -> Result<Vec<String>, TransactionError> {
        if matches!(self.state, TransactionState::Committed | TransactionState::RolledBack) { return Err(TransactionError::InvalidState); }
        self.state = TransactionState::RolledBack;
        Ok(self.steps.iter().rev().filter(|step| step.applied).map(|step| step.rollback_id.clone()).collect())
    }
    fn transition(&mut self, from: TransactionState, to: TransactionState) -> Result<(), TransactionError> {
        if self.state != from { return Err(TransactionError::InvalidState); }
        self.state = to; Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransactionError { InvalidState, DuplicatePath(PathBuf), UnknownPath(PathBuf), IncompleteApply, IncompleteVerification }
impl std::fmt::Display for TransactionError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "workspace transaction error: {self:?}") } }
impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rollback_order_is_reverse_apply_order() {
        let mut tx = WorkspaceTransaction::new("tx");
        tx.add_step(PathBuf::from("a"), "ra").unwrap();
        tx.add_step(PathBuf::from("b"), "rb").unwrap();
        tx.begin_apply().unwrap();
        tx.mark_applied(&PathBuf::from("a")).unwrap();
        tx.mark_applied(&PathBuf::from("b")).unwrap();
        assert_eq!(tx.rollback().unwrap(), vec!["rb", "ra"]);
    }
}
