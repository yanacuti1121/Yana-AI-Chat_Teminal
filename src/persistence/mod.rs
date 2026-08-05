// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use crate::{
    atlas::{Atlas, AtlasError},
    memory::{Memory, MemoryError},
};

#[derive(Debug, Clone)]
pub struct ProjectStores {
    root: PathBuf,
    state_dir: PathBuf,
}

impl ProjectStores {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let state_dir = root.join(".yana").join("state");
        Self { root, state_dir }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn memory_path(&self) -> PathBuf {
        self.state_dir.join("memory.json")
    }

    pub fn atlas_path(&self) -> PathBuf {
        self.state_dir.join("atlas.json")
    }

    pub fn open_memory(&self, capacity: usize) -> Result<Memory, MemoryError> {
        Memory::open(self.memory_path(), capacity)
    }

    pub fn open_atlas(&self) -> Result<Atlas, AtlasError> {
        Atlas::open(self.atlas_path())
    }

    pub fn flush(&self, memory: &Memory, atlas: &Atlas) -> Result<(), PersistenceError> {
        memory.save_to(&self.memory_path())?;
        atlas.save_to(&self.atlas_path())?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum PersistenceError {
    Memory(MemoryError),
    Atlas(AtlasError),
}

impl From<MemoryError> for PersistenceError {
    fn from(error: MemoryError) -> Self {
        Self::Memory(error)
    }
}

impl From<AtlasError> for PersistenceError {
    fn from(error: AtlasError) -> Self {
        Self::Atlas(error)
    }
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory(error) => write!(formatter, "memory persistence failed: {error}"),
            Self::Atlas(error) => write!(formatter, "atlas persistence failed: {error}"),
        }
    }
}

impl std::error::Error for PersistenceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_project_local_state_paths() {
        let stores = ProjectStores::new("/tmp/yana-project");
        assert!(stores.memory_path().ends_with(".yana/state/memory.json"));
        assert!(stores.atlas_path().ends_with(".yana/state/atlas.json"));
    }
}
