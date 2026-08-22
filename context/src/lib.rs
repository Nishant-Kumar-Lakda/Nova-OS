use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityKind {
    Person,
    File,
    App,
    Location,
    Device,
    Topic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextEntity {
    pub kind: EntityKind,
    pub name: String,
    pub reference: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextSnapshot {
    pub current_app: Option<String>,
    pub active_plan: Option<String>,
    pub recent_entities: Vec<ContextEntity>,
    pub last_user_input: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContextError {
    #[error("entity name cannot be empty")]
    EmptyEntityName,
    #[error("entity reference cannot be empty")]
    EmptyReference,
}

#[derive(Debug)]
pub struct ContextEngine {
    current_app: Option<String>,
    active_plan: Option<String>,
    recent_entities: VecDeque<ContextEntity>,
    last_user_input: Option<String>,
    max_entities: usize,
}

impl ContextEngine {
    pub fn new(max_entities: usize) -> Self {
        Self {
            current_app: None,
            active_plan: None,
            recent_entities: VecDeque::new(),
            last_user_input: None,
            max_entities: max_entities.max(1),
        }
    }

    pub fn set_current_app(&mut self, app: impl Into<String>) {
        self.current_app = Some(app.into());
    }

    pub fn set_active_plan(&mut self, plan: impl Into<String>) {
        self.active_plan = Some(plan.into());
    }

    pub fn set_user_input(&mut self, input: impl Into<String>) {
        self.last_user_input = Some(input.into());
    }

    pub fn remember(&mut self, entity: ContextEntity) -> Result<(), ContextError> {
        if entity.name.trim().is_empty() {
            return Err(ContextError::EmptyEntityName);
        }
        if entity.reference.trim().is_empty() {
            return Err(ContextError::EmptyReference);
        }

        self.recent_entities.retain(|existing| {
            !(existing.kind == entity.kind && existing.name.eq_ignore_ascii_case(&entity.name))
        });
        self.recent_entities.push_front(entity);
        while self.recent_entities.len() > self.max_entities {
            self.recent_entities.pop_back();
        }
        Ok(())
    }

    pub fn resolve_last(&self, kind: EntityKind) -> Option<&ContextEntity> {
        self.recent_entities
            .iter()
            .find(|entity| entity.kind == kind)
    }

    pub fn snapshot(&self) -> ContextSnapshot {
        ContextSnapshot {
            current_app: self.current_app.clone(),
            active_plan: self.active_plan.clone(),
            recent_entities: self.recent_entities.iter().cloned().collect(),
            last_user_input: self.last_user_input.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(name: &str, reference: &str, timestamp: u64) -> ContextEntity {
        ContextEntity {
            kind: EntityKind::Person,
            name: name.into(),
            reference: reference.into(),
            timestamp,
        }
    }

    #[test]
    fn remembers_and_resolves_latest_person() {
        let mut context = ContextEngine::new(4);
        context
            .remember(person("Rahul", "contact:rahul", 1))
            .unwrap();
        context
            .remember(person("Priya", "contact:priya", 2))
            .unwrap();

        assert_eq!(
            context.resolve_last(EntityKind::Person).unwrap().name,
            "Priya"
        );
    }

    #[test]
    fn replaces_duplicate_entity() {
        let mut context = ContextEngine::new(4);
        context.remember(person("Rahul", "contact:old", 1)).unwrap();
        context
            .remember(person("rahul", "contact:new", 2))
            .unwrap();

        let entity = context.resolve_last(EntityKind::Person).unwrap();
        assert_eq!(entity.reference, "contact:new");
        assert_eq!(context.snapshot().recent_entities.len(), 1);
    }

    #[test]
    fn enforces_context_capacity() {
        let mut context = ContextEngine::new(2);
        context.remember(person("A", "a", 1)).unwrap();
        context.remember(person("B", "b", 2)).unwrap();
        context.remember(person("C", "c", 3)).unwrap();

        let snapshot = context.snapshot();
        assert_eq!(snapshot.recent_entities.len(), 2);
        assert_eq!(snapshot.recent_entities[0].name, "C");
        assert_eq!(snapshot.recent_entities[1].name, "B");
    }

    #[test]
    fn tracks_application_and_plan() {
        let mut context = ContextEngine::new(2);
        context.set_current_app("messages");
        context.set_active_plan("send-message-1");
        context.set_user_input("send it to him");

        let snapshot = context.snapshot();
        assert_eq!(snapshot.current_app.as_deref(), Some("messages"));
        assert_eq!(snapshot.active_plan.as_deref(), Some("send-message-1"));
        assert_eq!(snapshot.last_user_input.as_deref(), Some("send it to him"));
    }
}
