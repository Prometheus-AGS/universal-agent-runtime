# Bounded representation execution 1.0.0

Initiative: C17, `afc-c17-executive-assistants-and-consented-human-representation`.
Capability: `collaboration_representation_execution_v1`. This is implemented
source coverage, not person-specific evaluation or an operational qualification.

The existing private RepresentationGrant remains the authority record. Office
names, executive personas and reusable AgentDefinition templates grant no powers.
Ordinary bound-agent and team-member execution consume the same current grant
references. No new scheduler, policy engine or agent loop is introduced.

## Issuance and current authority

REST and MCP installation require the middleware-established trusted-host marker.
`issuerPrincipalId` must equal the authenticated subject. The owner partition
remains the separately encoded tenant/subject storage key. The trusted
host must establish the human consent and organizational authority referenced by
the two protected evidence references before installing a grant. UAR stores those
references as owner-asserted provenance; it does not independently certify
external organizational evidence.

Binding admission already binds legacy service/team grantees, grant revision and
constraint digest. For a durable logical instance, trusted REST issuance attaches
the exact grant reference to a private, revisioned instance authority record;
it does not rewrite the immutable definition or deployment binding. The instance
view exposes only the authority revision and exact grant references. Existing
records deserialize with an empty authority set and revision zero.

The existing instance turn endpoint consumes the attached authority. Admission
checks the durable grantee identity and authenticated issuer. Execution additionally rejects inactive, revoked, suspended,
expired or not-yet-valid grants, including a non-null revocation record on an
otherwise active grant. Current authority is reloaded at run admission and before
each tool preparation and claim. A new grant revision or offboarding revocation
invalidates the existing pinned binding or instance authority for new access/effects. REST/MCP installation
also cancels affected in-flight runs through the existing cancellation boundary. Previously
dispatched effects are not undone by revocation. Grant references remain attached
after revocation or expiry, so a denied represented turn cannot fall back to an
ordinary unrestricted turn. REST attachment failure reports the committed grant
and leaves its existing command identity available for reconciliation. MCP
issuance remains catalog-only; it does not certify durable-instance attachment.

## Supported execution vocabulary

Scope identifiers match exact current identifiers. There are no wildcards.
Multiple referenced grants intersect restrictions; they do not union powers.

| Field | Implemented interpretation |
| --- | --- |
| `actionScopes` | `tool:PROVIDER_TOOL_NAME`, selecting actual eligible tools from local policy. Unknown or unavailable names deny admission. |
| `resourceScopes` | Exactly one `workspace:WORKSPACE_ID` matching the authenticated binding workspace. Existing host sandbox and tool policy remain mandatory. |
| `dataScopes` | `knowledge-base:KB_ID`, intersecting eligible local knowledge bases. Empty means no KB context. Explicit host-supplied task input remains input; conversation memory and preloaded memory hits are disabled. |
| `audienceScopes` | Must include `user:AUTHENTICATED_OWNER`; may also include exact `team-member:MEMBER_ID` recipients. |
| `approvalRequirements` | `current-policy` and optional `real-human`. Cedar is mandatory even if the array is empty. `real-human` forces the existing approval pause and requires the trusted Approved disposition at claim. |
| `disclosureRequirements` | Exactly `disclose-agent-assistance`. The host inserts a grant/revision disclosure into the prompt, run context and output event stream. |
| `offboarding` | `revoke-immediately`, with optional `disable-binding` obligation. Revocation invalidates pinned binding execution; UAR does not erase stored history or externally issued effects. |
| `retention` | `retain-audit` with `deleteAfter: null`. Automatic deletion promises are unsupported and denied. |
| `restrictions.forbiddenClaims` | `human-authorship` and `human-approval`. Host disclosure explicitly denies both; model text cannot satisfy a trusted human approval. |

Tools must have a trusted ReadOnly effect declaration, except the existing
attempt-owned `team_send` and `team_wait` controls. `team_send` must name an
authorized audience member and include the exact host disclosure in `payload.text`
before admission; UAR does not mutate an approved payload afterwards. Existing
team membership, task ownership, current-policy and mailbox checks still apply.
Opaque MCP effect classifications remain Unknown and are rejected.

Execution starts with an empty session and cannot import checkpoint or supplied
historical conversation state. A fresh durable instance with no stored history
uses that existing empty-session path; real retained history still denies admission.
Read-only file operation uses an actual registered `file_read` tool, explicitly
enabled and restricted to an operator-selected disposable root by native-tool
settings. At startup, registration consumes persisted native-tool preferences
after settings bootstrap, using configured defaults only for absent preferences.
Changes require restarting UAR; saving alone does not hot-reconfigure a live
registry. The representation grant only narrows eligible tools; it never enables
a tool or widens the filesystem root allowlist.
Existing retained grant strings such as
`artifact.read`, `artifact:assigned`, `artifact.content` and organization-wide
audiences remain readable authority records but return named
`REPRESENTATION_*_UNSUPPORTED` or scope-denial diagnostics during execution.
They are never translated into unrestricted tools or data access.

## Unsupported outcomes and remaining work

Provider mutations, code execution, financial actions, communication through
arbitrary provider tools, subdelegation, history/checkpoint import, deletion or
retention automation, and unspecified offboarding obligations are denied by this
profile. They require additional typed action/resource/audience contracts and
trusted provider boundaries. A human approval cannot widen this profile.

Executive office template catalogs, the host consent/evidence issuance workflow
and audit UI, organizational evidence certification, person-specific held-out
evaluation and the combined real-account operational gate remain separate work.
The broad C17 initiative is not certified by this bounded source delivery.
