## Context

See `proposal.md` for motivation and the change specs for externally visible behavior. The current settings panel derives the correct hyphenated URL, but the runtime never registers or mounts that namespace. User settings are process-local despite comments claiming persistence, null and omission collapse during decoding, and normal chat computes an effective caching flag without carrying it into dispatch. Anthropic-compatible handling can also install a default cache strategy independently of the resolved policy. Session configuration already has canonical/draft entity boundaries and owner-scoped persistence that this change must preserve.

## Goals / Non-Goals

**Goals:**

- Establish one typed effective policy with a source, shared by APIs and provider dispatch.
- Persist nullable session and user overrides without changing legacy-record behavior.
- Make Anthropic cache controls mechanically depend on effective On and keep OpenAI request construction unchanged.
- Keep the settings UI authoritative under load and failure while extending Session Configuration through its existing entity domain.
- Produce deterministic local evidence before installing and restarting the macOS service.

**Non-Goals:**

- Configurable Anthropic TTL or cache breakpoint placement.
- A switch that claims to disable OpenAI's provider-managed caching.
- New agent-level caching configuration or new toolbar controls.
- Broad changes to session ownership, authentication, provider routing, or unrelated settings panels.

## Decisions

### Use a resolved policy value instead of passing booleans independently

Introduce an effective prompt-caching value containing the boolean and a closed source enum. Resolve request, session, user, and global inputs once at the request boundary and pass the value through policy-bearing execution. This prevents API display and driver behavior from implementing different precedence. Keeping separate resolution in each handler was rejected because it already caused the normal-chat flag to be computed and discarded.

### Represent persisted overrides as nullable booleans

Add `Option<bool>` / `boolean | null` to run policy, agent-session configuration, and entity contracts. Missing fields deserialize to `None`, so existing records inherit without migration rewrites. A three-value enum was rejected because it would duplicate the existing wire meaning and complicate backward-compatible JSON.

### Preserve four user-update states with a nested optional decoder

Decode the user update field so outer absence means preserve and inner null means clear. The API converts verified identity into a stable principal key: tenant plus subject when tenant is present, otherwise subject. Display identity remains separate from the storage key. Plain `Option<bool>` was rejected because Serde maps both omission and null to `None`.

### Add user settings to the existing persistence boundary

Extend `PersistenceLayer` with save/load operations for the existing user settings record, then implement those operations for in-memory, Postgres, and Surreal providers. `UserSettingsStore` retains a read-through/write-through memory cache and receives an optional persistence handle. A standalone database abstraction was rejected because it would duplicate configured-backend selection and lifecycle wiring.

### Use one cache-strategy construction seam at LLM dispatch

Construct `CacheStrategy` only from an effective policy at the point where a provider request becomes an `LlmRequest`, and preserve it through initial, iterative, compatibility, and failover calls. Native Anthropic routing consumes `Some(strategy)` only for On and `None` for Off. OpenAI and liter-llm-compatible request serialization ignore the UAR toggle. Per-handler request mutation and a driver-level default were rejected because both can bypass policy precedence.

### Return absence without ownership disclosure

Owner session configuration reads return an exact empty 204 both when the record is missing and when the requester cannot access it. The write path retains existing ownership checks. The frontend recognizes both 204 and legacy 404 as absence. Returning a descriptive cross-owner error was rejected because it provides an ownership oracle.

### Keep UI state inside existing feature boundaries

The global panel continues through the settings feature store/API pipeline and renders no editable fallback before a successful load. Session tri-state state is added to the canonical/draft entity contracts and domain actions; the component does not fetch or own server-confirmed values. The global control uses the existing shadcn switch and the session control uses the existing shadcn select primitives. Adding a component-local business-state copy was rejected because it would violate the project entity-state contract.

### Treat browser extension failures as separately attributable evidence

Application browser certification uses an extension-free profile and rejects app-origin console errors. The repository MV3 extension is tested separately for connected and disconnected message handling. User-installed extension errors are classified only when the console source is a `chrome-extension://` URL; application code will not suppress generic promise errors.

## Risks / Trade-offs

- **[Risk] User-settings persistence methods enlarge every backend contract.** → Keep the record schema narrow and add backend-specific reload/isolation tests before integrating the API.
- **[Risk] A missed LLM constructor silently bypasses caching policy.** → Inventory production constructors and add stub-body assertions for each named path plus a constructor scan in review evidence.
- **[Risk] Native Anthropic selection changes fallback behavior.** → Gate the driver branch at compile time and add parity/fallback tests; do not alter model normalization or failover order.
- **[Risk] Empty 204 hides both absence and unauthorized access from the UI.** → This is intentional anti-enumeration behavior; mutation authorization remains unchanged and non-absence failures stay visible.
- **[Risk] Live provider cache creation may be unavailable without credentials or stable reuse timing.** → Treat live evidence as supplemental and make mocked upstream body/usage assertions release-gating.
- **[Risk] OpenAI users may expect the UAR toggle to force caching.** → Label the provider as automatic/provider-managed in UI and docs and prove the toggle does not change its request body.

## Migration Plan

1. Register and seed the global setting before exposing the panel route.
2. Add nullable fields with defaults, persistence operations, and backward-compatible response decoding.
3. Mount global, user, session-effective, and empty-absence contracts with focused API tests.
4. Route effective policy through LLM dispatch, remove unconditional Anthropic defaults, and verify upstream request bodies.
5. Ship the authoritative settings panel, entity-backed session tri-state, and documentation.
6. Run local tier checks and browser certification, then package and install the macOS binary. Record source and installed hashes before restarting the LaunchAgent.

Rollback installs the previous packaged binary and restarts the LaunchAgent. New nullable JSON fields remain safe for older readers, and the global default is Off, so rollback does not require destructive data migration.
