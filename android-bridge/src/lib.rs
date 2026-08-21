use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::JNIEnv;
use nova_air::backend::EchoBackend;
use nova_air::{ResourcePolicy, ResourceSnapshot};
use nova_core::NovaEngine;
use nova_nexus::parse;

/// JNI entry point used by the Android shell.
///
/// The bridge converts Java text into a validated NIL JSON object. It cannot
/// execute Android operations and performs no network access.
#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeUnderstand(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let result = match env.get_string(&input) {
        Ok(value) => match parse(value.to_str().unwrap_or_default()) {
            Ok(intent) => serde_json::json!({
                "ok": true,
                "intent": intent,
            })
            .to_string(),
            Err(error) => error_json(error.to_string().as_str()),
        },
        Err(error) => error_json(error.to_string().as_str()),
    };

    match env.new_string(result) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// JNI entry point for Android resource-aware AIR budgeting.
///
/// Android supplies read-only resource measurements; AIR decides how much
/// memory may be dedicated to resident local models.
#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeRecommendModelBudget(
    _env: JNIEnv,
    _class: JClass,
    available_memory_bytes: jlong,
    battery_percent: jint,
    low_memory: jboolean,
    low_power: jboolean,
) -> jlong {
    if available_memory_bytes < 0 || !(0..=100).contains(&battery_percent) {
        return 0;
    }

    let snapshot = ResourceSnapshot {
        available_memory_bytes: available_memory_bytes as u64,
        battery_percent: battery_percent as u8,
        low_memory: low_memory != 0,
        low_power: low_power != 0,
    };

    ResourcePolicy::default().recommended_model_budget(snapshot) as jlong
}

/// Boots the complete Rust NOVA Core in a deterministic development profile.
/// This is a diagnostics endpoint: it verifies that Core, AIR, Planner,
/// Memory, Context, Runtime, and built-in Skills can initialize together.
#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeBootDiagnostics(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let result = match NovaEngine::new(128 * 1024 * 1024, EchoBackend, 16) {
        Ok(mut nova) => match nova.register_builtin_skills() {
            Ok(()) => serde_json::json!({
                "ok": true,
                "core": true,
                "air": true,
                "planner": true,
                "memory": true,
                "context": true,
                "runtime": true,
                "skills": true,
            })
            .to_string(),
            Err(error) => serde_json::json!({
                "ok": false,
                "error": error.to_string(),
            })
            .to_string(),
        },
        Err(error) => serde_json::json!({
            "ok": false,
            "error": error.to_string(),
        })
        .to_string(),
    };

    match env.new_string(result) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn error_json(message: &str) -> String {
    serde_json::json!({
        "ok": false,
        "error": message,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_nil_for_supported_command() {
        let intent = parse("check battery").unwrap();
        assert_eq!(intent.action, "battery.status");
    }

    #[test]
    fn rejects_unknown_command() {
        let error = parse("do something random").unwrap_err();
        assert_eq!(error.to_string(), "unsupported command");
    }

    #[test]
    fn android_style_budget_is_resource_aware() {
        let normal = ResourceSnapshot {
            available_memory_bytes: 1_000_000_000,
            battery_percent: 80,
            low_memory: false,
            low_power: false,
        };
        let low_power = ResourceSnapshot {
            available_memory_bytes: 1_000_000_000,
            battery_percent: 15,
            low_memory: false,
            low_power: true,
        };

        let policy = ResourcePolicy::default();
        assert!(
            policy.recommended_model_budget(low_power)
                < policy.recommended_model_budget(normal)
        );
    }

    #[test]
    fn nova_core_can_boot_with_builtins() {
        let mut nova = NovaEngine::new(128 * 1024 * 1024, EchoBackend, 16).unwrap();
        assert!(nova.register_builtin_skills().is_ok());
    }
}
