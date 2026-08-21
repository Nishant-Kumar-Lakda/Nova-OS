# NOVA OS

NOVA is an offline-first, AI-native operating system project.

## Vision

NOVA treats intelligence as a system resource. A lightweight intent layer handles common device commands, while the AI Runtime (AIR) loads planners and specialist models only when required.

The long-term goal is not another chatbot or launcher. NOVA is intended to become an **intent-first operating environment** where the user expresses goals in natural language and the system turns those goals into safe, auditable actions across the device.

## Design Principles

- Offline-first: core operation must not require cloud services.
- Intent-first: natural language is converted into structured actions.
- Deterministic execution: models propose actions; skills execute validated actions.
- Resource-aware: models are loaded, cached, suspended, and unloaded according to CPU/RAM/battery constraints.
- Platform-independent core: Android, Linux, Windows, and future targets use the same capability contracts.
- Local privacy: user memory stays on-device by default.
- Small-model first: simple operations should bypass generative inference entirely.
- Safe experimentation: development uses mock platform capabilities before granting real device control.

## Current Architecture

```text
User Request
   |
   v
Task Session
   |
   v
NEXUS (intent understanding)
   |
   v
Context + Memory
   |
   v
Planner (dependency-aware action graph)
   |
   v
NOVA Core (orchestration)
   |
   +------> AIR (model residency / scheduling / inference)
   |
   +------> Runtime (skills / validation / confidence policy)
   |
   v
Security Policy
   |
   v
Platform Capability API
   |
   v
Android / Linux / Windows
```

## Repository Layout

- `core/` — top-level NOVA orchestration and task lifecycle.
- `runtime/` — skill registration, validation, confidence policy, and dispatch.
- `nexus/` — intent understanding and NIL generation.
- `planner/` — dependency-aware multi-step action graphs.
- `memory/` — local memory abstraction and deterministic store.
- `context/` — active entities, current app, active plan, and recent user context.
- `air/` — model residency, scheduling, security, and inference backend abstraction.
- `nil/` — NOVA Intent Language specification and schema.
- `skills/` — capability modules and SDK specification.
- `platform/` — platform capability contracts plus a safe mock implementation.
- `android/` — isolated phone prototype.
- `models/` — model manifests and metadata; model binaries are not committed.
- `tests/` — integration and conformance tests.
- `docs/` — architecture and design documents.

## Development Strategy

NOVA is being built from the inside out. The system architecture and contracts are established first, followed by real model integration and platform implementations.

The development stack intentionally includes deterministic backends and a mock platform so the core can be exercised without cloud services or device-side privileges.

## Current Core

The repository currently contains:

- NEXUS intent parsing.
- NIL v0.1 action format and confidence policy.
- Skill registry and dispatch runtime.
- AIR model registry, RAM accounting, LRU residency, scheduler, security policy, and backend abstraction.
- Dependency-aware planner/action graph.
- Local memory abstraction and context engine.
- NOVA Core orchestration.
- Task sessions with intent → plan → node execution → completion/failure lifecycle.
- Cross-platform capability API with a side-effect-free mock platform.
- A sandboxed Android prototype that does not grant dangerous device permissions.

## Status

**v0.2 — System Core in development**

The next phase is to connect the runtime to the platform capability API, replace deterministic intent rules with a tiny offline model, and then integrate the same Rust core into the safe Android prototype.
