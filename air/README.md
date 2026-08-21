# AIR — Artificial Intelligence Runtime

AIR owns local AI model lifecycle and resource policy for NOVA.

## v0.1

Implemented:

- Model registration
- Model metadata
- Memory budgeting
- Load/unload lifecycle
- Least-recently-used eviction
- Residency snapshots
- Unit-test coverage for lifecycle and edge cases

Not implemented yet:

- Actual inference
- Quantized model loading
- Weight streaming
- CPU/GPU/NPU scheduling
- Android/Linux model backends

The separation is intentional: resource policy should remain independent of the inference engine.
