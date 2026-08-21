# NOVA Android Prototype 0.2 — Android First

This is the Android-first NOVA phone prototype. The Android application is a thin host around the NOVA architecture, with Rust NEXUS/AIR/Core supplied through JNI plus a deterministic fallback when the native library is unavailable.

## Runtime architecture

```text
Android UI
   ↓
NovaAndroidEngine
   ↓
Task Session + Safety Gate
   ↓
Rust NEXUS / NIL (JNI)
   ↓
NOVA Core / AIR
   ↓
Local SmolLM2 model when needed
   ↓
Android Platform Adapter
   ↓
Device
```

## Offline AI

The first generative model is **SmolLM2 135M Instruct, GGUF Q2_K**. It is approximately 88 MB and is executed locally through a pinned llama.cpp build. The model is fetched during the native Android build and packaged into the APK, so the installed application does not need network access to use it.

The model manifest is `models/manifests/smollm2-135m-android-q2k.json`.

The model is a fallback layer: deterministic NEXUS handles known OS intents first; unsupported natural-language requests can be answered by the local model. The model is never allowed to directly execute Android APIs.

## Safety boundary

The prototype:

- Uses no `INTERNET` permission.
- Uses no Accessibility Service.
- Uses no Device Admin / Device Owner privileges.
- Uses no background service.
- Does not change Wi-Fi or Bluetooth state.
- Does not toggle the flashlight.
- Does not read contacts, SMS, notifications, files, microphone, or camera data.
- Executes only battery status and benign Settings/Camera launches.
- Only allowlisted app aliases can be launched (`settings`, `camera`, `calculator`).
- Unknown or non-allowlisted app requests are rejected.

Wi-Fi, Bluetooth, and flashlight commands are simulation-only.

## Run locally

Install Android Studio, Android SDK/NDK, Rust, Git, and CMake. From the repository root run:

```text
scripts/prepare_android_model.sh
```

That script pins llama.cpp to the exact commit recorded in the model manifest and fetches the GGUF model into the generated Android assets directory. Then open `android/` in Android Studio, allow Gradle to sync, enable USB debugging on a spare/test Android phone, select the phone, and run the `app` debug configuration.

The repository deliberately does not commit the GGUF binary or llama.cpp source tree.

## Run the native CI build

The `NOVA Android Native` GitHub Actions workflow builds the Rust JNI bridge and the llama.cpp native layer for:

- `arm64-v8a`
- `armeabi-v7a`
- `x86_64`
- `x86`

It fetches the model, packages the native libraries and model into the debug APK, verifies that cloud/privileged permissions are absent from the APK manifest, and uploads the installable APK artifact.

Install the resulting APK with:

```text
adb install -r app-debug.apk
```

## First commands

```text
battery status
open settings
open camera
open calculator
ask NOVA something it does not understand
turn on flashlight       # simulation only
turn on Wi-Fi            # simulation only
turn on Bluetooth        # simulation only
```

## Core diagnostics

The **Run Core Diagnostics** button reports Rust NEXUS, NOVA Core, AIR, built-in skills, local model bridge, RAM, battery, and model-store state.

## Security release gate

Do not add Accessibility, Device Admin, notification access, contacts, SMS, microphone, or network permissions to this prototype without a separate security review and explicit test plan.
