use nova_nexus::Intent;
use nova_planner::{ActionGraph, ActionNode, NodeState};
use std::collections::HashMap;
use thiserror::Error;

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
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("invalid task state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: TaskState, to: TaskState },
    #[error("task requires an intent before planning")]
    MissingIntent,
    #[error("task requires a plan before execution")]
    MissingPlan,
    #[error("task has no ready action node")]
    NoReadyNode,
    #[error("task has no current action node")]
    NoCurrentNode,
    #[error("planner error: {0}")]
    Planner(String),
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
        plan.validate()
            .map_err(|error| TaskError::Planner(error.to_string()))?;
        self.plan = Some(plan);
        self.transition(TaskState::Ready)
    }

    pub fn start(&mut self) -> Result<(), TaskError> {
        if self.plan.is_none() {
            return Err(TaskError::MissingPlan);
        }
        self.transition(TaskState::Running)
    }

    pub fn next_ready_node(&mut self) -> Result<ActionNode, TaskError> {
        if self.state != TaskState::Running {
            return Err(TaskError::InvalidStateTransition {
                from: self.state,
                to: TaskState::Running,
            });
        }

        let node = self
            .plan
            .as_mut()
            .ok_or(TaskError::MissingPlan)?
            .ready_nodes()
            .into_iter()
            .next()
            .ok_or(TaskError::NoReadyNode)?;

        self.plan
            .as_mut()
            .ok_or(TaskError::MissingPlan)?
            .transition(&node.id, NodeState::Ready)
            .map_err(|error| TaskError::Planner(error.to_string()))?;
        self.plan
            .as_mut()
            .ok_or(TaskError::MissingPlan)?
            .transition(&node.id, NodeState::Running)
            .map_err(|error| TaskError::Planner(error.to_string()))?;

        self.current_node = Some(node.id.clone());
        Ok(node)
    }

    pub fn complete_current_node(&mut self) -> Result<bool, TaskError> {
        let node_id = self.current_node.take().ok_or(TaskError::NoCurrentNode)?;
        let plan = self.plan.as_mut().ok_or(TaskError::MissingPlan)?;
        plan.transition(&node_id, NodeState::Succeeded)
            .map_err(|error| TaskError::Planner(error.to_string()))?;

        let all_complete = plan
            .nodes
            .iter()
            .all(|node| matches!(node.state, NodeState::Succeeded | NodeState::Skipped));

        if all_complete {
            self.complete()?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn fail_current_node(&mut self, message: impl Into<String>) -> Result<(), TaskError> {
        let node_id = self.current_node.take().ok_or(TaskError::NoCurrentNode)?;
        let plan = self.plan.as_mut().ok_or(TaskError::MissingPlan)?;
        plan.transition(&node_id, NodeState::Failed)
            .map_err(|error| TaskError::Planner(error.to_string()))?;
        self.fail(message)
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
        TaskState::Planning => {
            matches!(to, TaskState::Ready | TaskState::Failed | TaskState::Cancelled)
        }
        TaskState::Ready => matches!(to, TaskState::Running | TaskState::Cancelled),
        TaskState::Running => {
            matches!(to, TaskState::Completed | TaskState::Failed | TaskState::Cancelled)
        }
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
        tasks.sort_by_key(|task| {
            task.id
                .strip_prefix("task-")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0)
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

    fn graph() -> ActionGraph {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        graph
            .add_node(ActionNode {
                id: "a".into(),
                action: "test.run".into(),
                parameters: serde_json::json!({}),
                depends_on: vec![],
                state: NodeState::Pending,
            })
            .unwrap();
        graph
    }

    #[test]
    fn task_executes_action_graph_to_completion() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        task.attach_intent(intent()).unwrap();
        task.attach_plan(graph()).unwrap();
        task.start().unwrap();

        let node = task.next_ready_node().unwrap();
        assert_eq!(node.id, "a");
        assert!(!task.complete_current_node().unwrap() || task.state == TaskState::Completed);
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
        assert_eq!(task.attach_plan(graph()), Err(TaskError::MissingIntent));
    }

    #[test]
    fn failed_task_retains_error() {
        let mut task = TaskSession::new("task-1", "do something").unwrap();
        task.attach_intent(intent()).unwrap();
        task.attach_plan(graph()).unwrap();
        task.start().unwrap();
        task.next_ready_node().unwrap();
        task.fail_current_node("skill failed").unwrap();
        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error.as_deref(), Some("skill failed"));
    }

    #[test]
    fn manager_generates_numeric_ordered_ids() {
        let mut manager = TaskManager::new();
        for _ in 0..11 {
            manager.create("task").unwrap();
        }
        let snapshot = manager.snapshot();
        assert_eq!(snapshot.first().unwrap().id, "task-1");
        assert_eq!(snapshot.last().unwrap().id, "task-11");
    }
}
