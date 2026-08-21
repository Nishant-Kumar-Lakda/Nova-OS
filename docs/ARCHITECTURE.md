# NOVA OS Architecture v0.1

## 1. System Goal

Build an offline-first AI operating environment for resource-constrained phones and PCs. The system must remain useful without network access and must avoid keeping large generative models resident when a small deterministic component can complete the task.

## 2. Execution Pipeline

```text
Input
  -> NEXUS
  -> NIL validation
  -> Context resolution
  -> Planner (only when required)
  -> Permission policy
  -> AIR scheduler
  -> Skill
  -> Platform adapter
  -> Result
```

## 3. NEXUS

NEXUS converts natural language into a constrained intent representation. The first implementation will use deterministic rules so that the protocol and execution pipeline can be tested before selecting a neural model.

## 4. NIL

NIL is the stable contract between language understanding and execution. It is versioned independently from models. A NIL action contains an action identifier, typed parameters, optional context, confidence, and execution constraints.

## 5. AIR

AIR is the local AI/runtime layer. Responsibilities:

- model lifecycle and manifests;
- RAM/CPU/battery-aware scheduling;
- model caching and eviction;
- inference backend abstraction;
- specialist model loading on demand;
- execution telemetry without network telemetry;
- capability and permission checks.

## 6. Planner

The planner converts complex requests into an action graph. Planning is separate from execution. The planner cannot directly call privileged platform APIs.

## 7. Skills

Skills are isolated capabilities. A skill declares its actions, input schema, required permissions, resource requirements, and optional rollback behavior. Skills are the only layer allowed to request platform operations through approved adapters.

## 8. Memory

Memory is local and structured first. SQLite is the baseline storage engine. Vector retrieval may be added later for semantic document retrieval, but it is not a requirement for core device control.

## 9. Platform Layer

The core must not depend directly on Android or Linux APIs. Platform adapters implement device operations behind a stable interface.

Initial targets:

1. Android.
2. Linux desktop/mobile.

## 10. Resource Strategy

The default strategy is hierarchical:

- tiny wake/intent component remains resident;
- planner model is loaded only for complex requests;
- specialist models such as OCR or vision are loaded on demand;
- inactive models are evicted according to memory pressure;
- model streaming is optional and must be benchmarked against device storage latency and battery cost.

## 11. Security Boundary

Models are untrusted inputs. A model may propose an action, but it cannot bypass permission checks or directly execute privileged operations. Skill and model manifests must be validated before activation.

## 12. First Vertical Slice

The first end-to-end slice is intentionally small:

```text
"turn on flashlight"
        -> NEXUS
        -> flashlight.on
        -> permission check
        -> flashlight skill
        -> platform adapter
        -> success
```

The same pipeline will then support Wi-Fi, Bluetooth, battery status, and app launching.
