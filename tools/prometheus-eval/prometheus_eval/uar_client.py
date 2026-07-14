"""Thin HTTP client for the UAR runtime under test.

Produces the two things the golden set does not fix ahead of time
(``retrieved_contexts`` and ``response``/``actual_output``) by calling the
real running server: knowledge-base search for retrieval, then
``/v1/chat/completions`` for generation. Kept intentionally dumb — no
retries, no caching — this is a test harness, not a production client;
UAR's own SDKs (``sdks/python``) are the place for a real client.
"""

from __future__ import annotations

import os
from dataclasses import dataclass

import httpx

DEFAULT_BASE_URL = os.environ.get("UAR_EVAL_BASE_URL", "http://127.0.0.1:1906")
DEFAULT_KNOWLEDGE_BASE_ID = os.environ.get("UAR_EVAL_KNOWLEDGE_BASE_ID", "default")
DEFAULT_SEARCH_LIMIT = int(os.environ.get("UAR_EVAL_SEARCH_LIMIT", "5"))


@dataclass
class RagTrace:
    """What the pipeline under test actually produced for one question."""

    retrieved_contexts: list[str]
    response: str


class UarClient:
    """Talks to a running `universal-agent-runtime` server."""

    def __init__(
        self,
        base_url: str = DEFAULT_BASE_URL,
        api_key: str | None = None,
        model: str = "openai/gpt-4o-mini",
        knowledge_base_id: str = DEFAULT_KNOWLEDGE_BASE_ID,
        timeout: float = 60.0,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.model = model
        self.knowledge_base_id = knowledge_base_id
        # No blanket Content-Type header here: httpx sets the right one per
        # request (application/json for `json=`, multipart/form-data with a
        # boundary for `files=`) — a fixed client-level header would break
        # `upload_document`'s multipart request.
        headers = {}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        self._client = httpx.Client(base_url=self.base_url, headers=headers, timeout=timeout)

    def search(self, query: str, limit: int = DEFAULT_SEARCH_LIMIT) -> list[str]:
        """Retrieval-only: hits `/api/knowledge/{id}/search`."""
        resp = self._client.post(
            f"/api/knowledge/{self.knowledge_base_id}/search",
            json={"query": query, "limit": limit},
        )
        resp.raise_for_status()
        body = resp.json()
        # SearchResponse shape carries scored knowledge-base matches; the
        # exact field name is `results` with each item's text under
        # `content` per src/uar/api/knowledge.rs's SearchResponse/KnowledgeMatch.
        return [r.get("content", "") for r in body.get("results", [])]

    def chat(self, question: str) -> str:
        """Generation: hits the OpenAI-compatible `/v1/chat/completions`,
        which internally uses the configured knowledge base for RAG when
        the request/agent is configured to do so (server-side concern,
        not this client's).
        """
        resp = self._client.post(
            "/v1/chat/completions",
            json={
                "model": self.model,
                "messages": [{"role": "user", "content": question}],
            },
        )
        resp.raise_for_status()
        body = resp.json()
        return body["choices"][0]["message"]["content"]

    def run(self, question: str, search_limit: int = DEFAULT_SEARCH_LIMIT) -> RagTrace:
        return RagTrace(
            retrieved_contexts=self.search(question, limit=search_limit),
            response=self.chat(question),
        )

    def search_scored(
        self, query: str, limit: int, knowledge_base_id: str | None = None
    ) -> list[tuple[str, float]]:
        """Like `search`, but returns `(document_id, score)` pairs instead
        of chunk text — what BEIR's evaluator needs (per-document relevance
        scores), used by `beir_bench.make_search_fn`.
        """
        kb_id = knowledge_base_id or self.knowledge_base_id
        resp = self._client.post(
            f"/api/knowledge/{kb_id}/search",
            json={"query": query, "limit": limit},
        )
        resp.raise_for_status()
        body = resp.json()
        return [
            (r["document_id"], float(r["score"]))
            for r in body.get("results", [])
            if r.get("document_id") is not None
        ]

    def create_knowledge_base(self, name: str, description: str = "") -> str:
        """`POST /api/knowledge` — returns the new knowledge base's id."""
        resp = self._client.post(
            "/api/knowledge",
            json={"name": name, "description": description},
        )
        resp.raise_for_status()
        return resp.json()["id"]

    def upload_document(self, kb_id: str, filename: str, content: str) -> dict:
        """`POST /api/knowledge/{kb_id}/documents` (multipart) — returns the
        `DocumentResponse` JSON body (has `id`, `filename`, `status`, ...).
        """
        files = {"file": (filename, content.encode("utf-8"), "text/plain")}
        resp = self._client.post(f"/api/knowledge/{kb_id}/documents", files=files)
        resp.raise_for_status()
        return resp.json()

    def close(self) -> None:
        self._client.close()

    def __enter__(self) -> "UarClient":
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()
