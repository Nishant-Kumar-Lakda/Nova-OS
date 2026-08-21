# AIR v0.2

AIR is the NOVA Artificial Intelligence Runtime. It is responsible for model residency, inference abstraction, scheduling, and security policy.

## Model Residency

`ModelManager` tracks:

- Registered models
- Loaded models
- Memory budget
- Current memory usage
- Last-use timestamps
- Least-recently-used eviction

The runtime never assumes that all models fit in RAM.

## Inference Backend

AIR exposes `InferenceBackend` so NOVA is not tied to a specific inference engine.

```text
NOVA
  ↓
AIR InferenceEngine
  ↓
InferenceBackend
  ├── llama.cpp adapter (future)
  ├── ONNX Runtime adapter (future)
  ├── ExecuTorch adapter (future)
  └── mobile-native adapter (future)
```

`EchoBackend` is the deterministic development backend. It does not load a model and does not access the network.

## Security

Every platform action will eventually pass through an AIR security policy containing:

- Risk level
- Required permissions
- Confirmation requirement

The AI model itself never receives direct privileged access to the device.

## Mobile Goal

The first real mobile backend must support ARM CPUs and operate without network access. Model loading and inference must remain replaceable so we can benchmark multiple small model families without changing NEXUS, the planner, or skill code.
