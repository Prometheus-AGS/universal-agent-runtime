# Private fixture input preparation

Source authored only; phase verification gate CLOSED. Nothing here has been
imported or executed. No credential, file, directory, token, database, service,
network request or model request has been created by this implementation turn.

This program removes routine manual fixture input work after a future named
approval. It generates fresh fixture-only secrets, JWTs and datastore credential
inputs, derives source-backed owner identities, and privately copies the two
selected existing key values for the current Cohere/Liter recipe. It never sources a shell file, evaluates expressions,
modifies an operator profile, contacts a service, invokes a command or imports the
acceptance runner. It has no I/O on import.

The uncomfortable thing: generation creates credentials and temporary private
copies of existing provider secrets. This is an explicit future approval effect,
not a read-only composition step. An env.json is sensitive even though it is
mode600. The parent must keep it outside logs/reports and later perform the
specifically approved cleanup. This program never deletes an old scope to make a
retry appear fresh. Failure may leave a private partial scope; reconcile it.

## Future exact invocation and adoption boundary

`[frozen_python, absolute_prepare_private_inputs_py, absolute_private_config_json]`

No shell, flags, secret argv, or secret stdout. The parent captures one redacted
JSON receipt. It loads env.json in memory as a string-to-string environment map
for later approved child commands; do not print it, source it, put its values in
argv, or pass it to a shell. Loading this map and actual datastore/service creation
must be included in the future named packet. Preparation itself does not adopt an
artifact or open the verification gate.

Output scope is `<scope_parent>/uar_fixture_inputs_<sha256(epoch_id)[:20]>`.
The explicit absolute parent must already be private mode700. Scope is created
exclusively mode700; `postgres/`, `surreal/`, `packet/` children are private state
parents. Stable names are deterministic; secret values are fresh cryptographic
randomness. Reusing an existing epoch scope refuses instead of resetting anything.
Both backend cleanups must preserve packet/ and the common private inputs.

Files are exclusive mode600 JSON, at most1MiB, flushed and fsynced:

- env.json: sensitive generated values and only the selected existing keys.
- inputs.json: source-derived claims/identities, exact ports, security patch,
  named env references, future composition paths and unresolved input inventory.
- receipt.json: redacted names/paths, input/config digests, effects and source refs;
  identical to stdout. No secret values or hashes of individual secret values.

This does not invent charge ceilings, pricing, signing/adoption packets, budget
seeds, timestamps for prior consumption, or dispositions for unknown reservations.
Those remain explicitly named pending operator inputs. It never creates database
roles/databases, containers, images or services: generated credentials are inputs
for later separately approved owned datastore provisioning.

## Private configuration schema

The config is regular absolute nonsymlink mode600 JSON, at most1MiB:

```
{
  "schema_version": 1,
  "executable": true,
  "epoch_id": "EXACT_APPROVED_EPOCH",
  "scope_parent": "/existing/private/parent",
  "composition_output_dir": "/explicit/new/composition/directory",
  "selected_embedding_services": ["cohere"],
  "validity": {"not_before_unix": 0, "expires_unix": 0},
  "ports": {"postgres": 0, "surreal": 0},
  "existing_key_sources": {
    "COHERE_API_KEY": "/operator/home/.bash_profile",
    "LITER_LLM_MASTER_KEY": "/operator/home/.prometheus/kbd/secrets.env"
  },
  "source_contracts": {
    "claims": {"path": "/repo/src/uar/security/claims.rs", "sha256": "sha256:EXACT"},
    "verifier": {"path": "/repo/src/uar/security/verifier/mod.rs", "sha256": "sha256:EXACT"},
    "presentation_owner": {"path": "/repo/src/uar/runtime/actor/messages.rs", "sha256": "sha256:EXACT"},
    "user_settings": {"path": "/repo/src/uar/api/user_settings.rs", "sha256": "sha256:EXACT"},
    "collaboration_scope": {"path": "/repo/src/uar/api/collaboration.rs", "sha256": "sha256:EXACT"},
    "session_storage": {"path": "/repo/src/uar/persistence/mod.rs", "sha256": "sha256:EXACT"}
  },
  "preparation_approval": {"path": "/private/approved-preparation.json", "sha256": "sha256:EXACT"},
  "pending_operator_inputs": ["approved-full-chain-charge-ceilings", "budget-prior-consumption-and-unknowns", "final-signature-and-adoption"]
}
```

This schematic is deliberately nonexecutable: zero timestamps/ports and placeholder
digests must be replaced with reviewed exact inputs. `epoch_id` is1..128 characters
from letters/digits/._:-. Validity must contain current time, with positive duration
at most86400 seconds; there is no automatic token renewal. The packet must finish
inside its approved JWT validity. Every port is explicit integer1024..65535, all
values distinct. Include all HTTP/gRPC/Surreal/budget/metadata listener ports in
ports, plus the required owned Postgres and Surreal endpoints. No free-port
probe, default port, service discovery or loopback request occurs.

Each source contract must match the actual corresponding path in this checkout
and exact SHA. References are frozen inputs, not runtime checks executed now.
Preparation validates those adopted bytes only when explicitly invoked later.

The private preparation approval must equal:

```
{
  "operation": "prepare-uar-acceptance-private-inputs",
  "epoch_id": "EXACT_APPROVED_EPOCH",
  "scope_root": "/existing/private/parent/uar_fixture_inputs_EXACT_EPOCH_HASH20",
  "preparation_projection_sha256": "sha256:EXACT",
  "effects": [
    "create-exclusive-private-scope-and-state-parents",
    "generate-owned-hs256-jwt-and-admin-secrets",
    "mint-two-owned-fixture-user-tokens",
    "generate-shared-owned-ingress-raw-key",
    "generate-owned-postgres-and-surreal-credential-inputs",
    "privately-copy-two-selected-existing-provider-keys"
  ],
  "operator_approved": true
}
```

Projection SHA is compact sorted UTF8 JSON of the entire preparation config
except preparation_approval, with allow_nan=false. This avoids a self-reference.
The tool verifies exact approval bytes/scope but does not cryptographically verify
an operator signature. The parent must bind this command, source, config and
approval to the future explicitly authorized packet; writing a JSON boolean does
not grant approval now.

## Existing key retrieval

The current `selected_embedding_services=["cohere"]` considers only
COHERE_API_KEY and LITER_LLM_MASTER_KEY. The unused separate OpenAI allowance
does not authorize or require copying another key. Only explicit selection of
`["cohere", "openai_embeddings"]` additionally requires `OPENAI_API_KEY` from
the same `.bash_profile` literal-source contract. Then `existing_key_sources`
must contain exactly those three names and the exact approved effects append
`privately-copy-explicitly-selected-openai-embedding-key`. The selection is part
of the preparation projection and emitted inputs/receipt; changing it requires
matching preparation authority. Neither selection invents an OpenAI model.
A present nonempty
process environment value wins. Otherwise the program selects exactly one literal
assignment for that exact name in the corresponding file above. The selected
paths must equal the invoking operator's actual home paths. No other secret file
or variable is loaded into the resulting environment map.

Supported source syntax is optional export plus NAME=literal, with a single-quoted
literal, a double-quoted literal containing no expansion/backslash/backtick, or a
restricted plain token. A trailing comment is permitted. Multiple assignments,
missing key, command substitution, expansions and unsupported escaping block.
No source/eval or shell parser execution occurs. Profiles are not edited. The
recipe does not resolve KIMI_CODING_KEY or MINIMAX_KEY; the existing Liter service
already owns those credential dependencies.

## Actual auth and namespace contract

Fresh48-byte URL-safe JWT/admin/ingress secrets and36-byte datastore passwords
are produced with secrets.token_urlsafe. HS256 is HMAC-SHA256 over base64url
header/payload, without padding. Header is alg HS256, typ JWT. Claims are the
actual UserClaims fields sub/name(null)/roles([user])/tenant_id/exp, plus verified
standard iss/aud/iat/nbf. No issuer-credential marker, API-key marker, instance
identity, host grant or artificial admin role is minted. The separate admin key
serves only the configured settings header. Subject/tenant/issuer/audience and
workspace are fresh fixture-only ASCII identifiers derived from epoch suffix.
Both principals share a tenant/workspace but have different subjects, which keeps
cross-owner cases discriminating within the same tenant.

The generated security_patch sets remote deployment, required HS256 JWT,
issuer/audience/nbf validation, explicit two workspace_authorities and protected
settings admin-key refs. Composer merges it into its complete approved security
configuration; it is not a replacement for unrelated credential/role policies.

Names are intentionally distinct:

- credential_principal_key/session_owner_key and
  orchestrator.expected_principal_key use CredentialPrincipal::storage_key:
  `v1:k:issuer:i:<issuer UTF8 length>:<issuer>:t:<tenant length>:<tenant>:s:<subject length>:<subject>`.
- history_owner_key/collaboration_owner_key and
  orchestrator.expected_history_owner_key use ActorOwner::presentation_owner_key
  and user_settings::principal_storage_key:
  `v1:t:<tenant length>:<tenant>:s:<subject length>:<subject>`.
- workspace_id is exactly the x-uar-workspace-id header value, authorized by the
  generated mapping. No invented combined owner/workspace key is emitted.
- Actual session record keys are generated by the runtime source helper from
  full session_owner_key plus session ID; preparation does not predict runtime
  session IDs or present source derivations as observed authentication receipts.

## Composer environment contract

Generated refs accepted by the composer:

- UAR_FIXTURE_JWT_SECRET, UAR_FIXTURE_ADMIN_KEY.
- UAR_FIXTURE_PRIMARY_AUTHORIZATION, UAR_FIXTURE_FOREIGN_AUTHORIZATION: full
  `Bearer <JWT>` headers for actual middleware.
- UAR_FIXTURE_INGRESS_RAW_KEY: shared raw owned gateway-ingress token. Model and
  embedding api_key fields receive the raw value; budget comparison must use
  ingress_authorization_prefix="Bearer ". Never reuse a full header as api_key.
- UAR_FIXTURE_PG_USERNAME/PASSWORD/DATABASE/CONNECTION: fresh owned role/database
  names and percent-encoded postgresql loopback URL at explicit postgres port.
- UAR_FIXTURE_SURREAL_USERNAME/PASSWORD/URL: fresh owned username/password and
  explicit loopback HTTP origin. These are never operator root credentials.
- UAR_FIXTURE_POSTGRES_STATE_PARENT, UAR_FIXTURE_SURREAL_STATE_PARENT,
  UAR_FIXTURE_PACKET_STATE_PARENT: the three created children.
- UAR_FIXTURE_POSTGRES_RUNTIME_CONFIG/STARTUP_CONFIG and corresponding SURREAL
  names: exact future composer filenames postgres_pgvector-runtime/startup.json
  and surreal_disk-runtime/startup.json beneath composition_output_dir.
- Existing COHERE_API_KEY and LITER_LLM_MASTER_KEY remain those exact named refs.
  Upstream budget Authorization header_prefixes explicitly supplies "Bearer ".

Missing pricing, final signatures, source/build closure, datastore provisioning,
service identity, resource availability and actual auth/owner/vector receipts
remain unresolved. Generated inputs do not certify any of them.
