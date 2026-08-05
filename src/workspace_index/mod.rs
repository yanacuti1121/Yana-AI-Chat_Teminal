// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectKind {
    Rust,
    NextJs,
    Node,
    Python,
    Go,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WorkspaceCapability {
    Git,
    Tests,
    Benchmarks,
    ContinuousIntegration,
    Docker,
    Documentation,
    ReleasePipeline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    pub path: String,
    pub bytes: u64,
    pub modified_unix_ms: u64,
    pub content_hash: u64,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Constant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceIntent {
    pub primary_kind: ProjectKind,
    pub detected_kinds: BTreeSet<ProjectKind>,
    pub capabilities: BTreeSet<WorkspaceCapability>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceIndex {
    pub files: BTreeMap<String, FileRecord>,
    pub symbols: BTreeMap<String, Vec<SymbolRecord>>,
    pub dependencies: BTreeMap<String, BTreeSet<String>>,
    pub reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexPolicy {
    pub max_files: usize,
    pub max_file_bytes: u64,
}

impl Default for IndexPolicy {
    fn default() -> Self {
        Self {
            max_files: 50_000,
            max_file_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementalReport {
    pub added: usize,
    pub updated: usize,
    pub removed: usize,
    pub unchanged: usize,
}

impl WorkspaceIndex {
    pub fn build(root: &Path, policy: IndexPolicy) -> Result<(Self, WorkspaceIntent), IndexError> {
        let root = fs::canonicalize(root)?;
        let mut index = Self::default();
        let mut stack = vec![root.clone()];
        let mut count = 0usize;

        while let Some(directory) = stack.pop() {
            let mut entries = fs::read_dir(&directory)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|entry| entry.file_name());

            for entry in entries {
                let file_type = entry.file_type()?;
                let path = entry.path();

                if file_type.is_symlink() {
                    continue;
                }
                if file_type.is_dir() {
                    if !ignored_directory(&path) {
                        stack.push(path);
                    }
                    continue;
                }
                if !file_type.is_file() {
                    continue;
                }

                count += 1;
                if count > policy.max_files {
                    return Err(IndexError::FileLimitExceeded(policy.max_files));
                }

                let metadata = entry.metadata()?;
                if metadata.len() > policy.max_file_bytes {
                    continue;
                }

                let relative = path
                    .strip_prefix(&root)
                    .map_err(|_| IndexError::EscapedWorkspace(path.clone()))?;
                let normalized = normalize(relative);
                let bytes = fs::read(&path)?;
                let record = file_record(&normalized, &metadata, &bytes);

                if let Ok(source) = std::str::from_utf8(&bytes) {
                    index.index_source(&normalized, source);
                }
                index.files.insert(normalized, record);
            }
        }

        index.rebuild_reverse_dependencies();
        let intent = detect_intent(index.files.keys());
        Ok((index, intent))
    }

    pub fn refresh(
        &mut self,
        root: &Path,
        changed_paths: impl IntoIterator<Item = PathBuf>,
        max_file_bytes: u64,
    ) -> Result<IncrementalReport, IndexError> {
        let root = fs::canonicalize(root)?;
        let mut report = IncrementalReport::default();
        let mut changed = changed_paths.into_iter().collect::<Vec<_>>();
        changed.sort();
        changed.dedup();

        for relative in changed {
            if relative.is_absolute() || relative.components().any(|component| matches!(component, std::path::Component::ParentDir)) {
                return Err(IndexError::InvalidRelativePath(relative));
            }
            let normalized = normalize(&relative);
            let absolute = root.join(&relative);

            if !absolute.exists() {
                let existed = self.files.remove(&normalized).is_some();
                self.remove_path_indexes(&normalized);
                if existed {
                    report.removed += 1;
                }
                continue;
            }

            let canonical = fs::canonicalize(&absolute)?;
            if !canonical.starts_with(&root) {
                return Err(IndexError::EscapedWorkspace(canonical));
            }
            let metadata = fs::metadata(&canonical)?;
            if !metadata.is_file() || metadata.len() > max_file_bytes {
                continue;
            }
            let bytes = fs::read(&canonical)?;
            let next = file_record(&normalized, &metadata, &bytes);

            match self.files.get(&normalized) {
                Some(previous) if previous.content_hash == next.content_hash => {
                    report.unchanged += 1;
                    continue;
                }
                Some(_) => report.updated += 1,
                None => report.added += 1,
            }

            self.remove_path_indexes(&normalized);
            if let Ok(source) = std::str::from_utf8(&bytes) {
                self.index_source(&normalized, source);
            }
            self.files.insert(normalized, next);
        }

        self.rebuild_reverse_dependencies();
        Ok(report)
    }

    pub fn find_symbol(&self, name: &str) -> &[SymbolRecord] {
        self.symbols.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn references_to(&self, dependency: &str) -> impl Iterator<Item = &str> {
        self.reverse_dependencies
            .get(dependency)
            .into_iter()
            .flat_map(|paths| paths.iter().map(String::as_str))
    }

    fn index_source(&mut self, path: &str, source: &str) {
        let mut dependencies = BTreeSet::new();
        for (index, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if let Some((kind, name)) = parse_symbol(line) {
                self.symbols.entry(name.clone()).or_default().push(SymbolRecord {
                    name,
                    kind,
                    path: path.to_owned(),
                    line: index + 1,
                });
            }
            if let Some(dependency) = parse_dependency(line) {
                dependencies.insert(dependency);
            }
        }
        for records in self.symbols.values_mut() {
            records.sort_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)));
        }
        self.dependencies.insert(path.to_owned(), dependencies);
    }

    fn remove_path_indexes(&mut self, path: &str) {
        self.dependencies.remove(path);
        self.reverse_dependencies.remove(path);
        self.symbols.retain(|_, records| {
            records.retain(|record| record.path != path);
            !records.is_empty()
        });
    }

    fn rebuild_reverse_dependencies(&mut self) {
        self.reverse_dependencies.clear();
        for (path, dependencies) in &self.dependencies {
            for dependency in dependencies {
                self.reverse_dependencies
                    .entry(dependency.clone())
                    .or_default()
                    .insert(path.clone());
            }
        }
    }
}

fn detect_intent<'a>(paths: impl Iterator<Item = &'a String>) -> WorkspaceIntent {
    let set = paths.map(String::as_str).collect::<BTreeSet<_>>();
    let mut kinds = BTreeSet::new();
    let mut capabilities = BTreeSet::new();

    if set.contains("Cargo.toml") { kinds.insert(ProjectKind::Rust); }
    if set.contains("next.config.js") || set.contains("next.config.mjs") || set.contains("next.config.ts") { kinds.insert(ProjectKind::NextJs); }
    if set.contains("package.json") { kinds.insert(ProjectKind::Node); }
    if set.contains("pyproject.toml") || set.contains("requirements.txt") { kinds.insert(ProjectKind::Python); }
    if set.contains("go.mod") { kinds.insert(ProjectKind::Go); }

    if set.contains(".git/HEAD") { capabilities.insert(WorkspaceCapability::Git); }
    if set.iter().any(|path| path.starts_with("tests/") || path.contains("/tests/") || path.ends_with("_test.rs") || path.ends_with(".test.ts")) { capabilities.insert(WorkspaceCapability::Tests); }
    if set.iter().any(|path| path.starts_with("benches/") || path.contains("benchmark")) { capabilities.insert(WorkspaceCapability::Benchmarks); }
    if set.iter().any(|path| path.starts_with(".github/workflows/")) { capabilities.insert(WorkspaceCapability::ContinuousIntegration); }
    if set.contains("Dockerfile") || set.contains("docker-compose.yml") || set.contains("compose.yml") { capabilities.insert(WorkspaceCapability::Docker); }
    if set.contains("README.md") || set.iter().any(|path| path.starts_with("docs/")) { capabilities.insert(WorkspaceCapability::Documentation); }
    if set.iter().any(|path| path.contains("release") && path.starts_with(".github/workflows/")) { capabilities.insert(WorkspaceCapability::ReleasePipeline); }

    let primary_kind = match kinds.len() {
        0 => ProjectKind::Unknown,
        1 => *kinds.iter().next().unwrap_or(&ProjectKind::Unknown),
        _ => ProjectKind::Mixed,
    };

    WorkspaceIntent { primary_kind, detected_kinds: kinds, capabilities }
}

fn file_record(path: &str, metadata: &fs::Metadata, bytes: &[u8]) -> FileRecord {
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    FileRecord {
        path: path.to_owned(),
        bytes: metadata.len(),
        modified_unix_ms,
        content_hash: stable_hash(bytes),
        extension: Path::new(path).extension().and_then(|value| value.to_str()).map(str::to_owned),
    }
}

fn parse_symbol(line: &str) -> Option<(SymbolKind, String)> {
    let line = line.strip_prefix("pub ").or_else(|| line.strip_prefix("pub(crate) ")).unwrap_or(line);
    for (prefix, kind) in [
        ("async fn ", SymbolKind::Function), ("fn ", SymbolKind::Function),
        ("struct ", SymbolKind::Struct), ("enum ", SymbolKind::Enum),
        ("trait ", SymbolKind::Trait), ("mod ", SymbolKind::Module),
        ("const ", SymbolKind::Constant),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest.split(|c: char| c == '(' || c == '<' || c == '{' || c == ':' || c == ';' || c.is_whitespace()).next().unwrap_or_default();
            if !name.is_empty() { return Some((kind, name.to_owned())); }
        }
    }
    None
}

fn parse_dependency(line: &str) -> Option<String> {
    let rest = line.strip_prefix("use ").or_else(|| line.strip_prefix("pub use "))?;
    let root = rest.trim_end_matches(';').split("::").next().unwrap_or_default().trim();
    (!root.is_empty()).then(|| root.to_owned())
}

fn ignored_directory(path: &Path) -> bool {
    matches!(path.file_name().and_then(|name| name.to_str()), Some(".git" | "target" | "node_modules" | ".next" | "dist" | "build" | ".yana"))
}

fn normalize(path: &Path) -> String {
    path.components().map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes { hash ^= u64::from(*byte); hash = hash.wrapping_mul(0x100000001b3); }
    hash
}

#[derive(Debug)]
pub enum IndexError {
    Io(io::Error),
    FileLimitExceeded(usize),
    EscapedWorkspace(PathBuf),
    InvalidRelativePath(PathBuf),
}

impl From<io::Error> for IndexError { fn from(error: io::Error) -> Self { Self::Io(error) } }
impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "workspace index I/O failed: {error}"),
            Self::FileLimitExceeded(limit) => write!(f, "workspace index file limit exceeded: {limit}"),
            Self::EscapedWorkspace(path) => write!(f, "indexed path escaped workspace: {}", path.display()),
            Self::InvalidRelativePath(path) => write!(f, "invalid workspace-relative path: {}", path.display()),
        }
    }
}
impl std::error::Error for IndexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intent_detection_is_deterministic() {
        let paths = ["Cargo.toml".to_owned(), "README.md".to_owned(), ".github/workflows/release.yml".to_owned()];
        let intent = detect_intent(paths.iter());
        assert_eq!(intent.primary_kind, ProjectKind::Rust);
        assert!(intent.capabilities.contains(&WorkspaceCapability::Documentation));
        assert!(intent.capabilities.contains(&WorkspaceCapability::ReleasePipeline));
    }

    #[test]
    fn symbol_records_are_sorted() {
        let mut index = WorkspaceIndex::default();
        index.index_source("z.rs", "fn draw() {}");
        index.index_source("a.rs", "fn draw() {}");
        let records = index.find_symbol("draw");
        assert_eq!(records[0].path, "a.rs");
        assert_eq!(records[1].path, "z.rs");
    }

    #[test]
    fn reverse_dependencies_are_stable() {
        let mut index = WorkspaceIndex::default();
        index.index_source("ui.rs", "use theme::Palette;");
        index.rebuild_reverse_dependencies();
        assert_eq!(index.references_to("theme").collect::<Vec<_>>(), vec!["ui.rs"]);
    }
}
