use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub version: String,
    pub action: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub context: serde_json::Value,
    pub confidence: f32,
    #[serde(default)]
    pub constraints: serde_json::Value,
}

#[derive(Debug, Error, PartialEq)]
pub enum IntentError {
    #[error("empty input")]
    EmptyInput,
    #[error("unsupported command")]
    UnsupportedCommand,
    #[error("invalid confidence")]
    InvalidConfidence,
}

pub fn parse(input: &str) -> Result<Intent, IntentError> {
    let text = input.trim().to_ascii_lowercase();
    if text.is_empty() {
        return Err(IntentError::EmptyInput);
    }

    let (action, confidence, parameters) = match text.as_str() {
        "turn on flashlight" | "turn on the flashlight" | "flashlight on" => {
            ("flashlight.on", 0.99, serde_json::json!({}))
        }
        "turn off flashlight" | "turn off the flashlight" | "flashlight off" => {
            ("flashlight.off", 0.99, serde_json::json!({}))
        }
        "turn on wifi" | "turn on wi-fi" | "enable wifi" | "enable wi-fi" => {
            ("wifi.enable", 0.99, serde_json::json!({}))
        }
        "turn off wifi" | "turn off wi-fi" | "disable wifi" | "disable wi-fi" => {
            ("wifi.disable", 0.99, serde_json::json!({}))
        }
        "turn on bluetooth" | "enable bluetooth" => {
            ("bluetooth.enable", 0.99, serde_json::json!({}))
        }
        "turn off bluetooth" | "disable bluetooth" => {
            ("bluetooth.disable", 0.99, serde_json::json!({}))
        }
        "battery status" | "show battery" | "check battery" => {
            ("battery.status", 0.99, serde_json::json!({}))
        }
        "open camera" | "launch camera" | "start camera" => {
            ("camera.open", 0.99, serde_json::json!({}))
        }
        "open settings" | "open system settings" | "show settings" => {
            ("settings.open", 0.99, serde_json::json!({}))
        }
        "open calculator" | "launch calculator" => {
            ("app.open", 0.99, serde_json::json!({"app": "calculator"}))
        }
        "open browser" | "launch browser" | "open web browser" => {
            ("app.open", 0.99, serde_json::json!({"app": "browser"}))
        }
        _ if text.starts_with("open ") => {
            let app = text.trim_start_matches("open ").trim();
            if app.is_empty() {
                return Err(IntentError::UnsupportedCommand);
            }
            ("app.open", 0.95, serde_json::json!({"app": app}))
        }
        _ => return Err(IntentError::UnsupportedCommand),
    };

    if !(0.0..=1.0).contains(&confidence) {
        return Err(IntentError::InvalidConfidence);
    }

    Ok(Intent {
        version: "0.1".into(),
        action: action.into(),
        parameters,
        context: serde_json::json!({}),
        confidence,
        constraints: serde_json::json!({}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flashlight_command() {
        let intent = parse("Turn on the flashlight").unwrap();
        assert_eq!(intent.action, "flashlight.on");
        assert_eq!(intent.confidence, 0.99);
    }

    #[test]
    fn parses_wifi_command() {
        let intent = parse("enable Wi-Fi").unwrap();
        assert_eq!(intent.action, "wifi.enable");
    }

    #[test]
    fn parses_camera_command() {
        let intent = parse("open camera").unwrap();
        assert_eq!(intent.action, "camera.open");
    }

    #[test]
    fn parses_generic_app_command() {
        let intent = parse("open music").unwrap();
        assert_eq!(intent.action, "app.open");
        assert_eq!(intent.parameters["app"], "music");
        assert_eq!(intent.confidence, 0.95);
    }

    #[test]
    fn rejects_empty_generic_app() {
        assert_eq!(parse("open "), Err(IntentError::UnsupportedCommand));
    }

    #[test]
    fn rejects_unknown_command() {
        assert_eq!(
            parse("do something complicated"),
            Err(IntentError::UnsupportedCommand)
        );
    }

    #[test]
    fn rejects_empty_command() {
        assert_eq!(parse("   "), Err(IntentError::EmptyInput));
    }
}
