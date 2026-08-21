# NOVA OS

NOVA is an offline-first, AI-native operating system project.

## Vision

NOVA treats intelligence as a system resource. A lightweight intent layer handles common device commands, while the AI Runtime (AIR) loads planners and specialist models only when required.

## Design Principles

- Offline-first: core operation must not require cloud services.
- Intent-first: natural language is converted into structured actions.
- Deterministic execution: models propose actions; skills execute validated actions.
- Resource-aware: models are loaded, cached, suspended, and unloaded according to CPU/RAM/battery constraints.
- Platform-independent core: Android and Linux integration live behind platform adapters.
- Local privacy: user memory stays on-device by default.
- Small-model first: simple operations should bypass generative inference entirely.

## Current Architecture

```text
User Input
   |
   v
NEXUS (intent understanding)
   |
   v
Context (active entities / current task)
   |
   v
Planner (action graph)
   |
   v
NOVA Core (orchestration)
   |
   +------> Memory (local state)
   |
   +------> AIR (models / scheduling / inference)
   |
   v
Skill Runtime
   |
   v
Platform Adapter
   |
   v
Device / OS
```

## Repository Layout

- `core/` — top-level NOVA orchestration layer.
- `runtime/` — skill registration, validation, confidence policy, and dispatch.
- `nexus/` — intent understanding and NIL generation.
- `planner/` — dependency-aware multi-step action graphs.
- `memory/` — local memory abstraction and deterministic store.
- `context/` — active entities, current app, active plan, and recent user context.
- `air/` — model residency, scheduling, security, and inference backend abstraction.
- `nil/` — NOVA Intent Language specification and schema.
- `skills/` — capability modules and SDK specification.
- `platform/` — Android/Linux platform adapters.
- `models/` — model manifests and metadata; model binaries are not committed.
- `tests/` — integration and conformance tests.
- `docs/` — architecture and design documents.

## Development Strategy

NOVA is being built in layers. The core system is developed first; device integration and real model selection come afterward. Deterministic backends and local data structures provide the development substrate without requiring cloud services.

## Status

**v0.2 — System Core in development**

Current foundations include NEXUS, the skill runtime, AIR model management and inference interfaces, the action graph planner, local memory, active context, and the NOVA orchestration layer.
