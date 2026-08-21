# NOVA Local Models

NOVA is designed for offline, on-device inference. Model binaries are intentionally not committed to the source repository.

## Model roles

The first production phone profile will use small specialist models rather than one large always-resident model:

```text
Intent model       → short commands and classification
Planner model      → multi-step task decomposition
Summarizer model   → document/text summarization
Vision model       → optional camera/image tasks
```

AIR decides which model is needed and whether it fits the current RAM/battery budget.

## Bootstrap mode

The Android prototype currently operates with deterministic NEXUS rules and the `EchoBackend` test backend. This keeps the phone build fully offline and avoids shipping an unbenchmarked model.

## Model manifest contract

Each packaged model should provide:

- `id`
- `version`
- `format`
- `size_bytes`
- `context_tokens`
- `capabilities`
- `minimum_memory_bytes`
- `architectures`
- `quantization`
- `checksum`

The model loader must verify the manifest and checksum before registering a model with AIR.
