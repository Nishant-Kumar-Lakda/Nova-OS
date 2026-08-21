use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Background = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Running,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSpec {
    pub id: String,
    pub priority: Priority,
    pub memory_bytes: u64,
    pub resumable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSnapshot {
    pub id: String,
    pub priority: Priority,
    pub state: TaskState,
    pub memory_bytes: u64,
    pub resumable: bool,
    pub sequence: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SchedulerError {
    #[error("task id cannot be empty")]
    EmptyTaskId,
    #[error("task already exists: {0}")]
    DuplicateTask(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("task cannot transition from {from:?} to {to:?}")]
    InvalidTransition { from: TaskState, to: TaskState },
    #[error("task cannot be paused because it is not resumable")]
    NotResumable,
}

/// Small deterministic scheduler for AIR. It selects the highest-priority
/// queued task, breaking ties by FIFO sequence number.
#[derive(Default)]
pub struct Scheduler {
    tasks: HashMap<String, TaskSnapshot>,
    sequence: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, spec: TaskSpec) -> Result<(), SchedulerError> {
        if spec.id.trim().is_empty() {
            return Err(SchedulerError::EmptyTaskId);
        }

        if self.tasks.contains_key(&spec.id) {
            return Err(SchedulerError::DuplicateTask(spec.id));
        }

        self.sequence = self.sequence.saturating_add(1);
        self.tasks.insert(
            spec.id.clone(),
            TaskSnapshot {
                id: spec.id,
                priority: spec.priority,
                state: TaskState::Queued,
                memory_bytes: spec.memory_bytes,
                resumable: spec.resumable,
                sequence: self.sequence,
            },
        );

        Ok(())
    }

    pub fn next(&self) -> Option<TaskSnapshot> {
        self.tasks
            .values()
            .filter(|task| task.state == TaskState::Queued)
            .max_by(|left, right| {
                left.priority
                    .cmp(&right.priority)
                    .then_with(|| right.sequence.cmp(&left.sequence))
            })
            .cloned()
    }

    pub fn transition(
        &mut self,
        task_id: &str,
        next_state: TaskState,
    ) -> Result<(), SchedulerError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| SchedulerError::TaskNotFound(task_id.to_string()))?;
        let current = task.state;
        if current == TaskState::Paused && next_state == TaskState::Running && !task.resumable {
            return Err(SchedulerError::NotResumable);
        }

        if !valid_transition(current, next_state) {
            return Err(SchedulerError::InvalidTransition {
                from: current,
                to: next_state,
            });
        }

        task.state = next_state;
        Ok(())
    }

    pub fn cancel(&mut self, task_id: &str) -> Result<(), SchedulerError> {
        self.transition(task_id, TaskState::Cancelled)
    }

    pub fn snapshot(&self) -> Vec<TaskSnapshot> {
        let mut tasks: Vec<_> = self.tasks.values().cloned().collect();
        tasks.sort_by(|left, right| left.sequence.cmp(&right.sequence));
        tasks
    }
}

fn valid_transition(from: TaskState, to: TaskState) -> bool {
    match from {
        TaskState::Queued => matches!(to, TaskState::Running | TaskState::Cancelled),
        TaskState::Running => matches!(
            to,
            TaskState::Paused | TaskState::Completed | TaskState::Cancelled
        ),
        TaskState::Paused => matches!(to, TaskState::Running | TaskState::Cancelled),
        TaskState::Completed | TaskState::Cancelled => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, priority: Priority, resumable: bool) -> TaskSpec {
        TaskSpec {
            id: id.into(),
            priority,
            memory_bytes: 10,
            resumable,
        }
    }

    #[test]
    fn rejects_empty_task_id() {
        let mut scheduler = Scheduler::new();
        assert_eq!(
            scheduler.submit(task("", Priority::Normal, true)),
            Err(SchedulerError::EmptyTaskId)
        );
    }

    #[test]
    fn rejects_duplicate_task() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("a", Priority::Normal, true)).unwrap();
        assert_eq!(
            scheduler.submit(task("a", Priority::High, true)),
            Err(SchedulerError::DuplicateTask("a".into()))
        );
    }

    #[test]
    fn selects_highest_priority() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("low", Priority::Low, true)).unwrap();
        scheduler
            .submit(task("critical", Priority::Critical, true))
            .unwrap();
        scheduler
            .submit(task("normal", Priority::Normal, true))
            .unwrap();

        assert_eq!(scheduler.next().unwrap().id, "critical");
    }

    #[test]
    fn equal_priority_is_fifo() {
        let mut scheduler = Scheduler::new();
        scheduler
            .submit(task("first", Priority::Normal, true))
            .unwrap();
        scheduler
            .submit(task("second", Priority::Normal, true))
            .unwrap();

        assert_eq!(scheduler.next().unwrap().id, "first");
    }

    #[test]
    fn transitions_task_lifecycle() {
        let mut scheduler = Scheduler::new();
        scheduler
            .submit(task("work", Priority::Normal, true))
            .unwrap();

        scheduler.transition("work", TaskState::Running).unwrap();
        scheduler.transition("work", TaskState::Paused).unwrap();
        scheduler.transition("work", TaskState::Running).unwrap();
        scheduler.transition("work", TaskState::Completed).unwrap();

        assert!(scheduler.next().is_none());
    }

    #[test]
    fn rejects_pause_for_finished_task() {
        let mut scheduler = Scheduler::new();
        scheduler
            .submit(task("work", Priority::Normal, true))
            .unwrap();
        scheduler.transition("work", TaskState::Running).unwrap();
        scheduler.transition("work", TaskState::Completed).unwrap();

        assert_eq!(
            scheduler.transition("work", TaskState::Paused),
            Err(SchedulerError::InvalidTransition {
                from: TaskState::Completed,
                to: TaskState::Paused,
            })
        );
    }

    #[test]
    fn non_resumable_task_cannot_resume_after_pause() {
        let mut scheduler = Scheduler::new();
        scheduler
            .submit(task("work", Priority::Normal, false))
            .unwrap();
        scheduler.transition("work", TaskState::Running).unwrap();
        scheduler.transition("work", TaskState::Paused).unwrap();

        assert_eq!(
            scheduler.transition("work", TaskState::Running),
            Err(SchedulerError::NotResumable)
        );
    }

    #[test]
    fn unknown_task_is_rejected() {
        let mut scheduler = Scheduler::new();
        assert_eq!(
            scheduler.cancel("missing"),
            Err(SchedulerError::TaskNotFound("missing".into()))
        );
    }

    #[test]
    fn snapshot_is_in_submission_order() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("b", Priority::Low, true)).unwrap();
        scheduler.submit(task("a", Priority::High, true)).unwrap();

        let snapshot = scheduler.snapshot();
        assert_eq!(snapshot[0].id, "b");
        assert_eq!(snapshot[1].id, "a");
    }
}
