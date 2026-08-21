use std::collections::{HashMap, VecDeque};

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
    pub memory_mb: u64,
    pub resumable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub spec: TaskSpec,
    pub state: TaskState,
    pub sequence: u64,
}

#[derive(Debug, Default)]
pub struct Scheduler {
    tasks: HashMap<String, TaskInfo>,
    queue: VecDeque<String>,
    next_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    EmptyTaskId,
    DuplicateTask(String),
    UnknownTask(String),
    InvalidMemory,
    CannotPause(String),
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, spec: TaskSpec) -> Result<(), SchedulerError> {
        if spec.id.trim().is_empty() {
            return Err(SchedulerError::EmptyTaskId);
        }
        if spec.memory_mb == 0 {
            return Err(SchedulerError::InvalidMemory);
        }
        if self.tasks.contains_key(&spec.id) {
            return Err(SchedulerError::DuplicateTask(spec.id));
        }

        let id = spec.id.clone();
        let info = TaskInfo {
            spec,
            state: TaskState::Queued,
            sequence: self.next_sequence,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.tasks.insert(id.clone(), info);
        self.queue.push_back(id);
        Ok(())
    }

    pub fn next_task(&self) -> Option<&TaskInfo> {
        self.queue
            .iter()
            .filter_map(|id| self.tasks.get(id))
            .max_by_key(|task| (task.spec.priority, std::cmp::Reverse(task.sequence)))
    }

    pub fn start(&mut self, id: &str) -> Result<(), SchedulerError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| SchedulerError::UnknownTask(id.into()))?;
        task.state = TaskState::Running;
        self.queue.retain(|queued| queued != id);
        Ok(())
    }

    pub fn pause(&mut self, id: &str) -> Result<(), SchedulerError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| SchedulerError::UnknownTask(id.into()))?;
        if !task.spec.resumable {
            return Err(SchedulerError::CannotPause(id.into()));
        }
        task.state = TaskState::Paused;
        Ok(())
    }

    pub fn cancel(&mut self, id: &str) -> Result<(), SchedulerError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| SchedulerError::UnknownTask(id.into()))?;
        task.state = TaskState::Cancelled;
        self.queue.retain(|queued| queued != id);
        Ok(())
    }

    pub fn complete(&mut self, id: &str) -> Result<(), SchedulerError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| SchedulerError::UnknownTask(id.into()))?;
        task.state = TaskState::Completed;
        self.queue.retain(|queued| queued != id);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&TaskInfo> {
        self.tasks.get(id)
    }

    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, priority: Priority, memory_mb: u64) -> TaskSpec {
        TaskSpec {
            id: id.into(),
            priority,
            memory_mb,
            resumable: true,
        }
    }

    #[test]
    fn rejects_invalid_submission() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.submit(task("", Priority::Normal, 10)), Err(SchedulerError::EmptyTaskId));
        assert_eq!(scheduler.submit(task("a", Priority::Normal, 0)), Err(SchedulerError::InvalidMemory));
    }

    #[test]
    fn rejects_duplicate_task() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("a", Priority::Normal, 10)).unwrap();
        assert_eq!(scheduler.submit(task("a", Priority::High, 20)), Err(SchedulerError::DuplicateTask("a".into())));
    }

    #[test]
    fn chooses_highest_priority_first() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("low", Priority::Low, 10)).unwrap();
        scheduler.submit(task("critical", Priority::Critical, 10)).unwrap();
        scheduler.submit(task("normal", Priority::Normal, 10)).unwrap();
        assert_eq!(scheduler.next_task().unwrap().spec.id, "critical");
    }

    #[test]
    fn equal_priority_is_fifo() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("first", Priority::Normal, 10)).unwrap();
        scheduler.submit(task("second", Priority::Normal, 10)).unwrap();
        assert_eq!(scheduler.next_task().unwrap().spec.id, "first");
    }

    #[test]
    fn lifecycle_updates_state() {
        let mut scheduler = Scheduler::new();
        scheduler.submit(task("a", Priority::Normal, 10)).unwrap();
        scheduler.start("a").unwrap();
        assert_eq!(scheduler.get("a").unwrap().state, TaskState::Running);
        scheduler.pause("a").unwrap();
        assert_eq!(scheduler.get("a").unwrap().state, TaskState::Paused);
        scheduler.complete("a").unwrap();
        assert_eq!(scheduler.get("a").unwrap().state, TaskState::Completed);
    }

    #[test]
    fn cannot_pause_non_resumable_task() {
        let mut scheduler = Scheduler::new();
        let mut spec = task("a", Priority::Normal, 10);
        spec.resumable = false;
        scheduler.submit(spec).unwrap();
        assert_eq!(scheduler.pause("a"), Err(SchedulerError::CannotPause("a".into())));
    }

    #[test]
    fn unknown_task_is_rejected() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.start("missing"), Err(SchedulerError::UnknownTask("missing".into())));
        assert_eq!(scheduler.cancel("missing"), Err(SchedulerError::UnknownTask("missing".into())));
    }
}
