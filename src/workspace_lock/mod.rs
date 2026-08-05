// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct WorkspaceLocks {
    locked: BTreeSet<PathBuf>,
}

impl WorkspaceLocks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&mut self, path: impl Into<PathBuf>) -> Result<PathLease, LockError> {
        let path = path.into();
        if !self.locked.insert(path.clone()) {
            return Err(LockError::AlreadyLocked(path));
        }
        Ok(PathLease { path })
    }

    pub fn release(&mut self, lease: PathLease) -> bool {
        self.locked.remove(&lease.path)
    }

    pub fn is_locked(&self, path: impl AsRef<Path>) -> bool {
        self.locked.contains(path.as_ref())
    }

    pub fn len(&self) -> usize {
        self.locked.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathLease {
    path: PathBuf,
}

impl PathLease {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockError {
    AlreadyLocked(PathBuf),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyLocked(path) => write!(formatter, "workspace path is already locked: {}", path.display()),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_lock_is_rejected_until_release() {
        let mut locks = WorkspaceLocks::new();
        let lease = locks.acquire("src/lib.rs").unwrap();
        assert!(matches!(locks.acquire("src/lib.rs"), Err(LockError::AlreadyLocked(_))));
        assert!(locks.release(lease));
        assert!(locks.acquire("src/lib.rs").is_ok());
    }
}
