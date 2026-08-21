# NOVA OS

NOVA is an offline-first, AI-native operating system project.

## Vision

NOVA treats intelligence as a system resource. A lightweight intent layer handles common device commands, while an AI Runtime (AIR) loads planners and specialist models only when required.

## Design Principles

- Offline-first: core operation must not require cloud services.
- Intent-first: natural language is converted into structured actions.
- Deterministic execution: models propose actions; skills execute validated actions.
- Resource-aware: models are loaded, cached, suspended, and unloaded according to CPU/RAM/battery constraints.
- Platform-independent core: Android and Linux integration live behind platform adapters.
- Local privacy: user memory stays on-device by default.

## Current Architecture

```text
User Input
   |
   v
NEXUS (intent understanding)
   |
   v
NIL (structured intent)
   |
   v
Planner (multi-step tasks)
   |
   v
AIR (model/resource runtime)
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

- `runtime/` — AIR runtime and resource management.
- `nexus/` — intent understanding.
- `nil/` — NOVA Intent Language specification and schema.
- `planner/` — action graph planning.
- `memory/` — local memory subsystem.
- `skills/` — capability modules.
- `platform/` — Android/Linux platform adapters.
- `models/` — model manifests and metadata; model binaries are not committed.
- `tests/` — integration and conformance tests.
- `docs/` — architecture and design documents.

## Status

**v0.1 — Foundation**

The first milestone is a deterministic offline pipeline that can convert a small set of commands into validated NIL actions and dispatch them to skills.
