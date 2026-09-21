# Local runtime harness profile certification

This registered child milestone verifies the parent runtime-harness-gap-closure delivery locally at Tier 3. Its binding scope is ../../profile-certification.md, with thresholds and fixtures in ../../acceptance-contract.json. This goals file carries that existing scope forward; it does not add product behavior, deployment or release work.

Enter only after the parent's implementation units and Tier 2 candidate-readiness checks pass. Complete this milestone's own workflow and reviewed environment/command plan before Tier 3 execution. Freeze every required stable profile/OS/live/device row and its applicability evidence. Retain both Surreal and Postgres restart/crash receipts; in-memory tests do not prove durability. Product verification never runs in GitHub Actions.

Publish the required-row inventory and final evidence receipt at the paths specified in the parent certification scope. Missing required evidence is blocked. Parent task harness-governed-evaluation/3.1 checks this milestone's canonical completion plus the receipt and inventory hashes; /3.2 assembles the F1–F8 evidence matrix. Parent final phase completion depends on this milestone, but milestone entry does not depend on the parent already being complete.

The uncomfortable constraint: one development machine may not supply all stable platforms or live provider/device environments. Missing rows cannot be relabeled passed or inapplicable to close the phase.
