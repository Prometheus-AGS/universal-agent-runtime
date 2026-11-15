# RC5 OpenSpec verification

All six tasks are complete for the selected Linux amd64 UAR API profile. The dedicated AG-UI schema and lifecycle checks enter through the production HTTP route in `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`. The mounted scenarios cover first delivery, legacy replay, action retries, undeclared action and remote function refusal, and deletion. Runtime implementation lives in `src/uar/api/agui.rs`, `src/uar/api/agui_a2ui.rs`, and `src/uar/a2ui/protocol.rs`.

The published native-chat receipt verifies two overlapping requests reconcile the original durable run and that the governed case produces one effect. Source provenance records the PEM successor, the published PEM 5 consumer and the A2UI 1.0 patches. The signed tag, release audit, OCI index and scenario receipts identify the same qualified release source and image.

[Release evidence](https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.4.0-rc.5)
