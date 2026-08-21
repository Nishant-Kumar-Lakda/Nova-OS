use nova_air::backend::{
    InferenceBackend, InferenceEngine, InferenceError, InferenceRequest, InferenceResult,
};
use nova_air::ModelSpec;
use nova_context::{ContextEngine, ContextEntity, ContextError, ContextSnapshot};
use nova_core_skills::builtin_skills;
use nova_memory::{InMemoryStore, MemoryError, MemoryQuery, MemoryRecord, MemoryStore};
use nova_nexus::{parse, Intent, IntentError};
use nova_planner::{ActionGraph, ActionNode, PlannerError};
use nova_platform::Platform;
use nova_runtime::{
    execute_text, execute_with_platform, RuntimeError, Skill, SkillRegistry, SkillResult,
};
use thiserror::Error;

pub mod task;

pub use task::{TaskError, TaskManager, TaskSession, TaskState};

#[derive(Debug, Error)]
pub enum NovaError {
    #[error(transparent)]
    Intent(#[from] IntentError),
    #[error(transparent)]
    Planner(#[from] PlannerError),
    #[error(transparent)]
    Memory(#[from] MemoryError),
    #[error(transparent)]
    Context(#[from] ContextError),
    #[error(transparent)]
    Inference(#[from] InferenceError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error("AIR initialization failed: {0}")]
    AirInitialization(String),
}

/// Top-level NOVA orchestration layer. It connects intent understanding,
/// planning, memory, context, skills, task lifecycle, AIR, and platform
/// execution without tying the core to Android, Linux, or Windows.
pub struct NovaEngine<B>
where
    B: InferenceBackend,
{
    context: ContextEngine,
    memory: InMemoryStore,
    inference: InferenceEngine<B>,
    skills: SkillRegistry,
    tasks: TaskManager,
}

impl<B> NovaEngine<B>
where
    B: InferenceBackend,
{
    pub fn new(memory_budget: u64, backend: B, context_capacity: usize) -> Result<Self, NovaError> {
        let inference = InferenceEngine::new(memory_budget, backend)
            .map_err(|error| NovaError::AirInitialization(error.to_string()))?;

        Ok(Self {
            context: ContextEngine::new(context_capacity),
            memory: InMemoryStore::new(),
            inference,
            skills: SkillRegistry::new(),
            tasks: TaskManager::new(),
        })
    }

    pub fn register_builtin_skills(&mut self) -> Result<(), NovaError> {
        for skill in builtin_skills() {
            self.register_skill(skill)?;
        }
        Ok(())
    }

    pub fn understand(&mut self, input: &str) -> Result<Intent, NovaError> {
        self.context.set_user_input(input);
        Ok(parse(input)?)
    }

    pub fn create_task(&mut self, input: impl Into<String>) -> Result<String, NovaError> {
        Ok(self.tasks.create(input)?)
    }

    pub fn task(&self, task_id: &str) -> Result<&TaskSession, NovaError> {
        Ok(self.tasks.get(task_id)?)
    }

    pub fn task_snapshot(&self) -> Vec<TaskSession> {
        self.tasks.snapshot()
    }

    pub fn understand_task(&mut self, task_id: &str) -> Result<Intent, NovaError> {
        let input = self.tasks.get(task_id)?.input.clone();
        let intent = self.understand(&input)?;
        self.tasks.get_mut(task_id)?.attach_intent(intent.clone())?;
        Ok(intent)
    }

    pub fn attach_task_plan(
        &mut self,
        task_id: &str,
        plan: ActionGraph,
    ) -> Result<(), NovaError> {
        self.tasks.get_mut(task_id)?.attach_plan(plan)?;
        Ok(())
    }

    pub fn start_task(&mut self, task_id: &str) -> Result<(), NovaError> {
        self.tasks.get_mut(task_id)?.start()?;
        Ok(())
    }

    pub fn next_task_node(&mut self, task_id: &str) -> Result<ActionNode, NovaError> {
        Ok(self.tasks.get_mut(task_id)?.next_ready_node()?)
    }

    pub fn complete_task_node(&mut self, task_id: &str) -> Result<bool, NovaError> {
        Ok(self.tasks.get_mut(task_id)?.complete_current_node()?)
    }

    pub fn fail_task_node(
        &mut self,
        task_id: &str,
        message: impl Into<String>,
    ) -> Result<(), NovaError> {
        self.tasks
            .get_mut(task_id)?
            .fail_current_node(message)?;
        Ok(())
    }

    pub fn complete_task(&mut self, task_id: &str) -> Result<(), NovaError> {
        self.tasks.get_mut(task_id)?.complete()?;
        Ok(())
    }

    pub fn fail_task(&mut self, task_id: &str, message: impl Into<String>) -> Result<(), NovaError> {
        self.tasks.get_mut(task_id)?.fail(message)?;
        Ok(())
    }

    pub fn cancel_task(&mut self, task_id: &str) -> Result<(), NovaError> {
        self.tasks.get_mut(task_id)?.cancel()?;
        Ok(())
    }

    pub fn register_skill(&mut self, skill: Box<dyn Skill>) -> Result<(), NovaError> {
        self.skills.register(skill)?;
        Ok(())
    }

    pub fn execute_simple(&self, input: &str) -> Result<SkillResult, NovaError> {
        Ok(execute_text(&self.skills, input)?)
    }

    pub fn execute_simple_on_platform(
        &self,
        input: &str,
        platform: &dyn Platform,
    ) -> Result<SkillResult, NovaError> {
        let intent = parse(input)?;
        Ok(execute_with_platform(&self.skills, &intent, platform)?)
    }

    pub fn plan(
        &self,
        id: impl Into<String>,
        nodes: Vec<ActionNode>,
    ) -> Result<ActionGraph, NovaError> {
        let mut graph = ActionGraph::new(id)?;
        for node in nodes {
            graph.add_node(node)?;
        }
        graph.validate()?;
        Ok(graph)
    }

    pub fn register_model(&mut self, model: ModelSpec) -> Result<(), NovaError> {
        self.inference
            .register_model(model)
            .map_err(|error| NovaError::AirInitialization(error.to_string()))
    }

    pub fn infer(&mut self, request: &InferenceRequest) -> Result<InferenceResult, NovaError> {
        Ok(self.inference.infer(request)?)
    }

    pub fn remember(&mut self, record: MemoryRecord) -> Result<(), NovaError> {
        self.memory.put(record)?;
        Ok(())
    }

    pub fn recall(&self, query: &MemoryQuery) -> Vec<MemoryRecord> {
        self.memory.search(query)
    }

    pub fn remember_entity(&mut self, entity: ContextEntity) -> Result<(), NovaError> {
        self.context.remember(entity)?;
        Ok(())
    }

    pub fn context_snapshot(&self) -> ContextSnapshot {
        self.context.snapshot()
    }

    pub fn used_ai_memory(&self) -> u64 {
        self.inference.used_memory()
    }

    pub fn available_ai_memory(&self) -> u64 {
        self.inference.available_memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_air::backend::EchoBackend;
    use nova_platform::MockPlatform;

    #[test]
    fn creates_and_understands_task() {
        let mut nova = NovaEngine::new(1024, EchoBackend, 8).unwrap();
        let task_id = nova.create_task("check battery").unwrap();
        let intent = nova.understand_task(&task_id).unwrap();

        assert_eq!(intent.action, "battery.status");
        assert_eq!(nova.task(&task_id).unwrap().state, TaskState::Planning);
    }

    #[test]
    fn task_snapshot_is_available_from_engine() {
        let mut nova = NovaEngine::new(1024, EchoBackend, 8).unwrap();
        nova.create_task("check battery").unwrap();
        nova.create_task("turn on flashlight").unwrap();

        assert_eq!(nova.task_snapshot().len(), 2);
    }

    #[test]
    fn builtins_execute_against_mock_platform() {
        let mut nova = NovaEngine::new(1024, EchoBackend, 8).unwrap();
        nova.register_builtin_skills().unwrap();
        let platform = MockPlatform::new(92).unwrap();

        let battery = nova
            .execute_simple_on_platform("check battery", &platform)
            .unwrap();
        assert_eq!(battery.data["battery_percent"], 92);
    }
}
