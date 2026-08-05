// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandDecision {
    Allow,
    RequireApproval(String),
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    allowed_programs: BTreeSet<String>,
    denied_fragments: Vec<String>,
    max_timeout_secs: u64,
}

impl SandboxPolicy {
    pub fn conservative() -> Self {
        Self {
            allowed_programs: ["cargo", "git", "rg", "grep"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            denied_fragments: vec![
                "--force".into(),
                "reset --hard".into(),
                "clean -f".into(),
                "push".into(),
                "publish".into(),
                "rm -rf".into(),
            ],
            max_timeout_secs: 120,
        }
    }

    pub fn evaluate(&self, command: &CommandSpec) -> CommandDecision {
        if !self.allowed_programs.contains(&command.program) {
            return CommandDecision::Deny(format!(
                "program is not on the sandbox allowlist: {}",
                command.program
            ));
        }
        if command.timeout_secs == 0 || command.timeout_secs > self.max_timeout_secs {
            return CommandDecision::Deny(format!(
                "timeout must be between 1 and {} seconds",
                self.max_timeout_secs
            ));
        }

        let joined = format!("{} {}", command.program, command.args.join(" "));
        if let Some(fragment) = self
            .denied_fragments
            .iter()
            .find(|fragment| joined.contains(fragment.as_str()))
        {
            return CommandDecision::Deny(format!(
                "command matches denied operation: {fragment}"
            ));
        }

        match command.program.as_str() {
            "cargo" if command.args.first().is_some_and(|arg| arg == "test") => {
                CommandDecision::Allow
            }
            "git" if command.args.first().is_some_and(|arg| arg == "status" || arg == "diff") => {
                CommandDecision::Allow
            }
            _ => CommandDecision::RequireApproval("command has side effects or broad scope".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_read_only_git_status() {
        let policy = SandboxPolicy::conservative();
        let decision = policy.evaluate(&CommandSpec {
            program: "git".into(),
            args: vec!["status".into(), "--short".into()],
            timeout_secs: 10,
        });
        assert_eq!(decision, CommandDecision::Allow);
    }

    #[test]
    fn denies_force_push() {
        let policy = SandboxPolicy::conservative();
        let decision = policy.evaluate(&CommandSpec {
            program: "git".into(),
            args: vec!["push".into(), "--force".into()],
            timeout_secs: 10,
        });
        assert!(matches!(decision, CommandDecision::Deny(_)));
    }
}
