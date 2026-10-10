# Design

A connector binding names one provider, one exact target, allowed actions, trusted egress labels and an opaque host credential reference. The authenticated owner/workspace scopes the binding. A request can only narrow those capabilities. External content is data and cannot amend the binding.

UAR commits an intent with command digest before the host acquires a dispatch lease. The trusted host asks for the typed request plan, resolves the credential reference from protected storage and performs the request once. It records the returned external identity or an explicit uncertain disposition. A lost response is uncertain and never retried automatically. Reconciliation requires the host to present independently checked external evidence or leave the effect unresolved. A draft is local and has no dispatch lease. Read operations are also recorded; write, send and publish require a decision reference.

The host-only dispatch surface is unavailable until the sidecar's trusted host authentication and credential broker are wired. Ordinary API callers cannot supply tokens or claim a successful external effect. No connector action directly authorizes implementation or a roadmap promise.
