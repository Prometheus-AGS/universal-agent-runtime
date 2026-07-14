"""Focused request/response contract tests."""

import json

import httpx
import pytest
from pydantic import BaseModel

from universal_agent_runtime_sdk import ApiError, Client


def response(request: httpx.Request) -> httpx.Response:
    if request.url.path == "/v1/chat/completions":
        body = json.loads(request.content)
        if body.get("response_format"):
            content = json.dumps({"answer": "typed"})
        else:
            content = "hello"
        return httpx.Response(
            200,
            json={
                "id": "chat-1",
                "choices": [{"index": 0, "message": {"role": "assistant", "content": content}}],
            },
        )
    if request.url.path == "/v1/embeddings":
        return httpx.Response(200, json={"data": [{"index": 0, "embedding": [0.1, 0.2]}]})
    if request.url.path == "/api/uar/runs" and request.method == "POST":
        return httpx.Response(
            200, json={"run_id": "run-1", "stream_url": "/api/uar/runs/run-1/stream"}
        )
    if request.url.path.endswith("/cancel"):
        return httpx.Response(200, json={"cancelled": True})
    if request.url.path.endswith("/checkpoints"):
        return httpx.Response(200, json={"run_id": "run-1", "checkpoints": []})
    if "/resume" in request.url.path:
        return httpx.Response(
            200,
            json={
                "run_id": "run-2",
                "stream_url": "/api/uar/runs/run-2/stream",
                "resumed_from_run_id": "run-1",
            },
        )
    if request.url.path == "/api/uar/knowledge-bases/":
        return httpx.Response(200, json=[])
    return httpx.Response(400, headers={"x-request-id": "req-1"}, json={"error": "bad request"})


@pytest.fixture
def client() -> Client:
    http = httpx.AsyncClient(transport=httpx.MockTransport(response))
    return Client("https://uar.test", http_client=http)


@pytest.mark.asyncio
async def test_chat_tools_and_structured_output(client: Client) -> None:
    completion = await client.chat("hello")
    assert completion.choices[0].message.content == "hello"
    tool_result = await client.call_tools(
        "hello", [{"type": "function", "function": {"name": "x"}}]
    )
    assert tool_result.id == "chat-1"

    class Answer(BaseModel):
        answer: str

    assert (await client.structured_output("hello", Answer)).answer == "typed"


@pytest.mark.asyncio
async def test_embeddings_and_run_lifecycle(client: Client) -> None:
    assert (await client.embeddings("hello")).data[0].embedding == [0.1, 0.2]
    run = await client.create_run({"id": "agent"}, "go")
    assert run.run_id == "run-1"
    assert (await client.cancel_run(run.run_id)).cancelled
    assert not (await client.list_checkpoints(run.run_id)).checkpoints
    assert (await client.resume_run(run.run_id, {"id": "agent"})).run_id == "run-2"


@pytest.mark.asyncio
async def test_knowledge_and_typed_api_error(client: Client) -> None:
    assert await client.list_knowledge_bases() == []
    with pytest.raises(ApiError) as raised:
        await client.get_knowledge_base("missing")
    assert raised.value.status == 400
    assert raised.value.request_id == "req-1"


@pytest.mark.asyncio
async def test_stream_chat_decodes_sse() -> None:
    def stream_response(request: httpx.Request) -> httpx.Response:
        return httpx.Response(
            200,
            headers={"content-type": "text/event-stream"},
            content=b'id: 7\nevent: delta\ndata: {"text":"hi"}\n\ndata: [DONE]\n\n',
        )

    http = httpx.AsyncClient(transport=httpx.MockTransport(stream_response))
    client = Client("https://uar.test", http_client=http)
    events = [event async for event in client.stream_chat("hello")]
    assert events[0].id == "7"
    assert events[0].data == {"text": "hi"}
