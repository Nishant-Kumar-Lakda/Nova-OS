use nova_air::backend::{InferenceBackend, InferenceEngine, InferenceError, InferenceRequest, InferenceResult};
use nova_air::ModelSpec;
use nova_context::{ContextEngine, ContextSnapshot, ContextError, ContextEntity};
use nova_memory::{InMemoryStore, MemoryError, MemoryQuery, MemoryRecord, MemoryStore};
use nova_nexus::{parse, Intent, IntentError};
use nova_planner::{ActionGraph, ActionNode, PlannerError};
use nova_runtime::{execute_text, RuntimeError, SkillRegistry, SkillResult};
use thiserror::Error;

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
    #[error("AIR initialization failed: {0}")]
    AirInitialization(String),
}

/// Top-level NOVA orchestration layer. It connects intent understanding,
/// planning, memory, context, skills, and AIR without tying them to a phone
/// or desktop platform.
pub struct NovaEngine<B>
where
    B: InferenceBackend,
{
    context: ContextEngine,
    memory: InMemoryStore,
    inference: InferenceEngine<B>,
    skills: SkillRegistry,
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
        })
    }

    pub fn understand(&mut self, input: &str) -> Result<Intent, NovaError> {
        self.context.set_user_input(input);
        Ok(parse(input)?)
    }

    pub fn execute_simple(&self, input: &str) -> Result<SkillResult, NovaError> {
        Ok(execute_text(&self.skills, input)?)
    }

    pub fn plan(&self, id: impl Into<String>, nodes: Vec<ActionNode>) -> Result<ActionGraph, NovaError> {
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
