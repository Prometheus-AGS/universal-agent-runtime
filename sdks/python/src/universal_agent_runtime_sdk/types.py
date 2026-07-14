"""Typed public models for the Universal Agent Runtime API."""

from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field


class UarModel(BaseModel):
    """Base model that tolerates additive server fields."""

    model_config = ConfigDict(extra="allow")


class Message(UarModel):
    """A chat message."""

    role: Literal["system", "user", "assistant", "tool"] | str
    content: str | list[dict[str, Any]] | None = None
    name: str | None = None
    tool_call_id: str | None = None


class ToolDefinition(UarModel):
    """OpenAI-compatible function tool definition."""

    type: Literal["function"] = "function"
    function: dict[str, Any]


class ToolCall(UarModel):
    """A tool call returned by a model."""

    id: str
    type: str = "function"
    function: dict[str, Any]


class ChatChoice(UarModel):
    """One chat completion choice."""

    index: int = 0
    message: Message
    finish_reason: str | None = None


class ChatCompletion(UarModel):
    """A non-streaming chat completion."""

    id: str
    choices: list[ChatChoice]
    model: str | None = None
    usage: dict[str, int] | None = None


class Embedding(UarModel):
    """One embedding vector."""

    index: int
    embedding: list[float]
    object: str = "embedding"


class EmbeddingResponse(UarModel):
    """Response from the embeddings API."""

    data: list[Embedding]
    model: str | None = None
    usage: dict[str, int] | None = None


class StreamEvent(UarModel):
    """A decoded server-sent event."""

    event: str = "message"
    data: Any
    id: str | None = None
    retry: int | None = None


class RunResponse(UarModel):
    """Response from creating or resuming a run."""

    run_id: str
    stream_url: str
    resumed_from_run_id: str | None = None
    checkpoint_id: str | None = None


class CancelRunResponse(UarModel):
    """Result of idempotently cancelling a run."""

    cancelled: bool


class Checkpoint(UarModel):
    """A persisted run checkpoint."""

    id: str
    run_id: str
    node_id: str
    iteration: int
    state: dict[str, Any] = Field(default_factory=dict)
    created_at: str | None = None


class CheckpointList(UarModel):
    """Persisted checkpoints for one run."""

    run_id: str
    checkpoints: list[Checkpoint]


class KnowledgeBaseConfig(UarModel):
    """Knowledge-base configuration."""

    embedding_provider: str = ""
    embedding_model: str = ""
    vector_dimensions: int | None = None
    file_processor: str = ""
    chunk_strategy: str = ""


class KnowledgeBase(UarModel):
    """A knowledge base."""

    id: str
    name: str
    description: str | None = None
    config: KnowledgeBaseConfig = Field(default_factory=KnowledgeBaseConfig)
    created_at: str
    updated_at: str
    document_count: int = 0


class CreateKnowledgeBaseRequest(UarModel):
    """Values used to create a knowledge base."""

    name: str
    description: str | None = None
    config: dict[str, Any] | None = None


class Document(UarModel):
    """A document stored in a knowledge base."""

    id: str
    kb_id: str
    filename: str
    file_path: str | None = None
    mime_type: str | None = None
    chunk_count: int
    status: str
    error_message: str | None = None
    created_at: str | None = None
    updated_at: str | None = None


class SearchResult(UarModel):
    """A knowledge-search match."""

    content: str
    score: float
    metadata: dict[str, Any] = Field(default_factory=dict)
    document_id: str | None = None


class SearchResponse(UarModel):
    """Knowledge-search results."""

    results: list[SearchResult]


class IngestResponse(UarModel):
    """Result of uploading content for ingestion."""

    status: str
    message: str | None = None
    files: list[str] = Field(default_factory=list)
