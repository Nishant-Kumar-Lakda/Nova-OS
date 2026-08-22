#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    Normal,
    BatterySaver,
    CriticalBattery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub available_memory_bytes: u64,
    pub battery_percent: u8,
    pub low_memory: bool,
    pub low_power: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePolicy {
    pub hard_memory_cap_bytes: u64,
    pub normal_memory_ratio_percent: u8,
    pub low_memory_ratio_percent: u8,
    pub battery_saver_ratio_percent: u8,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            hard_memory_cap_bytes: 512 * 1024 * 1024,
            normal_memory_ratio_percent: 35,
            low_memory_ratio_percent: 20,
            battery_saver_ratio_percent: 15,
        }
    }
}

impl ResourcePolicy {
    pub fn power_mode(&self, snapshot: ResourceSnapshot) -> PowerMode {
        if snapshot.battery_percent < 10 {
            PowerMode::CriticalBattery
        } else if snapshot.low_power || snapshot.battery_percent < 20 {
            PowerMode::BatterySaver
        } else {
            PowerMode::Normal
        }
    }

    pub fn recommended_model_budget(&self, snapshot: ResourceSnapshot) -> u64 {
        let ratio = match self.power_mode(snapshot) {
            PowerMode::Normal => self.normal_memory_ratio_percent,
            PowerMode::BatterySaver => self.battery_saver_ratio_percent,
            PowerMode::CriticalBattery => 5,
        };

        let memory_based = snapshot.available_memory_bytes.saturating_mul(ratio as u64) / 100;

        let low_memory_cap = if snapshot.low_memory {
            snapshot
                .available_memory_bytes
                .saturating_mul(self.low_memory_ratio_percent as u64)
                / 100
        } else {
            memory_based
        };

        low_memory_cap.min(self.hard_memory_cap_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(memory: u64, battery: u8, low_memory: bool, low_power: bool) -> ResourceSnapshot {
        ResourceSnapshot {
            available_memory_bytes: memory,
            battery_percent: battery,
            low_memory,
            low_power,
        }
    }

    #[test]
    fn normal_devices_get_normal_budget() {
        let policy = ResourcePolicy::default();
        let result = policy.recommended_model_budget(snapshot(1_000_000_000, 80, false, false));
        assert_eq!(result, 350_000_000);
    }

    #[test]
    fn low_battery_reduces_model_budget() {
        let policy = ResourcePolicy::default();
        let result = policy.recommended_model_budget(snapshot(1_000_000_000, 15, false, true));
        assert_eq!(result, 150_000_000);
    }

    #[test]
    fn low_memory_mode_reduces_budget_further() {
        let policy = ResourcePolicy::default();
        let result = policy.recommended_model_budget(snapshot(1_000_000_000, 80, true, false));
        assert_eq!(result, 200_000_000);
    }

    #[test]
    fn hard_cap_is_respected() {
        let policy = ResourcePolicy::default();
        let result = policy.recommended_model_budget(snapshot(4_000_000_000, 80, false, false));
        assert_eq!(result, 512 * 1024 * 1024);
    }

    #[test]
    fn critical_battery_uses_minimal_budget() {
        let policy = ResourcePolicy::default();
        let result = policy.recommended_model_budget(snapshot(1_000_000_000, 5, false, false));
        assert_eq!(result, 50_000_000);
        assert_eq!(
            policy.power_mode(snapshot(1_000_000_000, 5, false, false)),
            PowerMode::CriticalBattery
        );
    }
}
