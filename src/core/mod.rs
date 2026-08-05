// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

mod capability;
mod doctor;
mod halt;
mod receipt;

pub use capability::{Capability, CapabilityRegistry, CapabilityState};
pub use doctor::{CoreDoctor, DoctorCheck, DoctorSeverity, DoctorSnapshot};
pub use halt::{HaltGuard, HaltState};
pub use receipt::{CoreReceipt, ReceiptSink};

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct YanaCore {
    root: PathBuf,
    executable: PathBuf,
    capabilities: CapabilityRegistry,
}

impl YanaCore {
    pub fn discover(workspace: impl AsRef<Path>) -> Result<Self, CoreError> {
        let workspace = workspace.as_ref();
        let candidates = [workspace.to_path_buf(), workspace.join("Yana-AI")];

        for root in candidates {
            let executable = root.join("bin/yana");
            if executable.is_file() {
                let capabilities = CapabilityRegistry::detect(&root);
                return Ok(Self {
                    root,
                    executable,
                    capabilities,
                });
            }
        }

        Err(CoreError::NotFound(workspace.to_path_buf()))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn capabilities(&self) -> &CapabilityRegistry {
        &self.capabilities
    }

    pub fn halt_guard(&self) -> HaltGuard {
        HaltGuard::new(self.root.clone())
    }

    pub fn doctor(&self) -> CoreDoctor {
        CoreDoctor::new(self.executable.clone(), self.root.clone())
    }
}

#[derive(Debug)]
pub enum CoreError {
    NotFound(PathBuf),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(
                formatter,
                "Yana Core was not found in or below workspace: {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_core_is_explicit() {
        let missing = std::env::temp_dir().join("yana-core-does-not-exist");
        assert!(matches!(YanaCore::discover(missing), Err(CoreError::NotFound(_))));
    }
}
