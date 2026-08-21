use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use nova_nexus::parse;

/// JNI entry point used by the Android shell.
///
/// The bridge only converts Java text into a validated NIL JSON object. It
/// cannot execute Android operations and performs no network access.
#[no_mangle]
pub extern "system" fn Java_org_nova_os_NativeNovaBridge_nativeUnderstand(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    let result = match env.get_string(&input) {
        Ok(value) => match parse(value.to_str().unwrap_or_default()) {
            Ok(intent) => serde_json::to_string(&intent)
                .unwrap_or_else(|_| error_json("serialization_failed")),
            Err(error) => error_json(error.to_string().as_str()),
        },
        Err(error) => error_json(error.to_string().as_str()),
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
}
