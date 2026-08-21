# NOVA Android-First Strategy

NOVA will target Android before other platforms.

## Why Android first

Android gives us a real mobile environment where memory, CPU, battery, storage bandwidth, permissions, and lifecycle behavior can be measured early. The same core architecture remains platform-independent, but Android is the first production target.

## Current Android boundary

```text
Android UI
   ↓
NovaAndroidEngine
   ↓
Task Session
   ↓
NOVA Command Router (temporary deterministic NEXUS stand-in)
   ↓
Safety Gate
   ↓
Android Platform Adapter
```

The current adapter intentionally permits only:

- Battery status
- Android Settings launch
- Camera launch

Flashlight, Wi-Fi, and Bluetooth state changes remain simulations.

## Rust integration target

The Java engine is temporary. The next bridge is:

```text
Android UI
   ↓
JNI / NDK boundary
   ↓
NOVA Core (Rust)
   ├── NEXUS
   ├── Context
   ├── Memory
   ├── Planner
   ├── AIR
   └── Skill Runtime
   ↓
Android Platform Adapter
```

The Java layer should eventually become a thin presentation/platform shell. Business logic should move to the Rust core.

## Safety requirements

The first Android prototype must remain a normal application. It must not become a launcher, accessibility service, device-admin application, VPN, or background agent until the core execution and permission model have been validated.

## Model strategy

The Android target is offline-first. A tiny local model will be introduced only after the deterministic pipeline is stable. The runtime must be able to choose between rule-based execution, a small intent model, and a larger planner model based on task complexity and device resources.
