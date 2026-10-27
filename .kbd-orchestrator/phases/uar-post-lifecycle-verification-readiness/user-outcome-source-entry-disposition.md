# Source inventory entry disposition

Verification CLOSED. Documentation disposition of snapshot001; no build or acceptance claim.

The source snapshot lists regular working files separately from symlinks and tracked deletions. Its source digest covers the listed regular files only; this must not be described as every filesystem entry or the installed dependency closure.

The189 symlinks are predominantly source catalog aliases and governance aliases. The inventory records each link target, resolved path and target-byte digest when readable. An alias is not an independently installed dependency. A future build-input closure must bind any alias actually consumed by that build to its resolved regular bytes. Missing unrelated aliases are not runtime failures, and this record supplies no evidence that an unbuilt customer artifact resolves them.

The six missing tracked paths are nested Parking Lot LICENSE-MIT files removed by the exact0.4.0 upstream production export. The exported root MIT license and crate license declarations remain. These are deliberate export deletions, not missing Rust modules. Their deletion records remain in the inventory/history; no unrelated license restoration is authorized here.

Installed A2UI0.10.4 and local entity4.2.0 differ from declared locked A2UI0.12.0 and umbrella/core entity3.0.3. Production source is reconciled against official locked source under task2.11. An old installed tree cannot establish locked-version behavior. Future dependency preparation must be an explicit approved operation, followed by a fresh installed closure and source-bound build receipt. None of that has run.

Snapshot001 is immutable historical source evidence, not executable fixture authority. Any source correction requires a newly numbered snapshot and renewed exact commands. All14 historical obligations and actual failures remain pending current-source acceptance.
