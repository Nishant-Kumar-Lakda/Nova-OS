use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputSource {
    Text,
    Voice,
    Camera,
    Notification,
    Shortcut,
    SystemEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInput {
    pub source: InputSource,
    pub text: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum InputError {
    #[error("input text cannot be empty")]
    EmptyText,
}

impl UserInput {
    pub fn new(source: InputSource, text: impl Into<String>, timestamp_ms: u64) -> Result<Self, InputError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(InputError::EmptyText);
        }
        Ok(Self {
            source,
            text,
            timestamp_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_text_input() {
        let input = UserInput::new(InputSource::Text, "check battery", 123).unwrap();
        assert_eq!(input.source, InputSource::Text);
        assert_eq!(input.text, "check battery");
    }

    #[test]
    fn supports_non_text_sources() {
        let voice = UserInput::new(InputSource::Voice, "open camera", 100).unwrap();
        let camera = UserInput::new(InputSource::Camera, "read this", 200).unwrap();
        assert_eq!(voice.source, InputSource::Voice);
        assert_eq!(camera.source, InputSource::Camera);
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(
            UserInput::new(InputSource::Text, "  ", 0),
            Err(InputError::EmptyText)
        );
    }
}
