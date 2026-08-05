// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorSeverity {
    Healthy,
    Notice,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: String,
    pub severity: DoctorSeverity,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorSnapshot {
    pub checks: Vec<DoctorCheck>,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCommandPlan {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_directory: PathBuf,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct CoreDoctor {
    executable: PathBuf,
    core_root: PathBuf,
}

impl CoreDoctor {
    pub fn new(executable: PathBuf, core_root: PathBuf) -> Self {
        Self { executable, core_root }
    }

    pub fn plan(&self) -> DoctorCommandPlan {
        DoctorCommandPlan {
            executable: self.executable.clone(),
            args: vec!["doctor".into()],
            working_directory: self.core_root.clone(),
            read_only: true,
        }
    }

    pub fn parse_output(&self, raw_output: impl Into<String>) -> DoctorSnapshot {
        let raw_output = raw_output.into();
        let checks = raw_output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| DoctorCheck {
                name: "yana doctor".into(),
                severity: classify(line),
                detail: line.trim().to_owned(),
            })
            .collect();
        DoctorSnapshot { checks, raw_output }
    }
}

fn classify(line: &str) -> DoctorSeverity {
    let normalized = line.to_ascii_lowercase();
    if normalized.contains("critical") || normalized.contains("fatal") || normalized.contains("failed") {
        DoctorSeverity::Critical
    } else if normalized.contains("warning") || normalized.contains("warn") {
        DoctorSeverity::Warning
    } else if normalized.contains("notice") || normalized.contains("stale") {
        DoctorSeverity::Notice
    } else {
        DoctorSeverity::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_is_read_only() {
        let doctor = CoreDoctor::new("bin/yana".into(), ".".into());
        let plan = doctor.plan();
        assert!(plan.read_only);
        assert_eq!(plan.args, vec!["doctor"]);
    }

    #[test]
    fn classifies_failed_line_as_critical() {
        assert_eq!(classify("audit failed"), DoctorSeverity::Critical);
    }
}
