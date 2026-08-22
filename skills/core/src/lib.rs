use nova_nexus::Intent;
use nova_runtime::{RuntimeError, Skill, SkillContext, SkillMetadata, SkillResult};

fn platform_required() -> RuntimeError {
    RuntimeError::SkillExecutionFailed("platform context required".into())
}

pub struct BatterySkill;
pub struct FlashlightSkill;
pub struct WifiSkill;
pub struct BluetoothSkill;

impl Skill for BatterySkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            id: "nova.core.battery".into(),
            version: "0.1.0".into(),
            actions: vec!["battery.status".into()],
            permissions: vec!["device.battery".into()],
        }
    }

    fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
        Err(platform_required())
    }

    fn execute_with_context(
        &self,
        _intent: &Intent,
        context: &SkillContext<'_>,
    ) -> Result<SkillResult, RuntimeError> {
        let percent = context
            .platform
            .battery_percent()
            .map_err(|error| RuntimeError::SkillExecutionFailed(error.to_string()))?;
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({"battery_percent": percent}),
        })
    }
}

impl Skill for FlashlightSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            id: "nova.core.flashlight".into(),
            version: "0.1.0".into(),
            actions: vec!["flashlight.on".into(), "flashlight.off".into()],
            permissions: vec!["device.flashlight".into()],
        }
    }

    fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
        Err(platform_required())
    }

    fn execute_with_context(
        &self,
        intent: &Intent,
        context: &SkillContext<'_>,
    ) -> Result<SkillResult, RuntimeError> {
        let enabled = match intent.action.as_str() {
            "flashlight.on" => true,
            "flashlight.off" => false,
            _ => return Err(RuntimeError::InvalidAction(intent.action.clone())),
        };
        context
            .platform
            .flashlight(enabled)
            .map_err(|error| RuntimeError::SkillExecutionFailed(error.to_string()))?;
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({"enabled": enabled}),
        })
    }
}

impl Skill for WifiSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            id: "nova.core.wifi".into(),
            version: "0.1.0".into(),
            actions: vec!["wifi.enable".into(), "wifi.disable".into()],
            permissions: vec!["device.wifi".into()],
        }
    }

    fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
        Err(platform_required())
    }

    fn execute_with_context(
        &self,
        intent: &Intent,
        context: &SkillContext<'_>,
    ) -> Result<SkillResult, RuntimeError> {
        let enabled = match intent.action.as_str() {
            "wifi.enable" => true,
            "wifi.disable" => false,
            _ => return Err(RuntimeError::InvalidAction(intent.action.clone())),
        };
        context
            .platform
            .wifi(enabled)
            .map_err(|error| RuntimeError::SkillExecutionFailed(error.to_string()))?;
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({"enabled": enabled}),
        })
    }
}

impl Skill for BluetoothSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            id: "nova.core.bluetooth".into(),
            version: "0.1.0".into(),
            actions: vec!["bluetooth.enable".into(), "bluetooth.disable".into()],
            permissions: vec!["device.bluetooth".into()],
        }
    }

    fn execute(&self, _intent: &Intent) -> Result<SkillResult, RuntimeError> {
        Err(platform_required())
    }

    fn execute_with_context(
        &self,
        intent: &Intent,
        context: &SkillContext<'_>,
    ) -> Result<SkillResult, RuntimeError> {
        let enabled = match intent.action.as_str() {
            "bluetooth.on" => true,
            "bluetooth.off" => false,
            "bluetooth.enable" => true,
            "bluetooth.disable" => false,
            _ => return Err(RuntimeError::InvalidAction(intent.action.clone())),
        };
        context
            .platform
            .bluetooth(enabled)
            .map_err(|error| RuntimeError::SkillExecutionFailed(error.to_string()))?;
        Ok(SkillResult {
            success: true,
            data: serde_json::json!({"enabled": enabled}),
        })
    }
}

pub fn builtin_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(BatterySkill),
        Box::new(FlashlightSkill),
        Box::new(WifiSkill),
        Box::new(BluetoothSkill),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_platform::{MockPlatform, Platform};
    use nova_runtime::{execute_with_platform, SkillRegistry};

    fn intent(action: &str) -> Intent {
        Intent {
            version: "0.1".into(),
            action: action.into(),
            parameters: serde_json::json!({}),
            context: serde_json::json!({}),
            confidence: 0.99,
            constraints: serde_json::json!({}),
        }
    }

    fn registry() -> SkillRegistry {
        let mut registry = SkillRegistry::new();
        for skill in builtin_skills() {
            registry.register(skill).unwrap();
        }
        registry
    }

    #[test]
    fn registers_all_builtin_skills() {
        assert_eq!(registry().list().len(), 4);
    }

    #[test]
    fn battery_skill_reads_mock_platform() {
        let registry = registry();
        let platform = MockPlatform::new(81).unwrap();
        let result = execute_with_platform(&registry, &intent("battery.status"), &platform).unwrap();
        assert_eq!(result.data["battery_percent"], 81);
    }

    #[test]
    fn flashlight_skill_changes_mock_state() {
        let registry = registry();
        let platform = MockPlatform::new(80).unwrap();
        execute_with_platform(&registry, &intent("flashlight.on"), &platform).unwrap();
        assert!(platform.snapshot().unwrap().flashlight_enabled);
    }

    #[test]
    fn wifi_skill_changes_mock_state() {
        let registry = registry();
        let platform = MockPlatform::new(80).unwrap();
        execute_with_platform(&registry, &intent("wifi.enable"), &platform).unwrap();
        assert!(platform.snapshot().unwrap().wifi_enabled);
    }

    #[test]
    fn bluetooth_skill_changes_mock_state() {
        let registry = registry();
        let platform = MockPlatform::new(80).unwrap();
        execute_with_platform(&registry, &intent("bluetooth.enable"), &platform).unwrap();
        assert!(platform.snapshot().unwrap().bluetooth_enabled);
    }

    #[test]
    fn skills_require_platform_context() {
        let skill = BatterySkill;
        assert_eq!(
            skill.execute(&intent("battery.status")),
            Err(RuntimeError::SkillExecutionFailed(
                "platform context required".into()
            ))
        );
    }

    #[test]
    fn mock_platform_trait_is_used() {
        let platform = MockPlatform::new(64).unwrap();
        assert_eq!(platform.battery_percent().unwrap(), 64);
    }
}
