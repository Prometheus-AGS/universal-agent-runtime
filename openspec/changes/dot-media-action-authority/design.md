## Decisions

- Match exact action/modality/capability tuples and exact session/target/purpose. The stored confirmation owns the original scope, while a delegation narrows permissions and expiry.
- Derive owner, tenant and host scope from authenticated request context. A trusted host attests explicit user confirmation and authenticated coordinator or worker identity. No observation handler or kernel may issue approvals or dispatch.
- Use the collaboration catalog CAS boundary to persist grants and intent, and to atomically claim one attempt and append its unknown receipt. Host I/O runs after the commit. Reconciliation appends an evidence-linked outcome to that attempt.
- Keep supplied content transient. Bind send content to its SHA-256 digest. A host resolves speech and video credentials in separate slots outside UAR.
- Implement the mock peer as a separate fixture process so UAR can restart while mock-side evidence survives. Production API requests use an externally provisioned identity endpoint.

## Risk

A host can disappear after dispatch but before observing the result. The stable effect ID and attempt prevent a second lease. Independent evidence is required to settle the outstanding attempt. The trusted host must never reuse a stale claim after reconciliation.
