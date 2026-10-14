# Dot20 effective settings — source handoff

Root authored `src/uar/api/effective_settings.rs`, API registration/server route,
read-only provider/native/MCP inspection callbacks, and actual embedding endpoint
accessors in the three existing remote backends. Provider read lock order follows
existing removal (providers then default), retained through native read lock and
synchronous MCP slot/governance inspection. No settings mutation authority changed.

Pinned Kubo schema/provenance and AG-UI source-only research are under
`implementation/protocol-sources/`. Endpoint documentation is
`docs/operations/effective-settings.md`; real HTTP fixture helpers are
`tests/fixtures/dot20_effective_settings.py`. The fixture does not launch fake
backends and has not run. Final runner must supply actual installations, signed
admin/non-admin credentials, configuration changes/restarts and schema validator.

The API requires verified Bearer identity with admin role, returns401/403 as
appropriate, excludes credential/header/env/argument material and hashes only
its effective public projection. Fixed adapters expose actual endpoint hosts;
unknown backend provenance fails unavailable. Native tools report actual
registered CodeExecution capabilities. Stored restart-only preferences remain
inactive until services are reconstructed. Scope is global defaults plus current
tool inventory; explicit owner/run overrides retain separate admission.

Source complete; runtime behavior, lock ordering under all mutation paths,
Axum composition, schema validation, hidden secret markers and revision changes
remain unverified. No test/check/compiler/build/lint/format/validator/product
review ran. Full-phase gate remains closed. No unrequested feature was added;
authentication, secret projection and unavailable-origin guards trace directly
to issue379 item8 and the existing host/identity boundary.
