# NOVA Android Phone Prototype Runbook

## Goal

Build and install the Android-first NOVA prototype on a spare/test Android phone without granting privileged device-control permissions.

## 1. Prepare the phone

Enable **Developer options** and **USB debugging**. Connect the phone to the development PC and accept the USB debugging prompt.

Verify the device:

```text
adb devices
```

A single device should appear as `device`.

## 2. Build the native NOVA APK

In GitHub, open:

```text
Actions → NOVA Android Native → Run workflow
```

The workflow builds the Rust bridge for:

```text
arm64-v8a
armeabi-v7a
x86_64
x86
```

and packages the native libraries into the debug APK.

## 3. Download the APK

From the successful workflow run, download the artifact:

```text
nova-android-native-debug
```

Extract `app-debug.apk`.

## 4. Install

```text
adb install -r app-debug.apk
```

If Android blocks the install, enable installation through the current USB/file source in the phone's security settings and retry.

## 5. First boot

Open **NOVA OS**.

The status area should show approximately:

```text
NEXUS: Rust/JNI
Core: READY
RAM: <device value>
Battery: <device value>
AIR budget: <device value>
```

Press **Run Core Diagnostics**. The expected result is:

```text
Core: READY
AIR: READY
Planner: READY
Memory: READY
Context: READY
Runtime: READY
Built-in Skills: READY
```

## 6. Safe command test

Test these in order:

```text
battery status
open settings
open camera
open calculator
turn on flashlight
turn on Wi-Fi
turn on Bluetooth
open browser
```

Expected behavior:

- Battery reports the local battery percentage.
- Settings opens Android Settings.
- Camera requests the Android camera application.
- Calculator opens only if the allowlisted calculator package exists.
- Flashlight, Wi-Fi, and Bluetooth return simulation-only results.
- Browser is rejected by the offline prototype policy.

## 7. What the phone is actually running

When the native artifact is installed, the execution path is:

```text
Android UI
  ↓
NovaAndroidEngine
  ↓
Rust JNI
  ↓
NEXUS / NIL
  ↓
NOVA Core boot diagnostics
  ↓
AIR resource policy
  ↓
Android safety gate
  ↓
Android platform adapter
```

The fallback Java router remains available so the APK can still open and demonstrate the UI if the native library is unavailable.

## 8. Current limitation

This is an Android application prototype, not yet a replacement Android launcher or a replacement kernel/operating system. The next major step is the first measured tiny offline generative model packaged into AIR and invoked only when deterministic NEXUS rules cannot satisfy a request.

Do not install this build as the phone's permanent launcher until the native execution, model runtime, persistence, crash recovery, and security review are complete.
