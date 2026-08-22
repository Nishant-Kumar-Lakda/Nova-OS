use nova_nexus::{parse, Intent};
use nova_platform::Platform;
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

pub struct SkillContext<'a> {
    pub platform: &'a dyn Platform,
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
    #[error("invalid skill id")]
    InvalidSkillId,
    #[error("skill must declare at least one action: {0}")]
    EmptySkillActions(String),
    #[error("invalid skill action: {0}")]
    InvalidSkillAction(String),
    #[error("duplicate skill: {0}")]
    DuplicateSkill(String),
    #[error("duplicate action claim: {0}")]
    DuplicateActionClaim(String),
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

    /// Optional platform-aware execution hook. Existing skills remain
    /// compatible; platform-enabled skills can override this method.
    fn execute_with_context(
        &self,
        intent: &Intent,
        _context: &SkillContext<'_>,
    ) -> Result<SkillResult, RuntimeError> {
        self.execute(intent)
    }
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
        validate_skill_metadata(&metadata)?;

        if self
            .skills
            .iter()
            .any(|existing| existing.metadata().id == metadata.id)
        {
            return Err(RuntimeError::DuplicateSkill(metadata.id));
        }

        for action in &metadata.actions {
            if self.skills.iter().any(|existing| {
                existing
                    .metadata()
                    .actions
                    .iter()
                    .any(|item| item == action)
            }) {
                return Err(RuntimeError::DuplicateActionClaim(action.clone()));
            }
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

pub fn validate_skill_metadata(metadata: &SkillMetadata) -> Result<(), RuntimeError> {
    if metadata.id.trim().is_empty() {
        return Err(RuntimeError::InvalidSkillId);
    }
    if metadata.actions.is_empty() {
        return Err(RuntimeError::EmptySkillActions(metadata.id.clone()));
    }
    for action in &metadata.actions {
        if action.trim().is_empty() || !action.contains('.') {
            return Err(RuntimeError::InvalidSkillAction(action.clone()));
        }
    }
    Ok(())
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

pub fn execute_with_platform(
    registry: &SkillRegistry,
    intent: &Intent,
    platform: &dyn Platform,
) -> Result<SkillResult, RuntimeError> {
    validate_intent(intent)?;

    let skill = registry
        .find_for_action(&intent.action)
        .ok_or_else(|| RuntimeError::SkillNotFound(intent.action.clone()))?;

    let context = SkillContext { platform };
    skill.execute_with_context(intent, &context)
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
    use nova_platform::MockPlatform;

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

    struct BatterySkill;

    impl Skill for BatterySkill {
        fn metadata(&self) -> SkillMetadata {
            SkillMetadata {
                id: "battery".into(),
                version: "0.1.0".into(),
                actions: vec!["battery.read".into()],
                permissions: vec!["device.battery".into()],
            }
        }

        fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
            unreachable!()
        }

        fn execute_with_context(
            &self,
            _intent: &Intent,
            context: &SkillContext<'_>,
        ) -> Result<SkillResult, RuntimeError> {
            let value = context
                .platform
                .battery_percent()
                .map_err(|error| RuntimeError::SkillExecutionFailed(error.to_string()))?;
            Ok(SkillResult {
                success: true,
                data: serde_json::json!({"battery_percent": value}),
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
        assert_eq!(execution_decision(&intent), ExecutionDecision::Confirm);

        intent.confidence = 0.60;
        assert_eq!(execution_decision(&intent), ExecutionDecision::Clarify);
    }

    #[test]
    fn rejects_invalid_skill_metadata() {
        assert_eq!(
            validate_skill_metadata(&SkillMetadata {
                id: "".into(),
                version: "0.1.0".into(),
                actions: vec!["test.run".into()],
                permissions: vec![],
            }),
            Err(RuntimeError::InvalidSkillId)
        );
    }

    #[test]
    fn rejects_action_collision() {
        let mut registry = SkillRegistry::new();
        registry.register(Box::new(TestSkill)).unwrap();

        struct OtherSkill;
        impl Skill for OtherSkill {
            fn metadata(&self) -> SkillMetadata {
                SkillMetadata {
                    id: "other".into(),
                    version: "0.1.0".into(),
                    actions: vec!["test.run".into()],
                    permissions: vec![],
                }
            }

            fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
                unreachable!()
            }
        }

        assert_eq!(
            registry.register(Box::new(OtherSkill)),
            Err(RuntimeError::DuplicateActionClaim("test.run".into()))
        );
    }

    #[test]
    fn executes_platform_aware_skill_without_cloud_access() {
        let mut registry = SkillRegistry::new();
        registry.register(Box::new(BatterySkill)).unwrap();
        let platform = MockPlatform::new(73).unwrap();
        let intent = Intent {
            version: "0.1".into(),
            action: "battery.read".into(),
            parameters: serde_json::json!({}),
            context: serde_json::json!({}),
            confidence: 0.99,
            constraints: serde_json::json!({}),
        };

        let result = execute_with_platform(&registry, &intent, &platform).unwrap();
        assert_eq!(result.data["battery_percent"], 73);
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
