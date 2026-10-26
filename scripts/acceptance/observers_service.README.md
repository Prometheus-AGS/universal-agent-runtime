# Configured Liter service identity observer

Source authored only. The phase verification gate is CLOSED. This script has not
been imported or run. Only local help/manual and selected source files were read.
No launchctl status, process/listener probe, service operation, HTTP or model call
was made during authoring.

The later named packet invokes this once, before the budget daemon and positive
calls: `[frozen_python, absolute_observers_service_py, private_config_json]`.
Root owns serial invocation, bounded execution, captured receipt and cleanup. The
script has no I/O on import and never starts, stops or changes the existing
service. It requires no secret environment references and never reads the secrets
file. Credential names are whitelisted metadata only. A subprocess timeout kills
only its own observation-command child, never the observed service.

The uncomfortable limit: on-disk hashes/mtime, a registered launch job, matching
process command and listener do not prove loaded in-memory configuration, a build
from current source, the identity of downstream model implementations, embedding
availability, provider tariff, or actual cost. The operator supplies the trusted
service provenance. `launchctl print` explicitly documents that its human format
is NOT an API. This observer accepts a narrow frozen format and refuses unknown
or ambiguous output; such a refusal requires investigation, never a green fallback.

## Frozen private configuration

Config is a regular absolute nonsymlink mode600 JSON file, at most1MiB. Fields:

- `schema_version:1`, `executable:true`, `uid:<operator's actual uid>`.
- `tools`: `launchctl`, `ps`, `lsof`, each `{path,sha256}` for exact `/bin/launchctl`,
  `/bin/ps`, `/usr/sbin/lsof`. SHA strings use `sha256:<64hex>`.
- `sources`: `launchagent`, `launch_script`, `service_config`, `executable`, each
  the same reference shape. Hash existing bytes; never rebuild as a preflight.
  Sources must be regular nonsymlink files; executable max1GiB, other sources
  max1MiB. Existing binary symlinks must be resolved and operator-approved as the
  actual executable path, rather than silently followed. No write occurs there.
- `expected_argv`: exactly `[sources.executable.path,"api","--config",
  sources.service_config.path]`. This narrow macOS ps parser requires arguments
  without whitespace. CLI master keys, watch flags and alternate configs refuse.
- `approved_listener_names`: explicit nonempty subset of `127.0.0.1:4000`,
  `[::1]:4000`, `*:4000`. Wildcard is reachable on loopback but is reported as
  `loopback_exclusive:false`. Inspected CLI defaults host0.0.0.0 and port4000;
  the selected launcher supplies no host override. No bind is changed here.
- `expected_models`: ordered exact selected config projection
  `[{alias,provider_model,upstream_origin}]`. `upstream_origin` is scheme/authority
  only. Expected current aliases are gpt-5.4, k3, MiniMax-M3; use inspected source
  and adopted input, never invent a usable embedding model from a wildcard.
- `expected_alias_patterns`: ordered exact list, currently `["openai/*"]`.
- `expected_max_retries:3`, `expected_cost_tracking:false` for selected config.
- `credential_env_names`: exactly `["LITER_LLM_MASTER_KEY","KIMI_CODING_KEY",
  "MINIMAX_KEY"]`. Values are not required/read/printed by this observer.
- `limits`: positive integers `command_seconds<=30`, `wall_seconds<=120`,
  `output_bytes<=1048576`. Combined stdout/stderr per command is bounded and
  discarded except whitelisted parsed fields. Parent separately bounds the
  entire invocation, including source hashing and file parsing.
- `receipt_path`: exact new absolute filename beneath existing private mode700
  parent outside backend cleanup. Exclusive mode600 creation, flush+fsync. Reuse
  is refused; this is one observation per packet, not a retry loop.

The installed help/man source establishes the argv below. They are to be run
only inside the subsequently approved observer command, not separately now:

1. `/bin/launchctl print gui/<uid>/ai.prometheus.liter-llm-api` parses only exact
   top-level target/path/state/pid. Registered path must equal adopted plist.
2. `/bin/ps -p <pid> -o uid=,lstart=,command= -ww` reads only that PID. UID must
   equal observer owner, command must exactly match adopted argv. LC_ALL=C and
   TZ=UTC make the documented lstart format parseable. Environment is not read.
3. `/usr/sbin/lsof -n -P -a -p <pid> -u <uid> -iTCP:4000 -sTCP:LISTEN -F pufnt`
   selects only that user's exact PID and TCP4000 LISTEN sockets; no DNS/port-name
   lookup. Field parser accepts only requested PID/UID/fd/name/type fields. Every
   observed listener must match explicit approved names.
4. The same exact-PID ps command again confirms unchanged owner/start/command.
   This is a fixed bounded identity interval, not a verification retry.

Sources are rehashed after observation. Their mtime must be no later than the
process start plus one second (ps precision); a later modification refuses the
claim. Older timestamps are evidence of ordering, not proof of loaded contents.
No raw launchctl, ps or lsof output, environment, prompts or secrets is emitted.
Local support was source-inspected with `launchctl help print`, `lsof -h`,
`lsof -F?`, and installed `/usr/share/man/man1/{launchctl,ps}.1` and
`/usr/share/man/man8/lsof.8`. Live output shape remains unverified.

## Success and failure receipts

Success writes one identical JSON object to stdout and receipt_path:

- `schema_version:"uar.configured-service-identity.v1"`, `status:"observed"`,
  `service_id:"ai.prometheus.liter-llm-api"`.
- `config_sha256` is observer config bytes; `sourceconfig_sha256` is selected
  Liter TOML bytes. Parent must match frozen adopted digests.
- `uid`, `service_target`, actual `pid`, `observed_at_unix`,
  `process_started_unix`, `expected_command_matched:true`, `process_argv`,
  `registered_launchagent_path`.
- `sources` contains adopted refs plus mtime, no contents; `listeners` contains
  actual approved socket names and `loopback_exclusive` reports wildcard honestly.
- `configured_models`, `configured_alias_patterns`, `credential_env_names`,
  `gateway_max_retries`, `gateway_cost_tracking`, and explicit `limits` disclaimer
  strings. These are current disk settings, not proved live loaded model support.

Failure exits2 with `{error:"configured_service_identity_unconfirmed",code:...}`;
only fixed refusal codes or exception class names are emitted. No receipt means
no successful observation. Root must not start positive calls after failure.

The selected service retries up to its configured retry policy and has cost
tracking disabled. Budget forwarding counts UAR→gateway requests, not physical
upstream-provider attempts. Each approved per-request charge ceiling must cover
that service's retries and fallback chain; unresolved maximum exposure blocks
charged execution. This observer never disables, reduces or counts those internal
attempts. Actual embedding model identities remain unresolved by selected TOML.
