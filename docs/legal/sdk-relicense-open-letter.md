<!-- DRAFT — operator review required before send (task 1.3 in
     openspec/changes/license-dual-license-agpl-mit/tasks.md). Do not send
     until the operator has reviewed and approved this text. -->

# Open letter: relicensing the UAR SDKs from AGPL-3.0-only to MIT

## Contributor audit (task 1.2)

`git log --all -- sdks/python sdks/rust sdks/typescript` shows all history
under `sdks/` was authored by:

- Travis James (`tjames@prometheusags.ai`, `travis@tribehealthsolutions.com`)
- `Ubuntu <azureuser@prometheus-db.*.cloudapp.net>` — a CI/build automation
  identity, not a human external contributor

**There are no third-party external contributors to `sdks/` to date.** The
audit in `plan.md` ("~10 known contributors") appears to have been an
estimate, not a verified count — the actual git history shows only the
operator's own identities plus a CI account.

**Consequence:** if this holds, the open-letter step (1.3) can likely be
satisfied by operator self-authorization rather than outreach to third
parties, and task 1.4 (removal of non-responsive contributions) does not
apply. The operator should confirm this reading (e.g. check for
non-git-tracked contributions, squashed history, or contributions not yet
merged) before treating consent as resolved.

## Template (for use only if third-party contributors are found)

---

Subject: Relicensing the UAR SDKs from AGPL-3.0-only to MIT

Hi {{contributor_name}},

You've previously contributed to the Universal Agent Runtime (UAR) SDK
packages (`sdks/python`, `sdks/rust`, and/or `sdks/typescript`). We're
relicensing those SDK packages from AGPL-3.0-only to MIT, to match the
license terms used by the SDKs of every comparable agent runtime project
and make it easier for developers to adopt UAR client libraries in
proprietary and commercial projects.

The UAR **runtime server** (everything outside `sdks/`) keeps its current
AGPL-3.0-only + commercial dual-license; this relicensing is scoped to the
SDK packages only.

We'd like your consent to relicense your past contribution(s) to the SDK
packages under the MIT License. If we don't hear back within 30 days of
this letter, or if you'd prefer we not relicense your contribution, we will
remove the affected code from the SDK packages rather than relicense it
without consent.

Please reply to confirm consent, or let us know if you'd like your
contribution removed instead.

Thanks,
The Prometheus AGS / UAR maintainers

---
