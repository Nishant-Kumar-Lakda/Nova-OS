use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelManifest {
    pub id: String,
    pub version: String,
    pub format: String,
    pub size_bytes: u64,
    pub context_tokens: u32,
    pub capabilities: Vec<String>,
    pub minimum_memory_bytes: u64,
    pub architectures: Vec<String>,
    pub quantization: String,
    pub checksum: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("model id cannot be empty")]
    EmptyId,
    #[error("model format cannot be empty")]
    EmptyFormat,
    #[error("model checksum cannot be empty")]
    EmptyChecksum,
    #[error("model size must be greater than zero")]
    InvalidSize,
    #[error("model minimum memory cannot be zero")]
    InvalidMinimumMemory,
    #[error("model must declare at least one capability")]
    EmptyCapabilities,
    #[error("model must declare at least one architecture")]
    EmptyArchitectures,
}

impl ModelManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.id.trim().is_empty() {
            return Err(ManifestError::EmptyId);
        }
        if self.format.trim().is_empty() {
            return Err(ManifestError::EmptyFormat);
        }
        if self.checksum.trim().is_empty() {
            return Err(ManifestError::EmptyChecksum);
        }
        if self.size_bytes == 0 {
            return Err(ManifestError::InvalidSize);
        }
        if self.minimum_memory_bytes == 0 {
            return Err(ManifestError::InvalidMinimumMemory);
        }
        if self.capabilities.is_empty() {
            return Err(ManifestError::EmptyCapabilities);
        }
        if self.architectures.is_empty() {
            return Err(ManifestError::EmptyArchitectures);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ModelManifest {
        ModelManifest {
            id: "nova.intent.tiny".into(),
            version: "0.1.0".into(),
            format: "gguf".into(),
            size_bytes: 8 * 1024 * 1024,
            context_tokens: 1024,
            capabilities: vec!["intent".into()],
            minimum_memory_bytes: 32 * 1024 * 1024,
            architectures: vec!["arm64-v8a".into()],
            quantization: "q4".into(),
            checksum: "sha256:placeholder".into(),
        }
    }

    #[test]
    fn accepts_valid_manifest() {
        assert!(manifest().validate().is_ok());
    }

    #[test]
    fn rejects_empty_checksum() {
        let mut value = manifest();
        value.checksum.clear();
        assert_eq!(value.validate(), Err(ManifestError::EmptyChecksum));
    }

    #[test]
    fn rejects_zero_size() {
        let mut value = manifest();
        value.size_bytes = 0;
        assert_eq!(value.validate(), Err(ManifestError::InvalidSize));
    }
}
