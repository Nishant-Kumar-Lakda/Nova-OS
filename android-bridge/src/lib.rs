use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring};
use jni::JNIEnv;
use nova_air::{AirSnapshot, ModelManager, ResourcePolicy};
use nova_core::{NovaEngine, NovaError};
use nova_nexus::{EchoBackend, Intent};
use nova_runtime::RuntimeMode;
use serde_json::json;

fn to_jstring(env: &mut JNIEnv, value: String) -> jstring {
    env.new_string(value)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeRecommendedModelBudget(
    mut env: JNIEnv,
    _class: JClass,
    available_memory_bytes: jlong,
    battery_percent: jint,
    low_memory: jboolean,
    low_power: jboolean,
) -> jlong {
    let snapshot = AirSnapshot {
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
    env: JNIEnv,
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

    to_jstring(&mut env, result)
}

#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeRunIntent(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let input = match env.get_string(&input) {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(error) => {
            return to_jstring(&mut env, json!({"ok": false, "error": error.to_string()}).to_string())
        }
    };

    let intent = Intent::new(input);
    let mut nova = match NovaEngine::new(128 * 1024 * 1024, EchoBackend, 16) {
        Ok(nova) => nova,
        Err(error) => return to_jstring(&mut env, nova_error_json(error)),
    };

    let result = match nova.execute_intent(intent) {
        Ok(result) => json!({
            "ok": true,
            "result": result,
            "mode": RuntimeMode::Deterministic,
        })
        .to_string(),
        Err(error) => nova_error_json(error),
    };

    to_jstring(&mut env, result)
}

fn nova_error_json(error: NovaError) -> String {
    json!({
        "ok": false,
        "error": error.to_string(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_budget_decreases_in_low_power_mode() {
        let normal = AirSnapshot {
            available_memory_bytes: 4 * 1024 * 1024 * 1024,
            battery_percent: 80,
            low_memory: false,
            low_power: false,
        };
        let low_power = AirSnapshot {
            low_power: true,
            ..normal
        };
        let policy = ResourcePolicy::default();
        assert!(
            policy.recommended_model_budget(low_power) < policy.recommended_model_budget(normal)
        );
    }
}
