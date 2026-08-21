pub mod scheduler;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Metadata for a local AI model known to AIR.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSpec {
    pub id: String,
    pub version: String,
    pub size_bytes: u64,
    pub capabilities: Vec<String>,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelState {
    Registered,
    Loaded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSnapshot {
    pub id: String,
    pub state: ModelState,
    pub size_bytes: u64,
    pub last_used: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AirError {
    #[error("model id cannot be empty")]
    EmptyModelId,
    #[error("model already registered: {0}")]
    DuplicateModel(String),
    #[error("model not found: {0}")]
    ModelNotFound(String),
    #[error("model is too large for AIR memory budget: {0} bytes")]
    ModelExceedsBudget(u64),
    #[error("memory budget must be greater than zero")]
    InvalidBudget,
}

pub struct ModelManager {
    registry: HashMap<String, ModelSpec>,
    loaded: HashMap<String, LoadedModel>,
    memory_budget: u64,
    used_memory: u64,
    clock: u64,
}

#[derive(Debug, Clone)]
struct LoadedModel {
    size_bytes: u64,
    last_used: u64,
}

impl ModelManager {
    pub fn new(memory_budget: u64) -> Result<Self, AirError> {
        if memory_budget == 0 {
            return Err(AirError::InvalidBudget);
        }
        Ok(Self {
            registry: HashMap::new(),
            loaded: HashMap::new(),
            memory_budget,
            used_memory: 0,
            clock: 0,
        })
    }

    pub fn register(&mut self, model: ModelSpec) -> Result<(), AirError> {
        if model.id.trim().is_empty() {
            return Err(AirError::EmptyModelId);
        }
        if self.registry.contains_key(&model.id) {
            return Err(AirError::DuplicateModel(model.id));
        }
        self.registry.insert(model.id.clone(), model);
        Ok(())
    }

    pub fn memory_budget(&self) -> u64 {
        self.memory_budget
    }

    pub fn used_memory(&self) -> u64 {
        self.used_memory
    }

    pub fn available_memory(&self) -> u64 {
        self.memory_budget.saturating_sub(self.used_memory)
    }

    pub fn is_loaded(&self, model_id: &str) -> bool {
        self.loaded.contains_key(model_id)
    }

    pub fn load(&mut self, model_id: &str) -> Result<(), AirError> {
        let spec = self
            .registry
            .get(model_id)
            .ok_or_else(|| AirError::ModelNotFound(model_id.to_string()))?
            .clone();

        if spec.size_bytes > self.memory_budget {
            return Err(AirError::ModelExceedsBudget(spec.size_bytes));
        }

        self.clock = self.clock.saturating_add(1);

        if let Some(model) = self.loaded.get_mut(model_id) {
            model.last_used = self.clock;
            return Ok(());
        }

        while self.used_memory.saturating_add(spec.size_bytes) > self.memory_budget {
            let victim = self
                .loaded
                .iter()
                .min_by_key(|(_, loaded)| loaded.last_used)
                .map(|(id, _)| id.clone());

            match victim {
                Some(id) => self.unload_internal(&id),
                None => return Err(AirError::ModelExceedsBudget(spec.size_bytes)),
            }
        }

        self.used_memory = self.used_memory.saturating_add(spec.size_bytes);
        self.loaded.insert(
            model_id.to_string(),
            LoadedModel {
                size_bytes: spec.size_bytes,
                last_used: self.clock,
            },
        );

        Ok(())
    }

    pub fn unload(&mut self, model_id: &str) -> Result<(), AirError> {
        if !self.loaded.contains_key(model_id) {
            return Err(AirError::ModelNotFound(model_id.to_string()));
        }
        self.unload_internal(model_id);
        Ok(())
    }

    pub fn snapshot(&self) -> Vec<ModelSnapshot> {
        let mut snapshots: Vec<_> = self
            .registry
            .values()
            .map(|spec| {
                if let Some(loaded) = self.loaded.get(&spec.id) {
                    ModelSnapshot {
                        id: spec.id.clone(),
                        state: ModelState::Loaded,
                        size_bytes: spec.size_bytes,
                        last_used: loaded.last_used,
                    }
                } else {
                    ModelSnapshot {
                        id: spec.id.clone(),
                        state: ModelState::Registered,
                        size_bytes: spec.size_bytes,
                        last_used: 0,
                    }
                }
            })
            .collect();
        snapshots.sort_by(|a, b| a.id.cmp(&b.id));
        snapshots
    }

    fn unload_internal(&mut self, model_id: &str) {
        if let Some(model) = self.loaded.remove(model_id) {
            self.used_memory = self.used_memory.saturating_sub(model.size_bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &str, size: u64) -> ModelSpec {
        ModelSpec {
            id: id.into(),
            version: "0.1.0".into(),
            size_bytes: size,
            capabilities: vec!["intent".into()],
            path: format!("models/{id}.bin"),
        }
    }

    #[test]
    fn rejects_zero_budget() {
        assert_eq!(ModelManager::new(0), Err(AirError::InvalidBudget));
    }

    #[test]
    fn rejects_empty_model_id() {
        let mut manager = ModelManager::new(100).unwrap();
        assert_eq!(manager.register(model("", 10)), Err(AirError::EmptyModelId));
    }

    #[test]
    fn registers_model() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("intent", 30)).unwrap();
        assert!(!manager.is_loaded("intent"));
        assert_eq!(manager.used_memory(), 0);
    }

    #[test]
    fn rejects_duplicate_model() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("intent", 30)).unwrap();
        assert_eq!(
            manager.register(model("intent", 30)),
            Err(AirError::DuplicateModel("intent".into()))
        );
    }

    #[test]
    fn loads_and_unloads_model() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("intent", 30)).unwrap();

        manager.load("intent").unwrap();
        assert!(manager.is_loaded("intent"));
        assert_eq!(manager.used_memory(), 30);
        assert_eq!(manager.available_memory(), 70);

        manager.unload("intent").unwrap();
        assert!(!manager.is_loaded("intent"));
        assert_eq!(manager.used_memory(), 0);
    }

    #[test]
    fn loading_same_model_does_not_double_count_memory() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("intent", 30)).unwrap();
        manager.load("intent").unwrap();
        manager.load("intent").unwrap();
        assert_eq!(manager.used_memory(), 30);
    }

    #[test]
    fn unloading_unknown_model_returns_error() {
        let mut manager = ModelManager::new(100).unwrap();
        assert_eq!(
            manager.unload("missing"),
            Err(AirError::ModelNotFound("missing".into()))
        );
    }

    #[test]
    fn rejects_model_larger_than_budget() {
        let mut manager = ModelManager::new(50).unwrap();
        manager.register(model("large", 51)).unwrap();
        assert_eq!(manager.load("large"), Err(AirError::ModelExceedsBudget(51)));
        assert_eq!(manager.used_memory(), 0);
    }

    #[test]
    fn evicts_least_recently_used_model() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("a", 40)).unwrap();
        manager.register(model("b", 40)).unwrap();
        manager.register(model("c", 40)).unwrap();

        manager.load("a").unwrap();
        manager.load("b").unwrap();
        manager.load("a").unwrap();
        manager.load("c").unwrap();

        assert!(manager.is_loaded("a"));
        assert!(manager.is_loaded("c"));
        assert!(!manager.is_loaded("b"));
        assert_eq!(manager.used_memory(), 80);
    }

    #[test]
    fn snapshot_reports_deterministic_residency() {
        let mut manager = ModelManager::new(100).unwrap();
        manager.register(model("planner", 50)).unwrap();
        manager.register(model("intent", 30)).unwrap();
        manager.load("planner").unwrap();

        let snapshots = manager.snapshot();
        assert_eq!(snapshots[0].id, "intent");
        assert_eq!(snapshots[0].state, ModelState::Registered);
        assert_eq!(snapshots[1].id, "planner");
        assert_eq!(snapshots[1].state, ModelState::Loaded);
    }
}
