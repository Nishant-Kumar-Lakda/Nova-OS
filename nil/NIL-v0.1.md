# NOVA Intent Language (NIL) v0.1

NIL is the machine-facing protocol between intent understanding and execution.

## Canonical Shape

```json
{
  "version": "0.1",
  "action": "flashlight.on",
  "parameters": {},
  "context": {},
  "confidence": 0.99,
  "constraints": {}
}
```

## Fields

- `version`: NIL protocol version.
- `action`: namespaced action identifier.
- `parameters`: typed action arguments.
- `context`: optional references needed to resolve ambiguous language.
- `confidence`: model/rule confidence in `[0,1]`; execution policy decides whether confirmation is required.
- `constraints`: optional limits such as timeout or confirmation requirement.

## Naming

Actions use lowercase dotted namespaces:

- `flashlight.on`
- `flashlight.off`
- `wifi.enable`
- `wifi.disable`
- `bluetooth.enable`
- `bluetooth.disable`
- `battery.status`
- `app.open`

## Safety

NIL is declarative. It does not contain executable code, shell commands, arbitrary URLs, or platform API calls.

A dispatcher must validate the action against the installed skill registry before execution.

## Confidence Policy

- `>= 0.95`: may execute automatically for low-risk actions.
- `0.75 - 0.949`: confirmation is normally required.
- `< 0.75`: clarification is required.

High-risk actions will require explicit confirmation regardless of confidence.

## Compatibility

NIL must remain backwards-compatible within a major protocol version. Models may change without changing the execution contract.
