# NOVA Task Execution Lifecycle v0.1

NOVA now models a user request as a durable task session instead of a single function call.

```text
User Request
    ↓
TaskSession Created
    ↓
NEXUS Intent
    ↓
ActionGraph Plan
    ↓
Task Ready
    ↓
Task Running
    ↓
Ready Action Node
    ↓
Skill / AIR / Platform
    ↓
Node Succeeded or Failed
    ↓
More Nodes? ── yes ──→ next ready node
    │
    no
    ↓
Task Completed
```

## Why this exists

A simple assistant can execute one command. NOVA must execute a sequence of dependent actions while preserving state, errors, and progress.

A task stores:

- original user input
- normalized NIL intent
- action graph
- task state
- current action node
- failure information

## State machine

```text
Created → Planning → Ready → Running → Completed
                         │        │
                         │        ├── Failed
                         │        └── Cancelled
                         └──────────── Cancelled
```

Terminal states cannot transition again.

## Action-node execution

The task session asks the planner for the next ready node. Dependencies must already be successful before a node becomes executable.

The session marks the node `Running`, exposes it to the future skill/runtime executor, and then accepts either `Succeeded` or `Failed`.

## Safety boundary

Task planning does not execute anything. The execution path remains:

```text
NEXUS / Planner
      ↓
NIL
      ↓
Runtime validation
      ↓
Security policy
      ↓
Skill
      ↓
Platform capability
```

The platform crate contains only capability interfaces and a side-effect-free mock implementation for development.
