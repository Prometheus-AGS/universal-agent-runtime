# Effective runtime settings

`GET /api/uar/settings/effective` accepts a verified `Authorization: Bearer` token
whose signed roles contain `admin`. Missing/invalid identity returns401; a verified
non-admin returns403. No `x-uar-admin-key` is required for this read. Existing
settings mutations retain their existing authority. A launch token or a forged
role-shaped context does not substitute for verified Bearer identity.

The response matches Kubo's effective settings schema pinned at commit
`d5dea2115ad59ed7b12df561e5b085b562df8674`,
`mcp/contract/agentic/uar-effective-settings.schema.json`. Schema revision1 reports:

- `llm`: actual live registry default model/provider and endpoint host; disabled
  when no usable default is selected. It never chooses the first catalog model.
- `knowledge_base_embedder`: actual constructed embedding backend model and host.
- `memory_embedder`: actual MemoryService configuration; disabled if absent.
- `governance.enabled`: the effective governance status, not stored preference.
  `tool_approval: per_run` means artifact/run/descriptor admission determines the
  decision; there is no misleading global Auto promise.
- `native_tools`: currently registered terminal/code-execution descriptors.
- `mcp_servers`: configured/live transport names, endpoint URLs or executable
  command names. Arguments, environment, headers and credentials are omitted;
  URL userinfo, query and fragment are removed.
- `revision`: SHA-256 of this ordered public projection before inserting the hash.
  It changes with any represented effective value, not secret rotation alone.

Loopback/in-process embedding is `local`; unavailable embedding is `disabled`.
Remote hosts include explicit nondefault ports. Arbitrary unknown backend origins
return503 instead of claiming local execution. Memory adapter hosts are fixed by
the pinned library implementation. Explicit per-owner/run overrides retain their
own admission and are outside the runtime-default selection shown here.

The provider/default and native registries stay read-locked through the MCP
transport snapshot and final synchronous governance sample. That final sample is
the common snapshot point; there is no detached settings-row mirror. Failure to
read an owning authority returns503. Embedding services are the actual immutable
instances constructed at startup. Restart-applied stored preferences do not
change this read until restart installs them. In particular, enabling terminal
execution through its restart setting is not reported active prematurely.

Source implementation and real HTTP fixture are authored. No compilation,
validator, schema execution or production test has run. The complete Dot gate
must verify Bearer401/403/admin, model/provider updates, terminal restart changes,
revision changes, secret omission, CORS and published artifact behavior. The
uncomfortable thing is that source composition has not yet demonstrated those
properties in a running release.
