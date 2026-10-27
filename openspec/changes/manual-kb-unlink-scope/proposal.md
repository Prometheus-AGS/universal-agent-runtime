# Preserve explicit agent KB unlinking

The manual KB-UNLINK-01 scenario exposed that removing the last KB link inherits automatic access, retrieving the unlinked and other available KBs. When a partial agent update explicitly changes KB links or enabled state, persist the corresponding explicit selected/none policy. Preserve unrelated policy fields and denied IDs; an explicit KB policy in the same patch takes precedence.

Legacy definitions, including the built-in default, retain their existing automatic behavior until deliberately edited. Scope: shared agent patch persistence and one manual Mac BDD; no old scenario reruns or broad verification.
