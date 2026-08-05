// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalStatus {
    Planned,
    Active,
    Blocked,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub status: GoalStatus,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub blocker: Option<String>,
}

impl Goal {
    pub fn progress_percent(&self) -> u8 {
        if self.total_tasks == 0 {
            return 0;
        }
        ((self.completed_tasks.min(self.total_tasks) * 100) / self.total_tasks) as u8
    }
}

#[derive(Debug, Default)]
pub struct GoalBoard {
    goals: BTreeMap<String, Goal>,
}

impl GoalBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, goal: Goal) {
        self.goals.insert(goal.id.clone(), goal);
    }

    pub fn advance(&mut self, id: &str, completed_tasks: usize) -> Result<(), GoalError> {
        let goal = self
            .goals
            .get_mut(id)
            .ok_or_else(|| GoalError::UnknownGoal(id.to_owned()))?;
        goal.completed_tasks = completed_tasks.min(goal.total_tasks);
        goal.status = if goal.completed_tasks == goal.total_tasks && goal.total_tasks > 0 {
            GoalStatus::Completed
        } else {
            GoalStatus::Active
        };
        goal.blocker = None;
        Ok(())
    }

    pub fn block(&mut self, id: &str, reason: impl Into<String>) -> Result<(), GoalError> {
        let goal = self
            .goals
            .get_mut(id)
            .ok_or_else(|| GoalError::UnknownGoal(id.to_owned()))?;
        goal.status = GoalStatus::Blocked;
        goal.blocker = Some(reason.into());
        Ok(())
    }

    pub fn active(&self) -> impl Iterator<Item = &Goal> {
        self.goals
            .values()
            .filter(|goal| matches!(goal.status, GoalStatus::Active | GoalStatus::Blocked))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalError {
    UnknownGoal(String),
}

impl std::fmt::Display for GoalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownGoal(id) => write!(formatter, "unknown goal: {id}"),
        }
    }
}

impl std::error::Error for GoalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_goal_when_all_tasks_finish() {
        let mut board = GoalBoard::new();
        board.upsert(Goal {
            id: "phase-4".into(),
            title: "Workspace intelligence".into(),
            status: GoalStatus::Planned,
            completed_tasks: 0,
            total_tasks: 4,
            blocker: None,
        });
        board.advance("phase-4", 4).unwrap();
        assert_eq!(board.goals["phase-4"].status, GoalStatus::Completed);
        assert_eq!(board.goals["phase-4"].progress_percent(), 100);
    }
}
