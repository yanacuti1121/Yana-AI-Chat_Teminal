// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseTarget {
    MacOsArm64,
    MacOsX64,
    LinuxX64,
    LinuxArm64,
    WindowsX64,
}

impl ReleaseTarget {
    pub const ALL: [Self; 5] = [
        Self::MacOsArm64,
        Self::MacOsX64,
        Self::LinuxX64,
        Self::LinuxArm64,
        Self::WindowsX64,
    ];

    pub fn rust_triple(self) -> &'static str {
        match self {
            Self::MacOsArm64 => "aarch64-apple-darwin",
            Self::MacOsX64 => "x86_64-apple-darwin",
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }

    pub fn archive_extension(self) -> &'static str {
        match self {
            Self::WindowsX64 => "zip",
            _ => "tar.gz",
        }
    }

    pub fn executable_name(self) -> &'static str {
        match self {
            Self::WindowsX64 => "yana-terminal.exe",
            _ => "yana-terminal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseManifest {
    pub version: String,
    pub target: ReleaseTarget,
    pub artifact_name: String,
    pub executable_name: String,
}

impl ReleaseManifest {
    pub fn build(version: impl Into<String>, target: ReleaseTarget) -> Result<Self, ReleaseError> {
        let version = version.into();
        validate_version(&version)?;
        let artifact_name = format!(
            "yana-terminal-v{version}-{}.{}",
            target.rust_triple(),
            target.archive_extension()
        );
        Ok(Self {
            version,
            target,
            artifact_name,
            executable_name: target.executable_name().to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseLayout {
    pub root: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub diagnostics_dir: PathBuf,
}

impl ReleaseLayout {
    pub fn project_local(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let yana = root.join(".yana");
        Self {
            root,
            config_dir: yana.join("config"),
            state_dir: yana.join("state"),
            diagnostics_dir: yana.join("diagnostics"),
        }
    }

    pub fn ensure_within_root(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
    }
}

fn validate_version(version: &str) -> Result<(), ReleaseError> {
    let mut parts = version.split('.');
    let valid = (0..3).all(|_| {
        parts
            .next()
            .map(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false)
    }) && parts.next().is_none();
    if valid {
        Ok(())
    } else {
        Err(ReleaseError::InvalidVersion(version.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseError {
    InvalidVersion(String),
}

impl std::fmt::Display for ReleaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVersion(version) => write!(formatter, "invalid release version: {version}"),
        }
    }
}

impl std::error::Error for ReleaseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_platform_specific_artifact_name() {
        let manifest = ReleaseManifest::build("1.0.0", ReleaseTarget::MacOsArm64).unwrap();
        assert_eq!(
            manifest.artifact_name,
            "yana-terminal-v1.0.0-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn rejects_non_semver_release_version() {
        assert!(ReleaseManifest::build("v1", ReleaseTarget::LinuxX64).is_err());
    }
}
