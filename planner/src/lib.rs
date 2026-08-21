use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeState {
    Pending,
    Ready,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionNode {
    pub id: String,
    pub action: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub state: NodeState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionGraph {
    pub id: String,
    pub nodes: Vec<ActionNode>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlannerError {
    #[error("graph id cannot be empty")]
    EmptyGraphId,
    #[error("node id cannot be empty")]
    EmptyNodeId,
    #[error("action cannot be empty")]
    EmptyAction,
    #[error("duplicate node: {0}")]
    DuplicateNode(String),
    #[error("unknown dependency: {node} -> {dependency}")]
    UnknownDependency { node: String, dependency: String },
    #[error("action graph contains a dependency cycle")]
    DependencyCycle,
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("invalid node state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: NodeState, to: NodeState },
}

impl ActionGraph {
    pub fn new(id: impl Into<String>) -> Result<Self, PlannerError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(PlannerError::EmptyGraphId);
        }
        Ok(Self {
            id,
            nodes: Vec::new(),
        })
    }

    pub fn add_node(&mut self, mut node: ActionNode) -> Result<(), PlannerError> {
        if node.id.trim().is_empty() {
            return Err(PlannerError::EmptyNodeId);
        }
        if node.action.trim().is_empty() {
            return Err(PlannerError::EmptyAction);
        }
        if self.nodes.iter().any(|existing| existing.id == node.id) {
            return Err(PlannerError::DuplicateNode(node.id));
        }
        node.state = NodeState::Pending;
        self.nodes.push(node);
        self.validate()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), PlannerError> {
        let ids: HashSet<&str> = self.nodes.iter().map(|node| node.id.as_str()).collect();

        for node in &self.nodes {
            if node.id.trim().is_empty() {
                return Err(PlannerError::EmptyNodeId);
            }
            if node.action.trim().is_empty() {
                return Err(PlannerError::EmptyAction);
            }
            for dependency in &node.depends_on {
                if !ids.contains(dependency.as_str()) {
                    return Err(PlannerError::UnknownDependency {
                        node: node.id.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }

        self.topological_order().map(|_| ())
    }

    pub fn ready_nodes(&self) -> Vec<ActionNode> {
        self.nodes
            .iter()
            .filter(|node| node.state == NodeState::Pending)
            .filter(|node| {
                node.depends_on.iter().all(|dependency| {
                    self.node(dependency)
                        .map(|parent| parent.state == NodeState::Succeeded)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .map(|mut node| {
                node.state = NodeState::Ready;
                node
            })
            .collect()
    }

    pub fn transition(
        &mut self,
        node_id: &str,
        next_state: NodeState,
    ) -> Result<(), PlannerError> {
        let node = self
            .nodes
            .iter_mut()
            .find(|node| node.id == node_id)
            .ok_or_else(|| PlannerError::NodeNotFound(node_id.to_string()))?;

        let current = node.state.clone();
        if !valid_transition(&current, &next_state) {
            return Err(PlannerError::InvalidStateTransition {
                from: current,
                to: next_state,
            });
        }

        node.state = next_state;
        Ok(())
    }

    pub fn node(&self, node_id: &str) -> Option<&ActionNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn topological_order(&self) -> Result<Vec<String>, PlannerError> {
        let mut indegree: HashMap<&str, usize> = self
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), 0))
            .collect();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            for dependency in &node.depends_on {
                *indegree
                    .get_mut(node.id.as_str())
                    .expect("node exists in indegree") += 1;
                dependents
                    .entry(dependency.as_str())
                    .or_default()
                    .push(node.id.as_str());
            }
        }

        let mut ready: Vec<&str> = self
            .nodes
            .iter()
            .filter(|node| indegree[node.id.as_str()] == 0)
            .map(|node| node.id.as_str())
            .collect();
        ready.reverse();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(node_id) = ready.pop() {
            order.push(node_id.to_string());
            if let Some(children) = dependents.get(node_id) {
                for child in children {
                    let entry = indegree
                        .get_mut(child)
                        .expect("dependent exists in indegree");
                    *entry -= 1;
                    if *entry == 0 {
                        ready.push(child);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(PlannerError::DependencyCycle);
        }

        Ok(order)
    }
}

fn valid_transition(from: &NodeState, to: &NodeState) -> bool {
    match from {
        NodeState::Pending => matches!(to, NodeState::Ready | NodeState::Cancelled | NodeState::Skipped),
        NodeState::Ready => matches!(to, NodeState::Running | NodeState::Cancelled | NodeState::Skipped),
        NodeState::Running => matches!(to, NodeState::Succeeded | NodeState::Failed | NodeState::Cancelled),
        NodeState::Succeeded | NodeState::Failed | NodeState::Cancelled | NodeState::Skipped => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, action: &str, depends_on: Vec<&str>) -> ActionNode {
        ActionNode {
            id: id.into(),
            action: action.into(),
            parameters: serde_json::json!({}),
            depends_on: depends_on.into_iter().map(str::to_owned).collect(),
            state: NodeState::Pending,
        }
    }

    #[test]
    fn rejects_empty_graph_id() {
        assert_eq!(ActionGraph::new(""), Err(PlannerError::EmptyGraphId));
    }

    #[test]
    fn rejects_duplicate_node() {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        graph.add_node(node("a", "files.find", vec![])).unwrap();
        assert_eq!(
            graph.add_node(node("a", "files.read", vec![])),
            Err(PlannerError::DuplicateNode("a".into()))
        );
    }

    #[test]
    fn rejects_unknown_dependency() {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        assert_eq!(
            graph.add_node(node("b", "files.read", vec!["missing"])),
            Err(PlannerError::UnknownDependency {
                node: "b".into(),
                dependency: "missing".into(),
            })
        );
    }

    #[test]
    fn detects_dependency_cycle() {
        let graph = ActionGraph {
            id: "cycle".into(),
            nodes: vec![
                node("a", "one", vec!["b"]),
                node("b", "two", vec!["a"]),
            ],
        };

        assert_eq!(graph.validate(), Err(PlannerError::DependencyCycle));
    }

    #[test]
    fn produces_topological_order() {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        graph.add_node(node("find", "files.find", vec![])).unwrap();
        graph
            .add_node(node("read", "files.read", vec!["find"]))
            .unwrap();
        graph
            .add_node(node("summarize", "text.summarize", vec!["read"]))
            .unwrap();

        assert_eq!(
            graph.topological_order().unwrap(),
            vec!["find", "read", "summarize"]
        );
    }

    #[test]
    fn ready_nodes_require_successful_dependencies() {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        graph.add_node(node("find", "files.find", vec![])).unwrap();
        graph
            .add_node(node("read", "files.read", vec!["find"]))
            .unwrap();

        assert_eq!(graph.ready_nodes()[0].id, "find");
        graph.transition("find", NodeState::Ready).unwrap();
        graph.transition("find", NodeState::Running).unwrap();
        graph.transition("find", NodeState::Succeeded).unwrap();

        assert_eq!(graph.ready_nodes()[0].id, "read");
    }

    #[test]
    fn enforces_node_state_machine() {
        let mut graph = ActionGraph::new("plan-1").unwrap();
        graph.add_node(node("a", "files.find", vec![])).unwrap();

        assert_eq!(
            graph.transition("a", NodeState::Running),
            Err(PlannerError::InvalidStateTransition {
                from: NodeState::Pending,
                to: NodeState::Running,
            })
        );

        graph.transition("a", NodeState::Ready).unwrap();
        graph.transition("a", NodeState::Running).unwrap();
        graph.transition("a", NodeState::Succeeded).unwrap();

        assert_eq!(
            graph.transition("a", NodeState::Running),
            Err(PlannerError::InvalidStateTransition {
                from: NodeState::Succeeded,
                to: NodeState::Running,
            })
        );
    }
}
