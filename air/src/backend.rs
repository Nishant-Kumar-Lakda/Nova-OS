use crate::{AirError, ModelManager, ModelSpec, ResourcePolicy, ResourceSnapshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A normalized request sent to an offline inference backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceRequest {
    pub model_id: String,
    pub input: String,
    pub max_tokens: u32,
    pub temperature_millis: u32,
}

/// A normalized result returned by an inference backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceResult {
    pub model_id: String,
    pub text: String,
    pub tokens_generated: u32,
    pub finished: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BackendError {
    #[error("model id cannot be empty")]
    EmptyModelId,
    #[error("input cannot be empty")]
    EmptyInput,
    #[error("max_tokens must be greater than zero")]
    InvalidMaxTokens,
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("inference failed: {0}")]
    InferenceFailed(String),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InferenceError {
    #[error(transparent)]
    Model(#[from] AirError),
    #[error(transparent)]
    Backend(#[from] BackendError),
}

/// Stable AIR interface for local model engines.
///
/// Concrete implementations can wrap llama.cpp, ONNX Runtime, ExecuTorch,
/// a future C/C++ engine, or a NOVA-specific mobile backend without changing
/// the rest of AIR.
pub trait InferenceBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResult, BackendError>;
}

/// Deterministic backend used by AIR tests and development builds.
/// It never loads a model or accesses the network.
#[derive(Debug, Default, Clone, Copy)]
pub struct EchoBackend;

impl InferenceBackend for EchoBackend {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResult, BackendError> {
        validate_request(request)?;

        Ok(InferenceResult {
            model_id: request.model_id.clone(),
            text: request.input.clone(),
            tokens_generated: request.input.split_whitespace().count() as u32,
            finished: true,
        })
    }
}

/// AIR's orchestration boundary between model residency and inference.
/// It guarantees that the requested model is registered and resident before
/// handing the request to a concrete backend.
pub struct InferenceEngine<B> {
    model_manager: ModelManager,
    backend: B,
    resource_policy: ResourcePolicy,
}

impl<B> InferenceEngine<B>
where
    B: InferenceBackend,
{
    pub fn new(memory_budget: u64, backend: B) -> Result<Self, AirError> {
        Ok(Self {
            model_manager: ModelManager::new(memory_budget)?,
            backend,
            resource_policy: ResourcePolicy::default(),
        })
    }

    pub fn register_model(&mut self, model: ModelSpec) -> Result<(), AirError> {
        self.model_manager.register(model)
    }

    pub fn apply_resource_snapshot(&mut self, snapshot: ResourceSnapshot) -> Result<u64, AirError> {
        self.model_manager
            .apply_resource_policy(self.resource_policy, snapshot)
    }

    pub fn infer(&mut self, request: &InferenceRequest) -> Result<InferenceResult, InferenceError> {
        if !self.backend.is_available() {
            return Err(InferenceError::Backend(BackendError::Unavailable(
                self.backend.name().into(),
            )));
        }

        self.model_manager.load(&request.model_id)?;
        Ok(self.backend.infer(request)?)
    }

    pub fn used_memory(&self) -> u64 {
        self.model_manager.used_memory()
    }

    pub fn available_memory(&self) -> u64 {
        self.model_manager.available_memory()
    }

    pub fn memory_budget(&self) -> u64 {
        self.model_manager.memory_budget()
    }

    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }
}

fn validate_request(request: &InferenceRequest) -> Result<(), BackendError> {
    if request.model_id.trim().is_empty() {
        return Err(BackendError::EmptyModelId);
    }
    if request.input.trim().is_empty() {
        return Err(BackendError::EmptyInput);
    }
    if request.max_tokens == 0 {
        return Err(BackendError::InvalidMaxTokens);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> InferenceRequest {
        InferenceRequest {
            model_id: "test.intent".into(),
            input: "turn on flashlight".into(),
            max_tokens: 16,
            temperature_millis: 0,
        }
    }

    fn model(id: &str, size: u64) -> ModelSpec {
        ModelSpec {
            id: id.into(),
            version: "0.1.0".into(),
            size_bytes: size,
            capabilities: vec!["intent".into()],
            path: format!("models/{id}.bin"),
        }
    }

    #[test]
    fn echo_backend_is_available() {
        let backend = EchoBackend;
        assert!(backend.is_available());
        assert_eq!(backend.name(), "echo");
    }

    #[test]
    fn echo_backend_returns_deterministic_result() {
        let backend = EchoBackend;
        let result = backend.infer(&request()).unwrap();

        assert_eq!(result.model_id, "test.intent");
        assert_eq!(result.text, "turn on flashlight");
        assert_eq!(result.tokens_generated, 3);
        assert!(result.finished);
    }

    #[test]
    fn rejects_empty_model() {
        let backend = EchoBackend;
        let mut request = request();
        request.model_id.clear();

        assert_eq!(backend.infer(&request), Err(BackendError::EmptyModelId));
    }

    #[test]
    fn rejects_empty_input() {
        let backend = EchoBackend;
        let mut request = request();
        request.input = "   ".into();

        assert_eq!(backend.infer(&request), Err(BackendError::EmptyInput));
    }

    #[test]
    fn rejects_zero_max_tokens() {
        let backend = EchoBackend;
        let mut request = request();
        request.max_tokens = 0;

        assert_eq!(
            backend.infer(&request),
            Err(BackendError::InvalidMaxTokens)
        );
    }

    #[test]
    fn engine_loads_model_before_inference() {
        let mut engine = InferenceEngine::new(100, EchoBackend).unwrap();
        engine.register_model(model("test.intent", 40)).unwrap();

        let result = engine.infer(&request()).unwrap();

        assert_eq!(result.text, "turn on flashlight");
        assert_eq!(engine.used_memory(), 40);
        assert_eq!(engine.available_memory(), 60);
        assert_eq!(engine.backend_name(), "echo");
    }

    #[test]
    fn engine_rejects_unregistered_model() {
        let mut engine = InferenceEngine::new(100, EchoBackend).unwrap();

        assert_eq!(
            engine.infer(&request()),
            Err(InferenceError::Model(AirError::ModelNotFound(
                "test.intent".into()
            )))
        );
    }

    #[test]
    fn engine_applies_android_style_resource_snapshot() {
        let mut engine = InferenceEngine::new(500_000_000, EchoBackend).unwrap();
        let budget = engine
            .apply_resource_snapshot(ResourceSnapshot {
                available_memory_bytes: 1_000_000_000,
                battery_percent: 15,
                low_memory: false,
                low_power: true,
            })
            .unwrap();

        assert_eq!(budget, 150_000_000);
        assert_eq!(engine.memory_budget(), 150_000_000);
    }
}
