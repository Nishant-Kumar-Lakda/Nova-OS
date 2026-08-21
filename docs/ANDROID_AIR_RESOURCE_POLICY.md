# Android → AIR Resource Policy

NOVA's Android shell can read a small, permission-free resource snapshot:

```text
available RAM
battery percentage
low-memory flag
low-power flag
```

AIR converts that snapshot into a model-residency budget.

## Policy

```text
Normal battery + normal memory
    → allow a larger local model budget

Battery saver / <20% battery
    → reduce model residency

Critical battery / <10%
    → keep only the smallest models resident

Low memory
    → reduce residency further and prefer eviction
```

The policy is deliberately conservative. A model is never given the entire available RAM pool because Android, the UI, and other applications still need memory.

The Android resource snapshot can later be passed to the Rust AIR policy through the native bridge without giving AIR direct access to Android APIs.
