# AIR Scheduler v0.1

AIR schedules local AI work independently from platform threads.

## Goals

- Prioritize user-visible work.
- Avoid running unnecessary models concurrently.
- Permit resumable work to pause under memory pressure.
- Keep scheduling deterministic and testable.

## Priority order

`Critical > High > Normal > Low > Background`

For equal priority, tasks are FIFO.

## Lifecycle

```text
Queued → Running → Completed
       ↘ Paused → Running
       ↘ Cancelled
```

A task may only be paused when it declares itself resumable.

## Current scope

The v0.1 scheduler manages logical task state and ordering. It does not yet create OS threads or invoke an inference backend. That separation lets us validate scheduling policy before coupling it to platform-specific execution.
