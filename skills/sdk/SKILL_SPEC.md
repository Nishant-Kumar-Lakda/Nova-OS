# NOVA Skill SDK v0.1

A Skill is a capability exposed to the NOVA runtime. Skills are isolated from NEXUS and must not execute privileged platform operations directly; they request those operations through the platform adapter.

## Manifest

```json
{
  "id": "nova.core.example",
  "version": "0.1.0",
  "actions": ["example.run"],
  "permissions": ["example.permission"]
}
```

### Required fields

- `id`: globally unique skill identifier.
- `version`: semantic version.
- `actions`: NIL actions supported by the skill.
- `permissions`: capabilities required by the skill.

## Lifecycle

```text
Discover → Verify → Register → Validate → Execute → Result
```

A future runtime will verify signatures before registration.

## Safety rules

1. A skill may only execute actions declared in its manifest.
2. A skill must declare every required permission.
3. The runtime owns authorization and confirmation decisions.
4. Skills must return structured results.
5. Platform-specific code belongs behind the platform abstraction.
6. Skills should be deterministic where possible and idempotent where practical.

## Result contract

```json
{
  "success": true,
  "data": {},
  "error": null
}
```
