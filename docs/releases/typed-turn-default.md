# Typed turn assembly default

Fresh installations and configurations that omit `harness.mode` now use
`typed` assembly. Explicit existing mode settings remain effective.

## Rollback

Set `harness.mode` to `legacy` in runtime settings, or in configuration:

```yaml
harness:
  mode: legacy
```

The environment equivalent is `UAR_HARNESS__MODE=legacy`. The legacy path is
deprecated but remains selectable for one minor release after this default
change; removal requires a separate change.

`shadow` remains an opt-in diagnostic mode. It assembles both paths for comparison
and dispatches only the legacy request. Its extra work is local assembly and
comparison, not a second inference request.

## Evidence and limitations

The pre-flip corpus covers three cases; the live k3 smoke covers basic input and
host instructions. Both recorded zero unexpected differences. Receipts are in
`openspec/changes/typed-turn-default-flip/`. The uncomfortable thing is the narrow
coverage: these receipts do not prove live parity for every provider, active
skill, memory, MCP, multi-step tool, or remote-child combination. Full local
phase-end tests are required before this change is considered verified.
