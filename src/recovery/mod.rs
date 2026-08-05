// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState { Idle, Running, AwaitingApproval, Verifying, Failed }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub schema_version: u32,
    pub session_id: String,
    pub task_id: Option<String>,
    pub state: RecoveryState,
    pub pending_receipt_id: Option<String>,
    pub touched_files: Vec<PathBuf>,
    pub updated_at_ms: u64,
}

pub struct RecoveryStore { path: PathBuf }

impl RecoveryStore {
    pub fn new(state_dir: impl AsRef<Path>) -> Self {
        Self { path: state_dir.as_ref().join("recovery.json") }
    }

    pub fn save(&self, snapshot: &SessionSnapshot) -> Result<(), RecoveryError> {
        if let Some(parent) = self.path.parent() { fs::create_dir_all(parent)?; }
        let tmp = self.path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(snapshot)?;
        fs::write(&tmp, bytes)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Option<SessionSnapshot>, RecoveryError> {
        match fs::read(&self.path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn clear(&self) -> Result<(), RecoveryError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[derive(Debug)]
pub enum RecoveryError { Io(io::Error), Json(serde_json::Error) }
impl From<io::Error> for RecoveryError { fn from(v: io::Error) -> Self { Self::Io(v) } }
impl From<serde_json::Error> for RecoveryError { fn from(v: serde_json::Error) -> Self { Self::Json(v) } }
impl std::fmt::Display for RecoveryError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Self::Io(e) => write!(f, "recovery I/O error: {e}"), Self::Json(e) => write!(f, "recovery JSON error: {e}") } } }
impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_snapshot_is_none() {
        let dir = std::env::temp_dir().join(format!("yana-recovery-{}", std::process::id()));
        let store = RecoveryStore::new(&dir);
        assert!(store.load().unwrap().is_none());
    }
}
