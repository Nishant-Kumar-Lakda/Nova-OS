use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlatformError {
    #[error("capability unavailable: {0}")]
    CapabilityUnavailable(String),
    #[error("operation denied: {0}")]
    PermissionDenied(String),
    #[error("invalid value: {0}")]
    InvalidValue(String),
}

/// Small capability interface implemented by Android, Linux, Windows, and test platforms.
/// No platform-specific APIs belong in NOVA's core crates.
pub trait Platform: Send + Sync {
    fn open_app(&self, app: &str) -> Result<(), PlatformError>;
    fn battery_percent(&self) -> Result<u8, PlatformError>;
    fn flashlight(&self, enabled: bool) -> Result<(), PlatformError>;
    fn wifi(&self, enabled: bool) -> Result<(), PlatformError>;
    fn bluetooth(&self, enabled: bool) -> Result<(), PlatformError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockPlatformState {
    pub last_app: Option<String>,
    pub battery_percent: u8,
    pub flashlight_enabled: bool,
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
}

/// Safe, side-effect-free platform used for development and integration tests.
#[derive(Clone)]
pub struct MockPlatform {
    state: Arc<Mutex<MockPlatformState>>,
}

impl MockPlatform {
    pub fn new(battery_percent: u8) -> Result<Self, PlatformError> {
        if battery_percent > 100 {
            return Err(PlatformError::InvalidValue(
                "battery percentage must be 0..=100".into(),
            ));
        }
        Ok(Self {
            state: Arc::new(Mutex::new(MockPlatformState {
                last_app: None,
                battery_percent,
                flashlight_enabled: false,
                wifi_enabled: false,
                bluetooth_enabled: false,
            })),
        })
    }

    pub fn snapshot(&self) -> Result<MockPlatformState, PlatformError> {
        self.state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| PlatformError::CapabilityUnavailable("platform state lock poisoned".into()))
    }
}

impl Platform for MockPlatform {
    fn open_app(&self, app: &str) -> Result<(), PlatformError> {
        if app.trim().is_empty() {
            return Err(PlatformError::InvalidValue("app cannot be empty".into()));
        }
        self.state
            .lock()
            .map_err(|_| PlatformError::CapabilityUnavailable("platform state lock poisoned".into()))?
            .last_app = Some(app.trim().to_string());
        Ok(())
    }

    fn battery_percent(&self) -> Result<u8, PlatformError> {
        Ok(self.snapshot()?.battery_percent)
    }

    fn flashlight(&self, enabled: bool) -> Result<(), PlatformError> {
        self.state
            .lock()
            .map_err(|_| PlatformError::CapabilityUnavailable("platform state lock poisoned".into()))?
            .flashlight_enabled = enabled;
        Ok(())
    }

    fn wifi(&self, enabled: bool) -> Result<(), PlatformError> {
        self.state
            .lock()
            .map_err(|_| PlatformError::CapabilityUnavailable("platform state lock poisoned".into()))?
            .wifi_enabled = enabled;
        Ok(())
    }

    fn bluetooth(&self, enabled: bool) -> Result<(), PlatformError> {
        self.state
            .lock()
            .map_err(|_| PlatformError::CapabilityUnavailable("platform state lock poisoned".into()))?
            .bluetooth_enabled = enabled;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_battery_range() {
        assert!(MockPlatform::new(101).is_err());
        assert!(MockPlatform::new(100).is_ok());
    }

    #[test]
    fn records_platform_state() {
        let platform = MockPlatform::new(72).unwrap();
        platform.open_app("camera").unwrap();
        platform.flashlight(true).unwrap();
        platform.wifi(true).unwrap();
        platform.bluetooth(false).unwrap();

        let state = platform.snapshot().unwrap();
        assert_eq!(state.last_app.as_deref(), Some("camera"));
        assert_eq!(state.battery_percent, 72);
        assert!(state.flashlight_enabled);
        assert!(state.wifi_enabled);
        assert!(!state.bluetooth_enabled);
    }

    #[test]
    fn rejects_empty_app_name() {
        let platform = MockPlatform::new(50).unwrap();
        assert_eq!(
            platform.open_app("  "),
            Err(PlatformError::InvalidValue("app cannot be empty".into()))
        );
    }
}
