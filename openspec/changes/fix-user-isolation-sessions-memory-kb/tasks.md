## 1. Threads
- [ ] 1.1 Add user ownership to session/thread model and all persistence providers; scope list/get/delete queries.
## 2. Memory
- [ ] 2.1 Replace client-supplied user_id in /api/memory with UserContext identity.
## 3. Knowledge
- [ ] 3.1 Add owner scoping to KBs/documents/chunks across providers and API handlers.
- [ ] 3.2 Remove the all-KB retrieval fallback in the run manager; fail closed.
## 4. Proof
- [ ] 4.1 Cross-user bleed integration tests (threads, memory, KB) under two JWT identities.
- [ ] 4.2 Document shared-admin resources decision (O4) in the compatibility policy.
