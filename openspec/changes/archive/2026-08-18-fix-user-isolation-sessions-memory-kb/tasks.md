## 1. Threads
- [x] 1.1 Add user ownership to session/thread model and all persistence providers; scope list/get/delete, direct and ACP active-run, configuration, and conversation-policy operations.
- [x] 1.2 Preserve legacy sessions in anonymous scope without allowing authenticated ID-based claims.
## 2. Memory
- [x] 2.1 Replace client-supplied user_id in /api/memory with UserContext identity.
## 3. Knowledge
- [x] 3.1 Add owner scoping and tenant-qualified durable identities to KBs/documents/chunks across providers and API handlers.
- [x] 3.2 Remove the all-KB retrieval fallback in the run manager; fail closed.
## 4. Proof
- [x] 4.1 Cross-user bleed integration tests (threads, runs, policy/configuration, memory, KB, documents) under two JWT identities.
- [x] 4.2 Durable PostgreSQL and SurrealKV tests prove identical logical IDs coexist; the PostgreSQL test also proves cross-owner parent links fail closed.
- [x] 4.3 Document shared-admin resources decision (O4) and legacy-session ownership in the compatibility policy.
