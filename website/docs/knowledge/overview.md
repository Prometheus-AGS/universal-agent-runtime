---
sidebar_position: 1
title: Use Knowledge
description: Ingest durable source material, retrieve ranked chunks, attach a knowledge base to execution, and inspect citations.
source_records:
  - openspec/specs/knowledge-rag-product-certification/spec.md
  - openspec/specs/rag-provenance/spec.md
  - .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/functional-verification.md
current_authority: /docs/knowledge/overview
---

# Use knowledge

## Boundary statement

**A knowledge base stores operator-supplied sources for retrieval; a model does
not “know” a document merely because it was uploaded.** The document must finish
processing, retrieval must return a qualifying chunk, the knowledge base must be
attached to the run, and the resulting context and citation must be observable.

## Knowledge is not memory

Knowledge resources are named collections of documents and indexed chunks.
Agent memory is a separate opt-in system for captured or explicitly written
records across user, agent, session, global, and task scopes. A selected model's
conversation context and a live citation event are also separate: neither is a
durable knowledge document.

| State | Authority | Use in a run | Observable boundary |
|---|---|---|---|
| Knowledge base and document | Configured persistence, tenant/user scoped | Selected by agent or conversation policy | Knowledge admin API/UI |
| Indexed chunks | Configured persistence plus the current embedding pipeline | Ranked retrieval context | Search results and retrieval audit |
| Citation | Runtime event derived from a retrieved chunk | Shows what source contributed | Chat source badge / normalized event |
| Model output | Provider response | May use retrieved context | Completion text; validate against the source fact |

## Packaged UI workflow

1. Open **Admin → Knowledge** at `/admin/knowledge` and choose **Add Knowledge
   Base**. Create a name and optional description.
2. Select the knowledge base and upload a supported document. Upload acceptance
   creates a pending record; it does not mean indexing completed.
3. Watch processing move from `pending` through `processing` to `indexed`.
   A failed document displays its error and offers retry with the original or a
   replacement file. Never treat `failed` as indexed.
4. Search the knowledge base for a distinctive fact. Require a non-empty ranked
   result above the selected threshold.
5. In Chat, attach the knowledge base directly with the Knowledge capability or
   select an agent whose knowledge configuration includes it.
6. Ask a question whose answer depends on the indexed source. Confirm the answer
   contains the source fact and the transcript displays the source document as
   a citation.

Deleting a knowledge base or document removes its durable source and search
data. Treat those UI actions as destructive.

## API workflow

The knowledge resource is mounted at `/api/knowledge`:

| Stage | Request | Observable result |
|---|---|---|
| Create | `POST /api/knowledge` | `201 Created` with a tenant/user-owned knowledge-base ID. |
| Ingest | Multipart `POST /api/knowledge/{kb_id}/documents` | `202 Accepted` with a pending document record. |
| Observe processing | `GET /api/knowledge/{kb_id}/documents/{document_id}` | `pending`, `processing`, `indexed`, or `failed` with an error. |
| Retrieve | `POST /api/knowledge/{kb_id}/search` | Ranked chunks with score, metadata, and document ID. |
| Attach | `POST /api/uar/sessions/{conversation_id}/agent-config` or conversation policy API | Knowledge-base selection for the next resolved run. |
| Run | `POST /v1/chat/completions` with the same session identifier | A genuine model response through the selected conversation. |
| Inspect provenance | Stream the run/chat response | Knowledge-base/document citation events and source identity. |

The search and chat paths share the hardened retrieval pipeline: query
decomposition, per-query retrieval, deduplication, verification annotation,
result limiting, and a `rag.retrieval.decision` audit record occur before
context injection.

## Durable authority and live state

Configured persistence is the durable authority for knowledge bases, documents,
document status, chunks, and ownership. The UI polls while documents are pending
or processing and projects the returned state; a local upload-progress message
is not durable status.

Retrieval results and citations are run-scoped observations. A citation names
the knowledge base, document, and source used for that run, but it does not copy
the entire durable source into the event stream. A process-local result list is
not a replacement for persisted documents or a subsequent retrieval.

## Genuine model use

To claim grounded inference, use a fact that is absent from the prompt except
through the selected document. Observe all four boundaries: indexed document,
non-zero ranked retrieval, source citation, and the genuine model's answer.

The retained 2026-08-22 `server-full` record, source SHA
`d41bf7c3a447869896664d44ac0563e1b4a1d9f3`, observed one API-created knowledge
base return a ranked match with score `0.8674219` and influence one genuine
`openai/gpt-5.4-mini` request. It separately observed the packaged UI create a
knowledge base, upload and index a text file, select it, return the distinctive
fact, and display that file as the source. These results apply only to those
artifacts, provider/model, profile, checkout, and date.

## Failure boundaries

- Upload acceptance without an ingestion pool leaves the document saved but not
  processed.
- A failed extraction or embedding stays failed and exposes its error.
- A search result below the configured threshold is not injected.
- An attached name or ID outside the resolved policy universe is not authority
  to read another tenant's knowledge.
- A plausible model answer without retrieval and citation evidence is not a
  grounded-inference result.

## Profile limits

- `server-full` includes the packaged Knowledge UI and the server ingestion,
  retrieval, chat, and citation paths described here.
- `minimal` contains the server and persistence base but does not inherit the
  full document-intelligence/admin-UI claim. Verify its actual feature
  composition before promising ingestion formats or UI behavior.
- `embedded-mobile` exposes no knowledge HTTP/UI path. The host supplies
  persistence and an embedding backend and calls the shared transport-free
  knowledge/run services in process.

No retained result transfers to another profile. Continue with [Use agent
memory](/docs/memory/overview).
