use nova_nexus::Intent;
use nova_planner::ActionGraph;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Created,
    Planning,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSession {
    pub id: String,
    pub input: String,
    pub state: TaskState,
    pub intent: Option<Intent>,
    pub plan: Option<ActionGraph>,
    pub current_node: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TaskError {
    #[error("task id cannot be empty")]
    EmptyTaskId,
    #[error("task input cannot be empty")]
    EmptyInput,
    #[error("task already exists: {0}")]
    DuplicateTask(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("invalid task state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: TaskState, to: TaskState },
    #[error("task requires an intent before planning")]
    MissingIntent,
    #[error("task requires a plan before execution")]
    MissingPlan,
}

impl TaskSession {
    pub fn new(id: impl Into<String>, input: impl Into<String>) -> Result<Self, TaskError> {
        let id = id.into();
        let input = input.into();
        if id.trim().is_empty() {
            return Err(TaskError::EmptyTaskId);
        }
        if input.trim().is_empty() {
            return Err(TaskError::EmptyInput);
        }
        Ok(Self {
            id,
            input,
            state: TaskState::Created,
            intent: None,
            plan: None,
            current_node: None,
            error: None,
        })
    }

    pub fn attach_intent(&mut self, intent: Intent) -> Result<(), TaskError> {
        self.transition(TaskState::Planning)?;
        self.intent = Some(intent);
        Ok(())
    }

    pub fn attach_plan(&mut self, plan: ActionGraph) -> Result<(), TaskError> {
        if self.intent.is_none() {
            return Err(TaskError::MissingIntent);
        }
        self.plan = Some(plan);
        self.transition(TaskState::Ready)
    }

    pub fn start(&mut self) -> Result<(), TaskError> {
        if self.plan.is_none() {
            return Err(TaskError::MissingPlan);
        }
        self.transition(TaskState::Running)
    }

    pub fn set_current_node(&mut self, node_id: impl Into<String>) -> Result<(), TaskError> {
        if self.state != TaskState::Running {
            return Err(TaskError::InvalidStateTransition {
                from: self.state,
                to: TaskState::Running,
            });
        }
        self.current_node = Some(node_id.into());
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), TaskError> {
        self.transition(TaskState::Completed)
    }

    pub fn fail(&mut self, message: impl Into<String>) -> Result<(), TaskError> {
        self.error = Some(message.into());
        self.transition(TaskState::Failed)
    }

    pub fn cancel(&mut self) -> Result<(), TaskError> {
        self.transition(TaskState::Cancelled)
    }

    fn transition(&mut self, next: TaskState) -> Result<(), TaskError> {
        if !valid_transition(self.state, next) {
            return Err(TaskError::InvalidStateTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }
}

fn valid_transition(from: TaskState, to: TaskState) -> bool {
    match from {
        TaskState::Created => matches!(to, TaskState::Planning | TaskState::Cancelled),
        TaskState::Planning => matches!(to, TaskState::Ready | TaskState::Failed | TaskState::Cancelled),
        TaskState::Ready => matches!(to, TaskState::Running | TaskState::Cancelled),
        TaskState::Running => matches!(to, TaskState::Completed | TaskState::Failed | TaskState::Cancelled),
        TaskState::Completed | TaskState::Failed | TaskState::Cancelled => false,
    }
}

#[derive(Default)]
pub struct TaskManager {
    tasks: HashMap<String, TaskSession>,
    sequence: u64,
}

impl TaskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, input: impl Into<String>) -> Result<String, TaskError> {
        let input = input.into();
        if input.trim().is_empty() {
            return Err(TaskError::EmptyInput);
        }
        self.sequence = self.sequence.saturating_add(1);
        let id = format!("task-{}", self.sequence);
        let task = TaskSession::new(id.clone(), input)?;
        self.tasks.insert(id.clone(), task);
        Ok(id)
    }

    pub fn get(&self, task_id: &str) -> Result<&TaskSession, TaskError> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| TaskError::TaskNotFound(task_id.to_string()))
    }

    pub fn get_mut(&mut self, task_id: &str) -> Result<&mut TaskSession, TaskError> {
        self.tasks
            .get_mut(task_id)
            .ok_or_else(|| TaskError::TaskNotFound(task_id.to_string()))
    }

    pub fn snapshot(&self) -> Vec<TaskSession> {
        let mut tasks: Vec<_> = self.tasks.values().cloned().collect();
        tasks.sort_by(|a, b| {
            let a_num = a.id.strip_prefix("task-").unwrap_or_default();
            let b_num = b.id.strip_prefix("task-").unwrap_or_default();
            a_num.cmp(b_num)
        });
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent() -> Intent {
        Intent {
            version: "0.1".into(),
            action: "test.run".into(),
            parameters: serde_json::json!({}),
            context: serde_json::json!({}),
            confidence: 0.99,
            constraints: serde_json::json!({}),
        }
    }

    fn plan() -> ActionGraph {
        ActionGraph::new("plan-1").unwrap()
    }

    #[test]
    fn task_lifecycle_is_ordered() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        task.attach_intent(intent()).unwrap();
        task.attach_plan(plan()).unwrap();
        task.start().unwrap();
        task.set_current_node("node-1").unwrap();
        task.complete().unwrap();
        assert_eq!(task.state, TaskState::Completed);
    }

    #[test]
    fn cannot_start_without_plan() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        task.attach_intent(intent()).unwrap();
        assert_eq!(task.start(), Err(TaskError::MissingPlan));
    }

    #[test]
    fn cannot_plan_without_intent() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        assert_eq!(task.attach_plan(plan()), Err(TaskError::MissingIntent));
    }

    #[test]
    fn failed_task_retains_error() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        task.attach_intent(intent()).unwrap();
        task.attach_plan(plan()).unwrap();
        task.start().unwrap();
        task.fail("skill failed").unwrap();
        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error.as_deref(), Some("skill failed"));
    }

    #[test]
    fn manager_generates_deterministic_ids() {
        let mut manager = TaskManager::new();
        assert_eq!(manager.create("first").unwrap(), "task-1");
        assert_eq!(manager.create("second").unwrap(), "task-2");
        assert_eq!(manager.snapshot().len(), 2);
    }
}
