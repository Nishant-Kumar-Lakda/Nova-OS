# NOVA Skill Specification v0.1

A skill exposes safe, typed capabilities to the NOVA runtime.

## Required Properties

- stable skill ID;
- action IDs;
- input schema;
- required permissions;
- resource requirements;
- execution result schema.

## Boundary

A skill receives validated NIL actions. It must not interpret natural language. It must not bypass the platform permission layer. It must not execute arbitrary shell commands from NIL parameters.

## Example

```yaml
id: flashlight
version: 0.1
actions:
  - id: flashlight.on
    permissions:
      - device.flashlight
  - id: flashlight.off
    permissions:
      - device.flashlight
```
