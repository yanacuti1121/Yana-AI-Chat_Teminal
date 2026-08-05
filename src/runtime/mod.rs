// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Idle,
    Background,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Running,
    Cancelling,
    Cancelled,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTask {
    pub id: TaskId,
    pub session_id: SessionId,
    pub name: String,
    pub priority: TaskPriority,
    pub state: TaskState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSession {
    pub id: SessionId,
    pub workspace: String,
    pub provider: String,
    pub state: SessionState,
    pub tasks: Vec<TaskId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvent {
    SessionStarted(SessionId),
    SessionPaused(SessionId),
    SessionResumed(SessionId),
    SessionStopped(SessionId),
    TaskQueued(TaskId),
    TaskStarted(TaskId),
    TaskCancelled(TaskId),
    TaskCompleted(TaskId),
    TaskFailed(TaskId),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeMetrics {
    pub sessions: usize,
    pub queued_tasks: usize,
    pub running_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub cancelled_tasks: usize,
}

#[derive(Debug, Default)]
pub struct AgentRuntime {
    next_session_id: u64,
    next_task_id: u64,
    sessions: BTreeMap<SessionId, RuntimeSession>,
    tasks: BTreeMap<TaskId, RuntimeTask>,
    queue: VecDeque<TaskId>,
    events: VecDeque<RuntimeEvent>,
}

impl AgentRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_session(
        &mut self,
        workspace: impl Into<String>,
        provider: impl Into<String>,
    ) -> SessionId {
        self.next_session_id += 1;
        let id = SessionId(self.next_session_id);
        self.sessions.insert(
            id,
            RuntimeSession {
                id,
                workspace: workspace.into(),
                provider: provider.into(),
                state: SessionState::Running,
                tasks: Vec::new(),
            },
        );
        self.events.push_back(RuntimeEvent::SessionStarted(id));
        id
    }

    pub fn pause_session(&mut self, id: SessionId) -> Result<(), RuntimeError> {
        let session = self.session_mut(id)?;
        match session.state {
            SessionState::Running => {
                session.state = SessionState::Paused;
                self.events.push_back(RuntimeEvent::SessionPaused(id));
                Ok(())
            }
            SessionState::Paused => Ok(()),
            SessionState::Stopped => Err(RuntimeError::InvalidSessionTransition),
        }
    }

    pub fn resume_session(&mut self, id: SessionId) -> Result<(), RuntimeError> {
        let session = self.session_mut(id)?;
        match session.state {
            SessionState::Paused => {
                session.state = SessionState::Running;
                self.events.push_back(RuntimeEvent::SessionResumed(id));
                Ok(())
            }
            SessionState::Running => Ok(()),
            SessionState::Stopped => Err(RuntimeError::InvalidSessionTransition),
        }
    }

    pub fn stop_session(&mut self, id: SessionId) -> Result<(), RuntimeError> {
        let task_ids = {
            let session = self.session_mut(id)?;
            session.state = SessionState::Stopped;
            session.tasks.clone()
        };
        for task_id in task_ids {
            let _ = self.cancel_task(task_id);
        }
        self.events.push_back(RuntimeEvent::SessionStopped(id));
        Ok(())
    }

    pub fn submit_task(
        &mut self,
        session_id: SessionId,
        name: impl Into<String>,
        priority: TaskPriority,
    ) -> Result<TaskId, RuntimeError> {
        let session = self.session_mut(session_id)?;
        if session.state == SessionState::Stopped {
            return Err(RuntimeError::SessionStopped);
        }

        self.next_task_id += 1;
        let id = TaskId(self.next_task_id);
        session.tasks.push(id);
        self.tasks.insert(
            id,
            RuntimeTask {
                id,
                session_id,
                name: name.into(),
                priority,
                state: TaskState::Queued,
            },
        );
        self.queue.push_back(id);
        self.reorder_queue();
        self.events.push_back(RuntimeEvent::TaskQueued(id));
        Ok(id)
    }

    pub fn next_runnable(&mut self) -> Option<TaskId> {
        let index = self.queue.iter().position(|task_id| {
            self.tasks
                .get(task_id)
                .and_then(|task| self.sessions.get(&task.session_id))
                .is_some_and(|session| session.state == SessionState::Running)
        })?;
        let task_id = self.queue.remove(index)?;
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.state = TaskState::Running;
        }
        self.events.push_back(RuntimeEvent::TaskStarted(task_id));
        Some(task_id)
    }

    pub fn complete_task(&mut self, id: TaskId) -> Result<(), RuntimeError> {
        let task = self.task_mut(id)?;
        if task.state != TaskState::Running {
            return Err(RuntimeError::InvalidTaskTransition);
        }
        task.state = TaskState::Completed;
        self.events.push_back(RuntimeEvent::TaskCompleted(id));
        Ok(())
    }

    pub fn fail_task(&mut self, id: TaskId) -> Result<(), RuntimeError> {
        let task = self.task_mut(id)?;
        if task.state != TaskState::Running {
            return Err(RuntimeError::InvalidTaskTransition);
        }
        task.state = TaskState::Failed;
        self.events.push_back(RuntimeEvent::TaskFailed(id));
        Ok(())
    }

    pub fn cancel_task(&mut self, id: TaskId) -> Result<(), RuntimeError> {
        let task = self.task_mut(id)?;
        match task.state {
            TaskState::Queued | TaskState::Running | TaskState::Cancelling => {
                task.state = TaskState::Cancelled;
                self.queue.retain(|queued| *queued != id);
                self.events.push_back(RuntimeEvent::TaskCancelled(id));
                Ok(())
            }
            TaskState::Cancelled => Ok(()),
            TaskState::Completed | TaskState::Failed => Err(RuntimeError::InvalidTaskTransition),
        }
    }

    pub fn session(&self, id: SessionId) -> Option<&RuntimeSession> {
        self.sessions.get(&id)
    }

    pub fn task(&self, id: TaskId) -> Option<&RuntimeTask> {
        self.tasks.get(&id)
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = RuntimeEvent> + '_ {
        self.events.drain(..)
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        let mut metrics = RuntimeMetrics {
            sessions: self.sessions.len(),
            ..RuntimeMetrics::default()
        };
        for task in self.tasks.values() {
            match task.state {
                TaskState::Queued => metrics.queued_tasks += 1,
                TaskState::Running | TaskState::Cancelling => metrics.running_tasks += 1,
                TaskState::Completed => metrics.completed_tasks += 1,
                TaskState::Failed => metrics.failed_tasks += 1,
                TaskState::Cancelled => metrics.cancelled_tasks += 1,
            }
        }
        metrics
    }

    fn reorder_queue(&mut self) {
        let mut ids = self.queue.drain(..).collect::<Vec<_>>();
        ids.sort_by(|left, right| {
            self.tasks[right]
                .priority
                .cmp(&self.tasks[left].priority)
                .then_with(|| left.cmp(right))
        });
        self.queue.extend(ids);
    }

    fn session_mut(&mut self, id: SessionId) -> Result<&mut RuntimeSession, RuntimeError> {
        self.sessions.get_mut(&id).ok_or(RuntimeError::UnknownSession(id))
    }

    fn task_mut(&mut self, id: TaskId) -> Result<&mut RuntimeTask, RuntimeError> {
        self.tasks.get_mut(&id).ok_or(RuntimeError::UnknownTask(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    UnknownSession(SessionId),
    UnknownTask(TaskId),
    SessionStopped,
    InvalidSessionTransition,
    InvalidTaskTransition,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSession(id) => write!(formatter, "unknown runtime session: {}", id.0),
            Self::UnknownTask(id) => write!(formatter, "unknown runtime task: {}", id.0),
            Self::SessionStopped => write!(formatter, "runtime session is stopped"),
            Self::InvalidSessionTransition => write!(formatter, "invalid session state transition"),
            Self::InvalidTaskTransition => write!(formatter, "invalid task state transition"),
        }
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_priority_tasks_run_first() {
        let mut runtime = AgentRuntime::new();
        let session = runtime.start_session(".", "ollama");
        let low = runtime.submit_task(session, "index", TaskPriority::Background).unwrap();
        let high = runtime.submit_task(session, "verify", TaskPriority::Critical).unwrap();
        assert_eq!(runtime.next_runnable(), Some(high));
        assert_eq!(runtime.next_runnable(), Some(low));
    }

    #[test]
    fn paused_sessions_do_not_dispatch_tasks() {
        let mut runtime = AgentRuntime::new();
        let session = runtime.start_session(".", "ollama");
        runtime.submit_task(session, "index", TaskPriority::Normal).unwrap();
        runtime.pause_session(session).unwrap();
        assert_eq!(runtime.next_runnable(), None);
        runtime.resume_session(session).unwrap();
        assert!(runtime.next_runnable().is_some());
    }

    #[test]
    fn stopping_session_cancels_pending_tasks() {
        let mut runtime = AgentRuntime::new();
        let session = runtime.start_session(".", "ollama");
        let task = runtime.submit_task(session, "index", TaskPriority::Normal).unwrap();
        runtime.stop_session(session).unwrap();
        assert_eq!(runtime.task(task).unwrap().state, TaskState::Cancelled);
    }

    #[test]
    fn metrics_are_derived_from_task_state() {
        let mut runtime = AgentRuntime::new();
        let session = runtime.start_session(".", "ollama");
        let task = runtime.submit_task(session, "verify", TaskPriority::High).unwrap();
        runtime.next_runnable();
        runtime.complete_task(task).unwrap();
        let metrics = runtime.metrics();
        assert_eq!(metrics.sessions, 1);
        assert_eq!(metrics.completed_tasks, 1);
        assert_eq!(metrics.queued_tasks, 0);
    }
}
