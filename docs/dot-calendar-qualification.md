# Installed calendar qualification

`scripts/dot/calendar_journey.py` is authored qualification code, not executed
evidence. The full-phase acceptance entrypoint calls
`run_calendar(config, evidence)` only after the lead opens the complete production
implementation boundary. This module has no standalone command or bypass.

Run this stage inside a dedicated, disposable Linux virtual machine. It changes
the guest's actual `CLOCK_REALTIME`; it must never run on a developer machine,
shared VM, physical host, or container. The driver requires an exact machine ID
and boot ID, detects hardware virtualization, rejects container execution, checks
that systemd reports NTP disabled, and binds the isolation and hypervisor clock
sync prerequisites to supplied file digests. Those files document provisioning
facts; they are not accepted scenario results. Hypervisor time synchronization
and any clock writer outside systemd must already be disabled. The driver checks
for unexpected wall-clock changes while observing schedules. It does not alter
NTP, hypervisor settings, the RTC, or the host timezone.

The guest runner needs permission to set its system clock. Runtime credentials,
deployment grants, signing keys, and any TLS certificates must remain valid over
the complete fixed fixture interval, March 13 through December 11, 2027. Use an
isolated test issuer and storage namespace. Provision the deployment binding
through the real compiler/catalog API. Its model connection must point to the
driver's controlled HTTP provider, and its Required effect tool must point to
the checked-in `tests/fixtures/dot17_effect_mcp.py` stdio peer with the configured
absolute effect trace. The driver does not mint grants or write protected rows.

The `config` object is a dedicated profile, not the active ordinary-profile
process objects. It has these fields:

| Field | Meaning |
| --- | --- |
| `hosts` | One local-KV or two shared-Surreal `Host` specifications from the acceptance runner. |
| `deployment_binding_id` | Admitted deployment with the actual controlled provider and Required effect tool. |
| `provider_port` | Guest loopback port used by the controlled provider. |
| `effect_trace` | Absolute path independently written by the effect peer. |
| `durability_evidence` | Shared profile's actual durable-storage prerequisite file. |
| `database_provisioning_evidence` | Shared profile's `{ "path": "absolute receipt", "sha256": "digest" }` proving actual durable SurrealDB resides in this guest clock domain. |
| `guest_clock.allow_guest_clock_changes` | Explicit `true` for this disposable guest. |
| `guest_clock.machine_id` | Exact `/etc/machine-id` from guest provisioning. |
| `guest_clock.boot_id` | Exact `/proc/sys/kernel/random/boot_id` for this run. |
| `guest_clock.isolated_guest_evidence` | `{ "path": "absolute provisioning receipt", "sha256": "receipt digest" }`. |
| `guest_clock.hypervisor_clock_sync_disabled_evidence` | Same shape, recording the actual disabled hypervisor synchronization setting. |

Each `Host` specification uses the existing runner's `base_url`, `workspace`,
`credentials` environment-variable mapping, `lifecycle: "child_process"`,
`binary`, `binary_sha256`, `config`, `environment`, `inherit_environment`, and
`working_directory`. Both shared hosts run in this same guest so their real
wall clock agrees. Their persistence configuration must match exactly. The
shared durable database must also run inside this guest and resolve to a loopback
endpoint because durable host invariants use database `time::now()`. A database
in another clock domain does not qualify. The driver checks loopback resolution
and binds its provisioning receipt by digest; it never changes database rows or
starts an arbitrary supplied database command. No ordinary-profile runtime or provider may still own these ports when
this separate stage begins.

The driver performs these fixed operations through actual HTTP endpoints:

- The four cases in `tests/fixtures/dot14_calendar_cases.json`: Chicago's spring
  gap produces skipped history and zero model calls; the autumn fold uses only
  the earlier UTC instant; summer and winter preserve 09:00 local time.
- A one-shot Required operation retains its admitted occurrence through pause,
  resume and schedule cancellation, produces no effect before approval, then
  commits exactly one real effect and a terminal schedule receipt.
- A daily operation commits a real effect whose peer exits before replying.
  Three subsequent due days retain the predecessor guard and coalesce December
  4/5 into December 6 pending intent. Revision, pause, cancel and joined runtime
  restart cannot replay the unknown operation or produce a terminal receipt.
- A paused daily schedule catches up December 8–11 without dispatch, records
  three skipped candidates, then executes only December 11 after resume and
  fresh Required approval.

Every negative observation spans at least eight monotonic seconds, covering the
production observer's two-second wakeups. Model requests, external effect counts,
schedule status, UTC instants, clock changes, installed Chicago timezone digest,
runtime logs and HTTP response digests enter the runner's evidence directory.
The driver stops its owned runtimes and provider and restores the initial wall
clock plus elapsed monotonic time in cleanup. It does not resolve unknown effects
or release their accounting holds merely to clean up a fixture.

The uncomfortable thing: a process killed before Python cleanup can leave the
guest clock advanced. Dispose of or independently restore this dedicated VM;
never treat the cleanup path as host isolation. A joined child-process restart
also does not prove whole-boot or Pod exclusion. That evidence remains in the
separate concrete supervisor stages. None of these scenarios has run yet.

Clock operations follow Python's
[CLOCK_REALTIME and monotonic documentation](https://docs.python.org/3/library/time.html).
Guest detection follows the upstream
[systemd-detect-virt contract](https://github.com/systemd/systemd/blob/main/man/systemd-detect-virt.xml),
and the NTP prerequisite uses
[timedatectl's machine-readable interface](https://github.com/systemd/systemd/blob/main/man/timedatectl.xml)
and its documented
[NTP property](https://github.com/systemd/systemd/blob/main/man/org.freedesktop.timedate1.xml).
