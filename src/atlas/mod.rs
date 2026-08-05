// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const ATLAS_SCHEMA_VERSION: u32 = 1;
const DEFAULT_MAX_FILES: usize = 20_000;
const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub path: String,
    pub symbols: BTreeMap<String, Symbol>,
    pub dependencies: BTreeSet<String>,
    pub tests: BTreeSet<String>,
    pub content_hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtlasDocument {
    schema_version: u32,
    modules: BTreeMap<String, ModuleRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct Atlas {
    modules: BTreeMap<String, ModuleRecord>,
    store_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndexLimits {
    pub max_files: usize,
    pub max_file_bytes: u64,
}

impl Default for IndexLimits {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_FILES,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexReport {
    pub visited_files: usize,
    pub indexed_files: usize,
    pub skipped_large_files: usize,
    pub skipped_non_utf8_files: usize,
}

impl Atlas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Self, AtlasError> {
        let path = path.into();
        if !path.exists() {
            return Ok(Self {
                modules: BTreeMap::new(),
                store_path: Some(path),
            });
        }

        let bytes = fs::read(&path)?;
        let document: AtlasDocument = serde_json::from_slice(&bytes)?;
        if document.schema_version != ATLAS_SCHEMA_VERSION {
            return Err(AtlasError::UnsupportedSchema(document.schema_version));
        }

        Ok(Self {
            modules: document.modules,
            store_path: Some(path),
        })
    }

    pub fn index_workspace(
        &mut self,
        root: &Path,
        limits: IndexLimits,
    ) -> Result<IndexReport, AtlasError> {
        let root = fs::canonicalize(root)?;
        let mut report = IndexReport::default();
        let mut queue = VecDeque::from([root.clone()]);
        let mut indexed = BTreeMap::new();

        while let Some(directory) = queue.pop_front() {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let path = entry.path();

                if file_type.is_symlink() {
                    continue;
                }
                if file_type.is_dir() {
                    if !is_ignored_directory(&path) {
                        queue.push_back(path);
                    }
                    continue;
                }
                if !file_type.is_file() || !is_supported_source(&path) {
                    continue;
                }

                report.visited_files += 1;
                if report.visited_files > limits.max_files {
                    return Err(AtlasError::FileLimitExceeded(limits.max_files));
                }

                let metadata = entry.metadata()?;
                if metadata.len() > limits.max_file_bytes {
                    report.skipped_large_files += 1;
                    continue;
                }

                let bytes = fs::read(&path)?;
                let Ok(source) = std::str::from_utf8(&bytes) else {
                    report.skipped_non_utf8_files += 1;
                    continue;
                };

                let relative = path
                    .strip_prefix(&root)
                    .map_err(|_| AtlasError::EscapedWorkspace(path.clone()))?;
                let module_path = normalize_path(relative);
                let record = parse_source(&module_path, source);
                indexed.insert(module_path, record);
                report.indexed_files += 1;
            }
        }

        self.modules = indexed;
        Ok(report)
    }

    pub fn record_symbol(&mut self, module: impl Into<String>, symbol: impl Into<String>) {
        let module = module.into();
        let symbol = symbol.into();
        self.modules
            .entry(module.clone())
            .or_insert_with(|| ModuleRecord {
                path: module,
                ..ModuleRecord::default()
            })
            .symbols
            .entry(symbol.clone())
            .or_insert(Symbol {
                name: symbol,
                kind: SymbolKind::Function,
                line: 0,
            });
    }

    pub fn symbols_in(&self, module: &str) -> impl Iterator<Item = &str> {
        self.modules
            .get(module)
            .into_iter()
            .flat_map(|record| record.symbols.keys().map(String::as_str))
    }

    pub fn find_symbol(&self, name: &str) -> Vec<(&str, &Symbol)> {
        self.modules
            .iter()
            .filter_map(|(path, module)| module.symbols.get(name).map(|symbol| (path.as_str(), symbol)))
            .collect()
    }

    pub fn dependencies_of(&self, module: &str) -> impl Iterator<Item = &str> {
        self.modules
            .get(module)
            .into_iter()
            .flat_map(|record| record.dependencies.iter().map(String::as_str))
    }

    pub fn affected_by(&self, dependency: &str) -> Vec<&str> {
        self.modules
            .iter()
            .filter(|(_, module)| module.dependencies.contains(dependency))
            .map(|(path, _)| path.as_str())
            .collect()
    }

    pub fn module(&self, path: &str) -> Option<&ModuleRecord> {
        self.modules.get(path)
    }

    pub fn modules(&self) -> impl Iterator<Item = &ModuleRecord> {
        self.modules.values()
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn symbol_count(&self) -> usize {
        self.modules.values().map(|module| module.symbols.len()).sum()
    }

    pub fn save(&self) -> Result<(), AtlasError> {
        let path = self
            .store_path
            .as_deref()
            .ok_or(AtlasError::NoStoreConfigured)?;
        self.save_to(path)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), AtlasError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let document = AtlasDocument {
            schema_version: ATLAS_SCHEMA_VERSION,
            modules: self.modules.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&document)?;
        atomic_write(path, &bytes)?;
        Ok(())
    }
}

fn parse_source(path: &str, source: &str) -> ModuleRecord {
    let mut record = ModuleRecord {
        path: path.to_owned(),
        content_hash: stable_hash(source.as_bytes()),
        ..ModuleRecord::default()
    };

    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        let line_number = index + 1;

        if let Some((kind, name)) = parse_symbol(line) {
            record.symbols.entry(name.clone()).or_insert(Symbol {
                name,
                kind,
                line: line_number,
            });
        }

        if let Some(dependency) = parse_dependency(line) {
            record.dependencies.insert(dependency);
        }

        if line.starts_with("#[test]") || line.contains("mod tests") {
            record.tests.insert(format!("{path}:{line_number}"));
        }
    }

    record
}

fn parse_symbol(line: &str) -> Option<(SymbolKind, String)> {
    let line = line
        .strip_prefix("pub ")
        .or_else(|| line.strip_prefix("pub(crate) "))
        .unwrap_or(line);

    for (prefix, kind) in [
        ("fn ", SymbolKind::Function),
        ("async fn ", SymbolKind::Function),
        ("struct ", SymbolKind::Struct),
        ("enum ", SymbolKind::Enum),
        ("trait ", SymbolKind::Trait),
        ("mod ", SymbolKind::Module),
        ("const ", SymbolKind::Constant),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .split(|character: char| {
                    character == '(' || character == '<' || character == '{' || character == ':' || character == ';' || character.is_whitespace()
                })
                .next()
                .unwrap_or_default()
                .trim();
            if !name.is_empty() {
                return Some((kind, name.to_owned()));
            }
        }
    }
    None
}

fn parse_dependency(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("use ")
        .or_else(|| line.strip_prefix("pub use "))?;
    let root = rest
        .trim_end_matches(';')
        .split("::")
        .next()
        .unwrap_or_default()
        .trim();
    (!root.is_empty()).then(|| root.to_owned())
}

fn is_supported_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "toml" | "md")
    )
}

fn is_ignored_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target" | "node_modules" | ".next" | "dist" | "build")
    )
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)?;
    fs::rename(temp, path)
}

#[derive(Debug)]
pub enum AtlasError {
    Io(io::Error),
    Json(serde_json::Error),
    UnsupportedSchema(u32),
    FileLimitExceeded(usize),
    EscapedWorkspace(PathBuf),
    NoStoreConfigured,
}

impl From<io::Error> for AtlasError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for AtlasError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for AtlasError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "atlas I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "atlas document is invalid: {error}"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported atlas schema version: {version}")
            }
            Self::FileLimitExceeded(limit) => write!(formatter, "atlas file limit exceeded: {limit}"),
            Self::EscapedWorkspace(path) => {
                write!(formatter, "indexed path escaped workspace: {}", path.display())
            }
            Self::NoStoreConfigured => write!(formatter, "no persistent atlas store configured"),
        }
    }
}

impl std::error::Error for AtlasError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_unique_symbols() {
        let mut atlas = Atlas::new();
        atlas.record_symbol("ui", "draw");
        atlas.record_symbol("ui", "draw");
        atlas.record_symbol("ui", "theme");

        assert_eq!(atlas.module_count(), 1);
        assert_eq!(atlas.symbol_count(), 2);
    }

    #[test]
    fn parses_symbols_dependencies_and_tests() {
        let record = parse_source(
            "src/ui/mod.rs",
            "use crate::theme::SkyLake;\npub struct App;\npub fn draw() {}\n#[test]\nfn renders() {}",
        );
        assert!(record.symbols.contains_key("App"));
        assert!(record.symbols.contains_key("draw"));
        assert!(record.dependencies.contains("crate"));
        assert_eq!(record.tests.len(), 1);
    }

    #[test]
    fn finds_reverse_dependencies() {
        let mut atlas = Atlas::new();
        let mut module = ModuleRecord {
            path: "src/ui.rs".into(),
            ..ModuleRecord::default()
        };
        module.dependencies.insert("theme".into());
        atlas.modules.insert(module.path.clone(), module);
        assert_eq!(atlas.affected_by("theme"), vec!["src/ui.rs"]);
    }
}
