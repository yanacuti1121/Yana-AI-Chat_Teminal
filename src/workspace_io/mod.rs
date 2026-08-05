// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    io,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct WorkspaceIo {
    root: PathBuf,
    max_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub line: usize,
    pub preview: String,
}

impl WorkspaceIo {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, WorkspaceIoError> {
        let root = fs::canonicalize(root.into()).map_err(WorkspaceIoError::Io)?;
        if !root.is_dir() {
            return Err(WorkspaceIoError::RootIsNotDirectory(root));
        }
        Ok(Self {
            root,
            max_file_bytes: 2 * 1024 * 1024,
        })
    }

    pub fn with_max_file_bytes(mut self, max_file_bytes: u64) -> Self {
        self.max_file_bytes = max_file_bytes.max(1);
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn read_text(&self, relative: impl AsRef<Path>) -> Result<String, WorkspaceIoError> {
        let path = self.resolve_existing_public(relative.as_ref())?;
        let metadata = fs::metadata(&path).map_err(WorkspaceIoError::Io)?;
        if !metadata.is_file() {
            return Err(WorkspaceIoError::NotAFile(path));
        }
        if metadata.len() > self.max_file_bytes {
            return Err(WorkspaceIoError::FileTooLarge {
                path,
                bytes: metadata.len(),
                limit: self.max_file_bytes,
            });
        }
        fs::read_to_string(path).map_err(WorkspaceIoError::Io)
    }

    pub fn search_text(
        &self,
        relative: impl AsRef<Path>,
        needle: &str,
        max_hits: usize,
    ) -> Result<Vec<SearchHit>, WorkspaceIoError> {
        if needle.is_empty() || max_hits == 0 {
            return Ok(Vec::new());
        }
        let content = self.read_text(relative.as_ref())?;
        let path = relative.as_ref().to_path_buf();
        Ok(content
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(needle))
            .take(max_hits)
            .map(|(index, line)| SearchHit {
                path: path.clone(),
                line: index + 1,
                preview: line.trim().to_owned(),
            })
            .collect())
    }

    pub fn resolve_for_write(&self, relative: impl AsRef<Path>) -> Result<PathBuf, WorkspaceIoError> {
        let relative = validate_relative(relative.as_ref())?;
        let candidate = self.root.join(relative);
        let parent = candidate
            .parent()
            .ok_or_else(|| WorkspaceIoError::EscapesWorkspace(candidate.clone()))?;
        let canonical_parent = fs::canonicalize(parent).map_err(WorkspaceIoError::Io)?;
        if !canonical_parent.starts_with(&self.root) {
            return Err(WorkspaceIoError::EscapesWorkspace(candidate));
        }
        Ok(candidate)
    }

    pub fn resolve_existing_public(
        &self,
        relative: impl AsRef<Path>,
    ) -> Result<PathBuf, WorkspaceIoError> {
        let relative = validate_relative(relative.as_ref())?;
        let canonical = fs::canonicalize(self.root.join(relative)).map_err(WorkspaceIoError::Io)?;
        if !canonical.starts_with(&self.root) {
            return Err(WorkspaceIoError::EscapesWorkspace(canonical));
        }
        Ok(canonical)
    }
}

fn validate_relative(path: &Path) -> Result<&Path, WorkspaceIoError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(WorkspaceIoError::InvalidRelativePath(path.to_path_buf()));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(WorkspaceIoError::InvalidRelativePath(path.to_path_buf()));
    }
    Ok(path)
}

#[derive(Debug)]
pub enum WorkspaceIoError {
    Io(io::Error),
    RootIsNotDirectory(PathBuf),
    InvalidRelativePath(PathBuf),
    EscapesWorkspace(PathBuf),
    NotAFile(PathBuf),
    FileTooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
}

impl std::fmt::Display for WorkspaceIoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "workspace I/O error: {error}"),
            Self::RootIsNotDirectory(path) => {
                write!(formatter, "workspace root is not a directory: {}", path.display())
            }
            Self::InvalidRelativePath(path) => {
                write!(formatter, "invalid workspace-relative path: {}", path.display())
            }
            Self::EscapesWorkspace(path) => {
                write!(formatter, "resolved path escapes workspace: {}", path.display())
            }
            Self::NotAFile(path) => write!(formatter, "path is not a file: {}", path.display()),
            Self::FileTooLarge { path, bytes, limit } => write!(
                formatter,
                "file is too large: {} ({bytes} bytes, limit {limit})",
                path.display()
            ),
        }
    }
}

impl std::error::Error for WorkspaceIoError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parent_traversal() {
        let result = validate_relative(Path::new("../secret"));
        assert!(matches!(
            result,
            Err(WorkspaceIoError::InvalidRelativePath(_))
        ));
    }

    #[test]
    fn empty_search_is_noop() {
        let root = std::env::current_dir().unwrap();
        let io = WorkspaceIo::open(root).unwrap();
        assert!(io.search_text("Cargo.toml", "", 10).unwrap().is_empty());
    }
}
