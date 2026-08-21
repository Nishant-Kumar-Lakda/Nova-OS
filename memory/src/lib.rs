use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryKind {
    Fact,
    Preference,
    Task,
    Episode,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    pub id: String,
    pub kind: MemoryKind,
    pub subject: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub confidence: f32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryError {
    #[error("memory id cannot be empty")]
    EmptyId,
    #[error("memory subject cannot be empty")]
    EmptySubject,
    #[error("memory content cannot be empty")]
    EmptyContent,
    #[error("memory already exists: {0}")]
    DuplicateId(String),
    #[error("memory not found: {0}")]
    NotFound(String),
    #[error("memory confidence must be between 0 and 1")]
    InvalidConfidence,
}

pub trait MemoryStore {
    fn put(&mut self, record: MemoryRecord) -> Result<(), MemoryError>;
    fn get(&self, id: &str) -> Option<&MemoryRecord>;
    fn delete(&mut self, id: &str) -> Result<MemoryRecord, MemoryError>;
    fn search(&self, query: &MemoryQuery) -> Vec<MemoryRecord>;
}

#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub subject: Option<String>,
    pub kind: Option<MemoryKind>,
    pub tag: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Default)]
pub struct InMemoryStore {
    records: HashMap<String, MemoryRecord>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl MemoryStore for InMemoryStore {
    fn put(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        validate_record(&record)?;
        if self.records.contains_key(&record.id) {
            return Err(MemoryError::DuplicateId(record.id));
        }
        self.records.insert(record.id.clone(), record);
        Ok(())
    }

    fn get(&self, id: &str) -> Option<&MemoryRecord> {
        self.records.get(id)
    }

    fn delete(&mut self, id: &str) -> Result<MemoryRecord, MemoryError> {
        self.records
            .remove(id)
            .ok_or_else(|| MemoryError::NotFound(id.to_string()))
    }

    fn search(&self, query: &MemoryQuery) -> Vec<MemoryRecord> {
        let mut results: Vec<_> = self
            .records
            .values()
            .filter(|record| {
                query
                    .subject
                    .as_ref()
                    .map(|subject| record.subject.eq_ignore_ascii_case(subject))
                    .unwrap_or(true)
            })
            .filter(|record| query.kind.map(|kind| record.kind == kind).unwrap_or(true))
            .filter(|record| {
                query
                    .tag
                    .as_ref()
                    .map(|tag| record.tags.iter().any(|value| value.eq_ignore_ascii_case(tag)))
                    .unwrap_or(true)
            })
            .filter(|record| {
                query
                    .text
                    .as_ref()
                    .map(|text| {
                        let text = text.to_ascii_lowercase();
                        record.content.to_ascii_lowercase().contains(&text)
                            || record.subject.to_ascii_lowercase().contains(&text)
                    })
                    .unwrap_or(true)
            })
            .cloned()
            .collect();

        results.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        results
    }
}

fn validate_record(record: &MemoryRecord) -> Result<(), MemoryError> {
    if record.id.trim().is_empty() {
        return Err(MemoryError::EmptyId);
    }
    if record.subject.trim().is_empty() {
        return Err(MemoryError::EmptySubject);
    }
    if record.content.trim().is_empty() {
        return Err(MemoryError::EmptyContent);
    }
    if !record.confidence.is_finite() || !(0.0..=1.0).contains(&record.confidence) {
        return Err(MemoryError::InvalidConfidence);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, kind: MemoryKind, subject: &str, content: &str, updated_at: u64) -> MemoryRecord {
        MemoryRecord {
            id: id.into(),
            kind,
            subject: subject.into(),
            content: content.into(),
            tags: vec!["work".into()],
            confidence: 0.9,
            created_at: updated_at,
            updated_at,
        }
    }

    #[test]
    fn stores_and_reads_memory() {
        let mut store = InMemoryStore::new();
        store
            .put(record("m1", MemoryKind::Fact, "Rahul", "Rahul works in sales", 1))
            .unwrap();

        assert_eq!(store.len(), 1);
        assert_eq!(store.get("m1").unwrap().subject, "Rahul");
    }

    #[test]
    fn rejects_duplicate_id() {
        let mut store = InMemoryStore::new();
        store
            .put(record("m1", MemoryKind::Fact, "Rahul", "sales", 1))
            .unwrap();

        assert_eq!(
            store.put(record("m1", MemoryKind::Fact, "Rahul", "sales", 2)),
            Err(MemoryError::DuplicateId("m1".into()))
        );
    }

    #[test]
    fn filters_memory_by_subject_and_tag() {
        let mut store = InMemoryStore::new();
        store
            .put(record("m1", MemoryKind::Fact, "Rahul", "sales", 1))
            .unwrap();
        store
            .put(record("m2", MemoryKind::Fact, "Priya", "engineering", 2))
            .unwrap();

        let query = MemoryQuery {
            subject: Some("rahul".into()),
            tag: Some("WORK".into()),
            ..MemoryQuery::default()
        };

        let results = store.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "m1");
    }

    #[test]
    fn returns_newest_memory_first() {
        let mut store = InMemoryStore::new();
        store
            .put(record("old", MemoryKind::Note, "project", "old", 1))
            .unwrap();
        store
            .put(record("new", MemoryKind::Note, "project", "new", 2))
            .unwrap();

        let results = store.search(&MemoryQuery {
            subject: Some("project".into()),
            ..MemoryQuery::default()
        });

        assert_eq!(results[0].id, "new");
    }

    #[test]
    fn deletes_memory() {
        let mut store = InMemoryStore::new();
        store
            .put(record("m1", MemoryKind::Note, "n", "text", 1))
            .unwrap();

        let removed = store.delete("m1").unwrap();
        assert_eq!(removed.id, "m1");
        assert!(store.is_empty());
    }
}
