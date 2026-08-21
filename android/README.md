# NOVA Android Prototype 0.1

This is a deliberately constrained phone prototype for validating the NOVA interaction model.

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

Wi-Fi, Bluetooth, and flashlight commands are routed to simulation-only actions so they cannot accidentally alter device state.

## What it proves

```text
User text
   ↓
Deterministic NOVA router
   ↓
Structured action
   ↓
Safety gate
   ↓
Safe Android operation OR simulation
```

This app is intentionally not the final NOVA UI and does not contain a cloud model.

## Build

Open the `android/` directory in Android Studio, allow Gradle to sync, then run the `app` debug configuration on a spare/test Android device.

The first test commands are:

- `battery status`
- `open settings`
- `open camera`
- `turn on flashlight`
- `turn on Wi-Fi`
- `turn on Bluetooth`

Only the first three execute a real operation; the last three must report `SIMULATION ONLY`.

## Release gate

Do not add Accessibility, Device Admin, notification access, contacts, SMS, microphone, or network permissions to this prototype without a separate security review and explicit test plan.
