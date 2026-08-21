use nova_nexus::{parse, Intent};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillMetadata {
    pub id: String,
    pub version: String,
    pub actions: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillResult {
    pub success: bool,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionDecision {
    Execute,
    Confirm,
    Clarify,
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("invalid intent version: {0}")]
    InvalidIntentVersion(String),
    #[error("invalid action: {0}")]
    InvalidAction(String),
    #[error("invalid confidence: {0}")]
    InvalidConfidence(f32),
    #[error("duplicate skill: {0}")]
    DuplicateSkill(String),
    #[error("skill not found for action: {0}")]
    SkillNotFound(String),
    #[error("skill execution failed: {0}")]
    SkillExecutionFailed(String),
    #[error("intent parsing failed: {0}")]
    IntentParsingFailed(String),
}

pub trait Skill: Send + Sync {
    fn metadata(&self) -> SkillMetadata;
    fn execute(&self, intent: &Intent) -> Result<SkillResult, RuntimeError>;
}

#[derive(Default)]
pub struct SkillRegistry {
    skills: Vec<Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) -> Result<(), RuntimeError> {
        let metadata = skill.metadata();
        if self
            .skills
            .iter()
            .any(|existing| existing.metadata().id == metadata.id)
        {
            return Err(RuntimeError::DuplicateSkill(metadata.id));
        }
        self.skills.push(skill);
        Ok(())
    }

    pub fn find_for_action(&self, action: &str) -> Option<&dyn Skill> {
        self.skills
            .iter()
            .find(|skill| {
                skill
                    .metadata()
                    .actions
                    .iter()
                    .any(|supported| supported == action)
            })
            .map(|skill| skill.as_ref())
    }

    pub fn list(&self) -> Vec<SkillMetadata> {
        self.skills.iter().map(|skill| skill.metadata()).collect()
    }
}

pub fn validate_intent(intent: &Intent) -> Result<(), RuntimeError> {
    if intent.version != "0.1" {
        return Err(RuntimeError::InvalidIntentVersion(intent.version.clone()));
    }

    if intent.action.trim().is_empty() || !intent.action.contains('.') {
        return Err(RuntimeError::InvalidAction(intent.action.clone()));
    }

    if !intent.confidence.is_finite() || !(0.0..=1.0).contains(&intent.confidence) {
        return Err(RuntimeError::InvalidConfidence(intent.confidence));
    }

    Ok(())
}

/// Maps NIL confidence to an execution policy. High-risk actions must later
/// override this with explicit confirmation requirements from the skill.
pub fn execution_decision(intent: &Intent) -> ExecutionDecision {
    if intent.confidence >= 0.95 {
        ExecutionDecision::Execute
    } else if intent.confidence >= 0.75 {
        ExecutionDecision::Confirm
    } else {
        ExecutionDecision::Clarify
    }
}

pub fn execute(registry: &SkillRegistry, intent: &Intent) -> Result<SkillResult, RuntimeError> {
    validate_intent(intent)?;

    let skill = registry
        .find_for_action(&intent.action)
        .ok_or_else(|| RuntimeError::SkillNotFound(intent.action.clone()))?;

    skill.execute(intent)
}

/// End-to-end local pipeline: text -> NEXUS -> NIL validation -> skill dispatch.
/// No network or cloud service is involved.
pub fn execute_text(registry: &SkillRegistry, input: &str) -> Result<SkillResult, RuntimeError> {
    let intent =
        parse(input).map_err(|error| RuntimeError::IntentParsingFailed(error.to_string()))?;

    match execution_decision(&intent) {
        ExecutionDecision::Execute => execute(registry, &intent),
        ExecutionDecision::Confirm => Err(RuntimeError::SkillExecutionFailed(
            "confirmation required".into(),
        )),
        ExecutionDecision::Clarify => Err(RuntimeError::SkillExecutionFailed(
            "clarification required".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSkill;

    impl Skill for TestSkill {
        fn metadata(&self) -> SkillMetadata {
            SkillMetadata {
                id: "test".into(),
                version: "0.1.0".into(),
                actions: vec!["test.run".into()],
                permissions: vec![],
            }
        }

        fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
            Ok(SkillResult {
                success: true,
                data: serde_json::json!({"message": "skill executed"}),
            })
        }
    }

    fn test_intent() -> Intent {
        Intent {
            version: "0.1".into(),
            action: "test.run".into(),
            parameters: serde_json::json!({}),
            context: serde_json::json!({}),
            confidence: 0.99,
            constraints: serde_json::json!({}),
        }
    }

    #[test]
    fn validates_intent() {
        assert!(validate_intent(&test_intent()).is_ok());
    }

    #[test]
    fn rejects_bad_version() {
        let mut intent = test_intent();
        intent.version = "9.0".into();
        assert_eq!(
            validate_intent(&intent),
            Err(RuntimeError::InvalidIntentVersion("9.0".into()))
        );
    }

    #[test]
    fn applies_confidence_policy() {
        assert_eq!(
            execution_decision(&test_intent()),
            ExecutionDecision::Execute
        );

        let mut intent = test_intent();
        intent.confidence = 0.80;
        assert_eq!(
            execution_decision(&intent),
            ExecutionDecision::Confirm
        );

        intent.confidence = 0.60;
        assert_eq!(
            execution_decision(&intent),
            ExecutionDecision::Clarify
        );
    }

    #[test]
    fn registers_and_executes_skill() {
        let mut registry = SkillRegistry::new();
        registry.register(Box::new(TestSkill)).unwrap();

        let result = execute(&registry, &test_intent()).unwrap();
        assert!(result.success);
        assert_eq!(result.data["message"], "skill executed");
    }

    #[test]
    fn rejects_unknown_action() {
        let registry = SkillRegistry::new();
        let mut intent = test_intent();
        intent.action = "missing.action".into();

        assert_eq!(
            execute(&registry, &intent),
            Err(RuntimeError::SkillNotFound("missing.action".into()))
        );
    }
}
