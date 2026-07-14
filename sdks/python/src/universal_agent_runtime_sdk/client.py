"""Asynchronous, typed HTTP and SSE client for UAR."""

import json
from collections.abc import AsyncIterator, Mapping, Sequence
from pathlib import Path
from typing import Any, TypeVar

import httpx
from httpx_sse import aconnect_sse
from pydantic import BaseModel

from .types import (
    CancelRunResponse,
    ChatCompletion,
    CheckpointList,
    CreateKnowledgeBaseRequest,
    Document,
    EmbeddingResponse,
    IngestResponse,
    KnowledgeBase,
    Message,
    RunResponse,
    SearchResponse,
    StreamEvent,
    ToolDefinition,
)

ModelT = TypeVar("ModelT", bound=BaseModel)


class UarError(Exception):
    """Base exception for SDK failures."""


class TransportError(UarError):
    """Network or protocol failure."""


class ApiError(UarError):
    """Non-success response returned by UAR."""

    def __init__(self, status: int, message: str, *, request_id: str | None = None) -> None:
        super().__init__(message)
        self.status = status
        self.message = message
        self.request_id = request_id


class Client:
    """Typed asynchronous client for Universal Agent Runtime."""

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 30.0,
        http_client: httpx.AsyncClient | None = None,
    ) -> None:
        headers = {"Authorization": f"Bearer {api_key}"} if api_key else {}
        self.base_url = base_url.rstrip("/")
        self._owns_http = http_client is None
        self._http = http_client or httpx.AsyncClient(headers=headers, timeout=timeout)

    async def __aenter__(self) -> "Client":
        return self

    async def __aexit__(self, *args: object) -> None:
        await self.close()

    async def close(self) -> None:
        """Close the underlying client when this instance created it."""
        if self._owns_http:
            await self._http.aclose()

    async def chat(
        self,
        messages: str | Sequence[Message | Mapping[str, Any]],
        *,
        model: str | None = None,
        tools: Sequence[ToolDefinition | Mapping[str, Any]] | None = None,
        response_format: Mapping[str, Any] | None = None,
        **options: Any,
    ) -> ChatCompletion:
        """Create a non-streaming OpenAI-compatible chat completion."""
        payload = self._chat_payload(messages, model, tools, response_format, options)
        payload["stream"] = False
        return await self._request_model(
            "POST", "/v1/chat/completions", ChatCompletion, json=payload
        )

    async def stream_chat(
        self,
        messages: str | Sequence[Message | Mapping[str, Any]],
        *,
        model: str | None = None,
        tools: Sequence[ToolDefinition | Mapping[str, Any]] | None = None,
        response_format: Mapping[str, Any] | None = None,
        **options: Any,
    ) -> AsyncIterator[StreamEvent]:
        """Stream an OpenAI-compatible chat completion as typed SSE events."""
        payload = self._chat_payload(messages, model, tools, response_format, options)
        payload["stream"] = True
        async for event in self._stream("POST", "/v1/chat/completions", json=payload):
            yield event

    async def call_tools(
        self,
        messages: str | Sequence[Message | Mapping[str, Any]],
        tools: Sequence[ToolDefinition | Mapping[str, Any]],
        *,
        model: str | None = None,
        tool_choice: str | Mapping[str, Any] = "auto",
    ) -> ChatCompletion:
        """Request a completion with function tools available."""
        return await self.chat(messages, model=model, tools=tools, tool_choice=tool_choice)

    async def structured_output(
        self,
        messages: str | Sequence[Message | Mapping[str, Any]],
        schema: type[ModelT],
        *,
        model: str | None = None,
        name: str = "response",
    ) -> ModelT:
        """Request JSON constrained by a Pydantic model and validate the result."""
        response_format = {
            "type": "json_schema",
            "json_schema": {"name": name, "strict": True, "schema": schema.model_json_schema()},
        }
        completion = await self.chat(messages, model=model, response_format=response_format)
        content = completion.choices[0].message.content
        if not isinstance(content, str):
            raise TransportError("structured response did not contain JSON text")
        return schema.model_validate_json(content)

    async def embeddings(
        self, input: str | Sequence[str], *, model: str | None = None
    ) -> EmbeddingResponse:
        """Create embeddings through the OpenAI-compatible endpoint."""
        payload: dict[str, Any] = {"input": input}
        if model is not None:
            payload["model"] = model
        return await self._request_model("POST", "/v1/embeddings", EmbeddingResponse, json=payload)

    async def create_run(
        self,
        artifact: Mapping[str, Any],
        input: str,
        *,
        session_id: str | None = None,
    ) -> RunResponse:
        """Create an agent run."""
        return await self._request_model(
            "POST",
            "/api/uar/runs",
            RunResponse,
            json={"artifact": dict(artifact), "input": input, "session_id": session_id},
        )

    async def stream_run(
        self, run_id: str, *, last_event_id: str | None = None, agui_spec: bool = False
    ) -> AsyncIterator[StreamEvent]:
        """Stream replayable events for a run."""
        headers = {"Last-Event-ID": last_event_id} if last_event_id else None
        params = {"stream_mode": "agui_spec"} if agui_spec else None
        async for event in self._stream(
            "GET", f"/api/uar/runs/{run_id}/stream", headers=headers, params=params
        ):
            yield event

    async def cancel_run(self, run_id: str) -> CancelRunResponse:
        """Idempotently cancel an in-flight run."""
        return await self._request_model(
            "POST", f"/api/uar/runs/{run_id}/cancel", CancelRunResponse
        )

    async def list_checkpoints(self, run_id: str) -> CheckpointList:
        """List persisted checkpoints for a run."""
        return await self._request_model(
            "GET", f"/api/uar/runs/{run_id}/checkpoints", CheckpointList
        )

    async def resume_run(
        self,
        run_id: str,
        artifact: Mapping[str, Any],
        *,
        input: str | None = None,
        session_id: str | None = None,
        checkpoint_id: str | None = None,
    ) -> RunResponse:
        """Resume from the latest or a selected checkpoint."""
        suffix = f"/{checkpoint_id}" if checkpoint_id else ""
        return await self._request_model(
            "POST",
            f"/api/uar/runs/{run_id}/resume{suffix}",
            RunResponse,
            json={"artifact": dict(artifact), "input": input, "session_id": session_id},
        )

    async def list_knowledge_bases(self) -> list[KnowledgeBase]:
        """List knowledge bases."""
        data = await self._request_json("GET", "/api/uar/knowledge-bases/")
        return [KnowledgeBase.model_validate(value) for value in data]

    async def create_knowledge_base(self, request: CreateKnowledgeBaseRequest) -> KnowledgeBase:
        """Create a knowledge base."""
        return await self._request_model(
            "POST",
            "/api/uar/knowledge-bases/",
            KnowledgeBase,
            json=request.model_dump(exclude_none=True),
        )

    async def get_knowledge_base(self, kb_id: str) -> KnowledgeBase:
        """Get a knowledge base."""
        return await self._request_model("GET", f"/api/uar/knowledge-bases/{kb_id}", KnowledgeBase)

    async def update_knowledge_base(self, kb_id: str, **updates: Any) -> KnowledgeBase:
        """Update knowledge-base fields."""
        return await self._request_model(
            "PUT", f"/api/uar/knowledge-bases/{kb_id}", KnowledgeBase, json=updates
        )

    async def delete_knowledge_base(self, kb_id: str) -> None:
        """Delete a knowledge base."""
        await self._request_json("DELETE", f"/api/uar/knowledge-bases/{kb_id}")

    async def list_documents(self, kb_id: str) -> list[Document]:
        """List documents in a knowledge base."""
        data = await self._request_json("GET", f"/api/uar/knowledge-bases/{kb_id}/documents")
        return [Document.model_validate(value) for value in data]

    async def upload_document(
        self, kb_id: str, path: str | Path, *, content_type: str = "application/octet-stream"
    ) -> Document:
        """Upload a document to a knowledge base."""
        file_path = Path(path)
        with file_path.open("rb") as handle:
            return await self._request_model(
                "POST",
                f"/api/uar/knowledge-bases/{kb_id}/documents",
                Document,
                files={"file": (file_path.name, handle, content_type)},
            )

    async def search(
        self, kb_id: str, query: str, *, limit: int = 5, min_score: float = 0.7
    ) -> SearchResponse:
        """Search a knowledge base."""
        return await self._request_model(
            "POST",
            f"/api/uar/knowledge-bases/{kb_id}/search",
            SearchResponse,
            json={"query": query, "limit": limit, "min_score": min_score},
        )

    async def ingest(
        self, path: str | Path, *, content_type: str = "application/octet-stream"
    ) -> IngestResponse:
        """Upload a file to the default ingestion endpoint."""
        file_path = Path(path)
        with file_path.open("rb") as handle:
            return await self._request_model(
                "POST",
                "/api/ingest",
                IngestResponse,
                files={"file": (file_path.name, handle, content_type)},
            )

    def _chat_payload(
        self,
        messages: str | Sequence[Message | Mapping[str, Any]],
        model: str | None,
        tools: Sequence[ToolDefinition | Mapping[str, Any]] | None,
        response_format: Mapping[str, Any] | None,
        options: Mapping[str, Any],
    ) -> dict[str, Any]:
        values: Sequence[Message | Mapping[str, Any]] = (
            [Message(role="user", content=messages)] if isinstance(messages, str) else messages
        )
        payload: dict[str, Any] = {
            "messages": [
                value.model_dump(exclude_none=True) if isinstance(value, BaseModel) else dict(value)
                for value in values
            ],
            **options,
        }
        if model is not None:
            payload["model"] = model
        if tools is not None:
            payload["tools"] = [
                value.model_dump(exclude_none=True) if isinstance(value, BaseModel) else dict(value)
                for value in tools
            ]
        if response_format is not None:
            payload["response_format"] = dict(response_format)
        return payload

    async def _request_model(
        self, method: str, path: str, model: type[ModelT], **kwargs: Any
    ) -> ModelT:
        return model.model_validate(await self._request_json(method, path, **kwargs))

    async def _request_json(self, method: str, path: str, **kwargs: Any) -> Any:
        try:
            response = await self._http.request(method, f"{self.base_url}{path}", **kwargs)
        except httpx.HTTPError as error:
            raise TransportError(str(error)) from error
        self._check_response(response)
        if response.status_code == 204 or not response.content:
            return None
        try:
            return response.json()
        except ValueError as error:
            raise TransportError("UAR returned invalid JSON") from error

    async def _stream(self, method: str, path: str, **kwargs: Any) -> AsyncIterator[StreamEvent]:
        try:
            async with aconnect_sse(
                self._http, method, f"{self.base_url}{path}", **kwargs
            ) as source:
                self._check_response(source.response)
                async for event in source.aiter_sse():
                    if event.data == "[DONE]":
                        return
                    try:
                        data: Any = json.loads(event.data)
                    except json.JSONDecodeError:
                        data = event.data
                    yield StreamEvent(
                        event=event.event or "message", data=data, id=event.id or None
                    )
        except httpx.HTTPError as error:
            raise TransportError(str(error)) from error

    @staticmethod
    def _check_response(response: httpx.Response) -> None:
        if response.is_success:
            return
        request_id = response.headers.get("x-request-id")
        try:
            body = response.json()
            message = body.get("message") or body.get("error") or response.text
        except ValueError:
            message = response.text
        raise ApiError(response.status_code, str(message), request_id=request_id)
