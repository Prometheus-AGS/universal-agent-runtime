# Selected qualification contract — draft grounded in existing decisions

Tracker #476; continuous claim owner #466; final qualification owner #465. This is the work selection contract, not a certification statement.

Primary deployment: standalone native service / supervised sidecar, initially the existing macOS native execution lane; architecture is recorded per actual artifact, never inferred from a macOS label; Linux amd64/arm64 container lanes require their own installed-artifact evidence before promotion. No production deployment or signing is authorized. Default persistence is pinned SurrealDB 3.3.0; PostgreSQL 18/pgvector is a restricted secondary lane using docs/operations/persistence-capabilities.md and #459's retained receipts. Kubernetes/Helm PG17 does not inherit Compose PG18 acceptance. Memory remains an independent service. Native Windows, Intel Mac, mobile, browser WASM and Rust in-process embedding stay unqualified unless explicitly selected later.

Public surfaces in scope for investigation/qualification: normal text/tool execution over native REST, OpenAI-compatible chat, Anthropic Messages, AG-UI/A2UI and documented A2A/MCP roles. Preserve current identity/owner validation, authorization, approvals, destination budget (#388), replay and uncertain effects. Supported protocol revisions/capabilities require independent pinned-peer evidence under #465; metadata alone grants no support.

#463/#464 are the next shared dependency slice: define a bounded image representation/public path and carry hard capability/policy requirements into actual dispatch and fallback. Do not infer vision from a catalog flag; unimplemented paths must refuse explicitly. Preserve final-destination budget compilation. Remote URL fetching, audio/video and adaptive learned routing are excluded.

Recovery promises remain interface-specific: durable sessions/history do not imply full-harness durability. Full harness is process_ephemeral, unsupported_after_restart, steer_supported:false. No automatic replay of uncertain external effects; preserve pending/reserved-unknown liability until authoritative disposition. #466 owns one guarantee table (admission/checkpoint/replay/effects/in-doubt/upgrade); #465 qualifies its promoted rows. No global exactly-once or full PostgreSQL parity claim.

No arbitrary SLO, comparative-quality superiority, every-provider certification, new kernel, global harness or full-harness rewrite. #472 needs separate workload/threshold selection; real-model spend remains an authorization boundary. Synthetic local fixtures are appropriate wiring evidence, never real-model quality evidence.

Pending lane-specific qualification is explicitly unverified. Source-corrected claims do not promote capabilities. Existing receipts transfer only with unchanged relevant source/config/artifact and explicit justification.
