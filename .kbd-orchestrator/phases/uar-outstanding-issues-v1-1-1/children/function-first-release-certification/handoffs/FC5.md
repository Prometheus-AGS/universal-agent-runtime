# FC5 handoff — exact implementation inventory

Date: 2026-10-17. Execute task `3.2`. Operator verification lock: **CLOSED**.

The parent ledger has 24 completed implementation rows. Child FC1–FC4 are
canonically complete with source-only handoffs. The new parent inventory lists
those 28 rows plus this FC5 row. The operator explicitly instructed the lead
to run `python3 /tmp/uar111-fc5-freeze-20261017.py`; it exited 0 and recorded
full HEAD `38f0b5d85960e3cb8445607750538ca500efdf17`, source tree
`sha256:8486a2d4edee965efcdd68697f4b5ba10447a744b96e400bb75ee606cabf5347`,
fixture tree
`sha256:387278df16cc41aab1d1e4ac588f589f556a607cd40d628ecde35690c428105d`,
and parent inventory digest
`sha256:0bcc3c31a7ae1157b68b16dec21d24e49ca4205d5f013267242163cd7543364c`.
The command read saved source/fixture bytes and wrote the inventory and local
template identity fields; it did not test behavior or start services.
The parent inventory is `implementation-inventory-function-first.json`; the
child view is `implementation-inventory.json`. The local and both published
manifest files are deliberately non-executable templates. The local selection
is `uar:1.1.1-function-first`, linux/arm64, six groups and four native source
supplements. Published selections are `streams,recovery` only. Exact image IDs,
qualified supervisor evidence and published digests do not exist for this
candidate yet.

The complete FC1–FC4 file ownership and interface effects are in their named
handoffs. FC4's runbook gives source-backed same-image guest preparation but
requires actual FC6 image/private values before commands can be rendered.
The operator packet names the sole proposed FC5 identity command and the later
build, review and functional gates. No behavior, performance or release
result has been observed for these changed bytes.

Other commands used for this handoff were source/ledger inspection and
authorized documentation writes. No behavior test, import/compiler probe,
Docker operation, build, review-driven verification or acceptance command
ran. Image and qualified supervisor preparation still need separate exact
operator instructions. If source or fixture code changes, renew identity
again. No unrequested product feature or guard was added in FC5.
