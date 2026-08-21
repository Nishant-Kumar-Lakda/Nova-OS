# NOVA Platform API v0.1

The platform layer isolates NOVA from Android, Linux, Windows, and future targets.

## Architecture

```text
Skill
  ↓
NOVA Runtime
  ↓
Platform API
  ↓
Android / Linux / Windows
```

## Initial capabilities

- `device.flashlight`
- `device.battery`
- `device.wifi`
- `device.bluetooth`

The runtime and skills must not call platform-specific APIs directly. Each target implements the same capability contract.

## Design requirements

- Offline-first.
- No network access unless explicitly required by a future skill and approved by policy.
- Platform calls return structured errors.
- Permissions are enforced by the platform and runtime, not by the model.
- The API must be usable from Rust and callable from Android through a narrow FFI/JNI boundary.
