# NOVA Android Prototype 0.2 — Android First

This is the Android-first NOVA phone prototype. The Android application is a thin host around the NOVA architecture, with Rust NEXUS/AIR/Core supplied through an optional JNI library and a deterministic fallback when that library is not packaged.

## Runtime architecture

```text
Android UI
   ↓
NovaAndroidEngine
   ↓
Task Session + Safety Gate
   ↓
Rust NEXUS / NIL (JNI, preferred)
   ↓
NOVA Core / AIR diagnostics
   ↓
Android Platform Adapter
   ↓
Device
```

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
- Only allowlisted app aliases can be launched (`settings`, `camera`, `calculator`, `browser`).
- Unknown commands are rejected.

Wi-Fi, Bluetooth, and flashlight commands are simulation-only so they cannot accidentally alter device state.

## Run locally

Open the `android/` directory in Android Studio, allow Gradle to sync, enable USB debugging on a spare/test Android phone, select the phone, and run the `app` debug configuration.

Without native libraries, the app runs using the safe deterministic Android fallback router.

## Run the native Rust build

The repository includes `.github/workflows/android-native.yml`. Run the **NOVA Android Native** workflow from GitHub Actions. It builds the Rust JNI bridge for:

- `arm64-v8a`
- `armeabi-v7a`
- `x86_64`
- `x86`

It packages those libraries into the debug APK and uploads the APK as the `nova-android-native-debug` artifact.

Install the resulting APK with Android Studio or:

```text
adb install -r app-debug.apk
```

The app status line should then report `NEXUS: Rust/JNI` and `Core: READY`.

## Core diagnostics

The **Run Core Diagnostics** button verifies that the packaged native library can initialize the Rust NOVA Core and built-in skills without contacting a network.

## First commands

```text
battery status
open settings
open camera
open calculator
open browser
turn on flashlight
turn on Wi-Fi
turn on Bluetooth
```

The final three remain simulation-only.

## Offline AI status

The architecture already includes AIR model residency, resource-aware budgeting, and a pluggable local inference backend. The current phone build intentionally uses the deterministic NEXUS bootstrap path until a measured tiny offline model is selected and packaged for the target phone class.

## Security release gate

Do not add Accessibility, Device Admin, notification access, contacts, SMS, microphone, or network permissions to this prototype without a separate security review and explicit test plan.
