# NOVA Android Native Bridge

NOVA is Android-first. The Android application is a thin host around the NOVA core rather than the location of the intelligence itself.

## Current path

```text
Android UI
    ↓
NovaAndroidEngine
    ↓
Rust NEXUS (JNI, when native library is packaged)
    ↓
NIL JSON
    ↓
Android safety gate
    ↓
Android platform adapter
```

A deterministic Java router remains as a safe fallback until the native `.so` is available.

## Native library

Rust crate:

```text
android-bridge/
```

Library name:

```text
libnova_android_bridge.so
```

Supported ABIs in the native CI pipeline:

- arm64-v8a
- armeabi-v7a
- x86_64
- x86

## Boundary rule

The JNI bridge only performs intent understanding and NIL serialization. It does not call Android APIs, change device state, access the network, or hold Android permissions.

Android operations remain behind `AndroidPlatformAdapter` and the NOVA safety gate.

## Resource awareness

The Android shell exposes a read-only resource snapshot containing available RAM, total RAM, and battery percentage. AIR can use those measurements later to select or evict local models.

## Model strategy

The native bridge is deliberately independent from model loading. NEXUS can use deterministic rules while AIR is developed. A future tiny on-device model can be added behind the same interface without changing Android's safety boundary.
