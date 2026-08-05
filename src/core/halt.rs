// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltState {
    Clear,
    Halted { marker: PathBuf },
}

#[derive(Debug, Clone)]
pub struct HaltGuard {
    core_root: PathBuf,
}

impl HaltGuard {
    pub fn new(core_root: PathBuf) -> Self {
        Self { core_root }
    }

    pub fn inspect(&self, workspace: impl AsRef<Path>) -> HaltState {
        let workspace = workspace.as_ref();
        let candidates = [
            workspace.join("HALT.lock"),
            workspace.join(".yana/HALT.lock"),
            self.core_root.join("HALT.lock"),
            self.core_root.join(".claude/state/HALT.lock"),
            self.core_root.join(".yana/HALT.lock"),
        ];

        candidates
            .into_iter()
            .find(|candidate| candidate.is_file())
            .map(|marker| HaltState::Halted { marker })
            .unwrap_or(HaltState::Clear)
    }

    pub fn require_clear(&self, workspace: impl AsRef<Path>) -> Result<(), HaltState> {
        match self.inspect(workspace) {
            HaltState::Clear => Ok(()),
            halted => Err(halted),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_marker_is_clear() {
        let guard = HaltGuard::new(std::env::temp_dir().join("missing-yana-core"));
        assert_eq!(guard.inspect(std::env::temp_dir()), HaltState::Clear);
    }
}
