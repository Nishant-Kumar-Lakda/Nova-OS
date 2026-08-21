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

        assert_eq!(
            backend.infer(&request),
            Err(BackendError::EmptyModelId)
        );
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
}
