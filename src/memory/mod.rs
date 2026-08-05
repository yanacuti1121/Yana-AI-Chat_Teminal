// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::VecDeque,
    fs,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const MEMORY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    Working,
    Project,
    Decision,
    User,
    Tool,
    Pattern,
    Conversation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub kind: MemoryKind,
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub expires_at: Option<u64>,
}

impl MemoryEntry {
    pub fn new(kind: MemoryKind, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            kind,
            key: key.into(),
            value: value.into(),
            project: None,
            created_at: 0,
            expires_at: None,
        }
    }

    pub fn for_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    pub fn with_ttl(mut self, created_at: u64, ttl_seconds: u64) -> Self {
        self.created_at = created_at;
        self.expires_at = Some(created_at.saturating_add(ttl_seconds));
        self
    }

    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at.is_some_and(|deadline| deadline <= now)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryDocument {
    schema_version: u32,
    capacity: usize,
    entries: VecDeque<MemoryEntry>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    capacity: usize,
    entries: VecDeque<MemoryEntry>,
    store_path: Option<PathBuf>,
}

impl Memory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: VecDeque::new(),
            store_path: None,
        }
    }

    pub fn open(path: impl Into<PathBuf>, capacity: usize) -> Result<Self, MemoryError> {
        let path = path.into();
        if !path.exists() {
            let mut memory = Self::with_capacity(capacity);
            memory.store_path = Some(path);
            return Ok(memory);
        }

        let bytes = fs::read(&path)?;
        let document: MemoryDocument = serde_json::from_slice(&bytes)?;
        if document.schema_version != MEMORY_SCHEMA_VERSION {
            return Err(MemoryError::UnsupportedSchema(document.schema_version));
        }

        let mut memory = Self {
            capacity: capacity.max(document.capacity).max(1),
            entries: document.entries,
            store_path: Some(path),
        };
        memory.enforce_capacity();
        Ok(memory)
    }

    pub fn remember(&mut self, entry: MemoryEntry) {
        if let Some(position) = self.entries.iter().position(|current| {
            current.kind == entry.kind
                && current.key == entry.key
                && current.project == entry.project
        }) {
            self.entries.remove(position);
        }

        self.entries.push_back(entry);
        self.enforce_capacity();
    }

    pub fn recall(&self, kind: MemoryKind, key: &str) -> Option<&str> {
        self.recall_for_project(kind, key, None)
    }

    pub fn recall_for_project(
        &self,
        kind: MemoryKind,
        key: &str,
        project: Option<&str>,
    ) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|entry| {
                entry.kind == kind
                    && entry.key == key
                    && entry.project.as_deref() == project
            })
            .map(|entry| entry.value.as_str())
    }

    pub fn entries(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.iter()
    }

    pub fn project_entries<'a>(
        &'a self,
        project: &'a str,
    ) -> impl Iterator<Item = &'a MemoryEntry> {
        self.entries
            .iter()
            .filter(move |entry| entry.project.as_deref() == Some(project))
    }

    pub fn prune_expired(&mut self, now: u64) -> usize {
        let before = self.entries.len();
        self.entries.retain(|entry| !entry.is_expired(now));
        before - self.entries.len()
    }

    pub fn save(&self) -> Result<(), MemoryError> {
        let path = self
            .store_path
            .as_deref()
            .ok_or(MemoryError::NoStoreConfigured)?;
        self.save_to(path)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), MemoryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let document = MemoryDocument {
            schema_version: MEMORY_SCHEMA_VERSION,
            capacity: self.capacity,
            entries: self.entries.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&document)?;
        atomic_write(path, &bytes)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn enforce_capacity(&mut self) {
        while self.entries.len() > self.capacity {
            self.entries.pop_front();
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::with_capacity(256)
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)?;
    fs::rename(temp, path)
}

#[derive(Debug)]
pub enum MemoryError {
    Io(io::Error),
    Json(serde_json::Error),
    UnsupportedSchema(u32),
    NoStoreConfigured,
}

impl From<io::Error> for MemoryError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for MemoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "memory I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "memory document is invalid: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported memory schema version: {version}")
            }
            Self::NoStoreConfigured => write!(formatter, "no persistent memory store configured"),
        }
    }
}

impl std::error::Error for MemoryError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "yana-memory-{}-{name}.json",
            std::process::id()
        ))
    }

    #[test]
    fn replaces_memory_with_same_identity() {
        let mut memory = Memory::with_capacity(4);
        memory.remember(MemoryEntry::new(MemoryKind::Decision, "sidebar", "avoid"));
        memory.remember(MemoryEntry::new(
            MemoryKind::Decision,
            "sidebar",
            "overlay-only",
        ));

        assert_eq!(memory.len(), 1);
        assert_eq!(
            memory.recall(MemoryKind::Decision, "sidebar"),
            Some("overlay-only")
        );
    }

    #[test]
    fn project_memory_is_isolated() {
        let mut memory = Memory::default();
        memory.remember(
            MemoryEntry::new(MemoryKind::Project, "theme", "Sky Lake")
                .for_project("yana"),
        );
        assert_eq!(
            memory.recall_for_project(MemoryKind::Project, "theme", Some("yana")),
            Some("Sky Lake")
        );
        assert_eq!(
            memory.recall_for_project(MemoryKind::Project, "theme", Some("other")),
            None
        );
    }

    #[test]
    fn persists_and_restores_entries() {
        let path = temp_path("restore");
        let mut memory = Memory::open(&path, 8).unwrap();
        memory.remember(MemoryEntry::new(
            MemoryKind::Decision,
            "sidebar",
            "overlay-only",
        ));
        memory.save().unwrap();

        let restored = Memory::open(&path, 8).unwrap();
        assert_eq!(
            restored.recall(MemoryKind::Decision, "sidebar"),
            Some("overlay-only")
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn prunes_expired_working_memory() {
        let mut memory = Memory::default();
        memory.remember(
            MemoryEntry::new(MemoryKind::Working, "scope", "src/ui")
                .with_ttl(100, 10),
        );
        assert_eq!(memory.prune_expired(111), 1);
        assert!(memory.is_empty());
    }
}
