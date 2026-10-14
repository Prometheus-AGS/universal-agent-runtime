# Safe host defaults

New local installations bind HTTP and gRPC to `127.0.0.1`, require JWT authentication, select no LLM model, and leave agent memory disabled. If memory is explicitly enabled without a provider override, its embedder is `local` with `BAAI/bge-small-en-v1.5`. Local embedding may download model weights on first use; leaving memory disabled does not initialize its embedder. Knowledge-base embedding is a separate setting.

The uncomfortable thing: a copied profile is not proof of an installation's behavior. A saved YAML file, persisted provider selection, service environment, legacy `LLM_MODEL`, or supervisor override can change the effective behavior. Installable artifact and actual socket/egress/auth acceptance remain deferred until the complete Dot implementation boundary.

## Credentials and models

Before first service startup, provision `UAR_SECURITY__JWT_SECRET` and `UAR_SECURITY__SETTINGS_ADMIN_KEY` in the service environment file. The native credential refresh preserves these existing entries; it does not import arbitrary secrets from the launching shell. Service environment locations are:

| Platform | Environment file |
|---|---|
| macOS | `~/.uar/service.env` |
| Linux | `/etc/uar/uar.env` |
| Windows | `%ProgramData%\Prometheus\UniversalAgentRuntime\config\uar.env` |

Protect the file using the service account's existing access permissions. There is no shipped signing secret. Authentication remains enabled and startup reports a configuration error until required secrets are provisioned. A valid JWT is still required on protected routes over loopback; public health and agent discovery routes retain their existing public contract.

Select a model deliberately using `llm.model`, `UAR_LLM__MODEL`, `--llm-model`, or an enabled provider's explicit `default_model`. A request may also explicitly select a configured model. API-key discovery, local proxy model discovery, and catalog enrichment supply provider inventory only. They no longer select a default model. Missing-model inference fails with a configuration diagnostic before the driver sends a request.

Existing operator YAML, provider `default_model`, persisted provider rows and persisted `llm.default_provider` remain authoritative. Native refresh adds absent provider inventory but does not rewrite saved Alibaba models or API-key selectors. To intentionally change a saved model, use the existing settings authority or edit the operator-owned configuration.

## Browser origins

`server.cors_allowed_origins` is a YAML list of exact browser origin strings. Its default is `[]`; the server emits no `Access-Control-Allow-Origin` header for cross-origin requests. Same-origin browser access and non-browser API clients do not require an allowlist entry. CORS does not replace JWT authorization.

```yaml
server:
  host: "127.0.0.1"
  port: 1906
  cors_allowed_origins:
    - "https://console.example.com"
    - "http://localhost:3000"
```

Alternatively, set `UAR_SERVER__CORS_ALLOWED_ORIGINS=https://console.example.com,http://localhost:3000`. Use comma-separated values without surrounding spaces. Leave this variable unset for the empty default; use `cors_allowed_origins: []` in YAML to explicitly clear a file setting. Include the origin's scheme and port where applicable, without a path or trailing slash. Wildcards, `null`, empty entries, and invalid HTTP header values are rejected. Matching is exact, with no suffix matching or reflected-any-origin fallback. Public agent-card routes use the same CORS policy as the rest of the HTTP server.

## Deliberate shared-host overrides

The production Compose and Kubernetes profiles deliberately bind their container or pod listener to `0.0.0.0`. Their JWT requirement stays enabled, memory stays off by default, and they inherit the empty browser-origin allowlist. Configure the protected service environment or mounted YAML with the intended secrets and origins. Do not copy a test profile's anonymous settings into a shared installation.

For remote identity admission, explicitly use the existing `remote` policy and exact issuer/subject/tenant/workspace mappings. A wider socket alone does not establish tenant authority. Merge this example into the deployment's configuration, replace the identity placeholders, and supply both secrets separately:

```yaml
server:
  host: "0.0.0.0"
  port: 1906
  cors_allowed_origins: ["https://console.example.com"]
security:
  deployment_profile: remote
  jwt_required: true
  jwt_algorithm: HS256
  jwt_validate_nbf: true
  jwt_issuer: "https://identity.example.com"
  jwt_audience: "uar-shared"
  settings_mutation_auth_required: true
  workspace_authorities:
    - issuer: "https://identity.example.com"
      subject: "application-service"
      tenant_id: "tenant-a"
      workspace_id: "workspace-a"
memory:
  enabled: false
llm:
  model: ""
```

The model remains empty until selected by the operator. Configure one exact authority entry per admitted identity/workspace. An explicit remote memory embedder remains an operator decision and must include its provider credentials and matching vector dimensions.

## Deferred acceptance inventory

Run through the actual installed local and shared artifacts only after the lead records every Dot implementation item complete:

1. Fresh native profile: absent signing/admin secrets fail startup; supplied credentials start loopback listeners; unauthenticated protected requests return 401; memory stays off; inference with no model reports missing configuration without upstream traffic.
2. Explicit model profile: a configured model is used; restart and credential refresh preserve saved provider/default-model choices, including an older Alibaba model. Credentials alone and proxy `/models` discovery do not create a selection.
3. Empty origins: simple and preflight requests from an arbitrary browser origin have no allow-origin header. Public discovery routes do not bypass that rule.
4. Explicit origins: only the exact configured scheme/host/port matches. A different port, suffix domain, `null`, or unlisted origin receives no allow-origin header. Invalid configured values fail startup.
5. Shared profile: deliberate listener override, JWT 401/403/admitted behavior, exact tenant isolation, explicit allowed origin, no implicit model, and local/off memory behavior all agree with effective settings.
6. Upgrade: compare persisted provider IDs, default models, default-provider key and operator configuration before/after refresh; missing defaults may be added but explicit selections must be unchanged.

Record command, source revision, profile identity, installed artifact digest, actual result, and any egress/socket observations. Authored fixtures are not evidence of passing acceptance.
