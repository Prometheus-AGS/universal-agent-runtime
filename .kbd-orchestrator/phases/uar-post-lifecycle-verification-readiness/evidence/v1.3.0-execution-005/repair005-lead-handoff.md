# Repair005 lead source handoff

Verification CLOSED; source authoring only. tests/support/post_lifecycle_store.rs history cleanup now requires explicitly declared historyOwnerKey matching ActorOwner::presentation_owner_key (tenant/subject catalog namespace). Accounting/CAS ownerStorageKey remains the distinct issuer-scoped principal key. Exact record/run/event ownership and snapshot fences remain; no inferred owner or cleanup waiver. Existing main JWTsubject/tenant determine the new explicit manifest key.

src/uar/runtime/manager.rs early model binding previously computed budget_admission/category then emitted generic orchestrator_start_failed for every non-team error. It now emits that existing public budget category only for the budget_admission stage; team and other failure behavior unchanged, diagnostic reference still redacted. This repairs a source-proven closed-class projection defect under the original root-budget requirement. The exact original685-byte root response was not retained, so do not call its complete original contents/cause recovered. ERROR01 remains a required real-path oracle for the next approved invocation.

No arbitrary raw body capture. Driver authoring retains assertion observed/expected counts and actual CAS report-derived controls. All changes uncompiled/unexercised; no independent acceptance review yet.
