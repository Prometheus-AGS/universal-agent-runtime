"""Run two chat turns through the shipped Python SDK; no credentials.

Reads JSON inputs on stdin and writes one JSON receipt to stdout.
"""
from __future__ import annotations

import asyncio
import json
import sys

import httpx
from universal_agent_runtime_sdk import Client


async def turn(client: Client, mode: str, text: str, options: dict) -> dict:
    if mode == "json":
        value = await client.chat(text, **options)
        return {"id": value.id, "model": value.model, "choices": len(value.choices),
                "finish_reason": value.choices[0].finish_reason, "content": value.choices[0].message.content}
    chunks, ids, finishes, events = [], set(), [], []
    async for event in client.stream_chat(text, **options):
        events.append(event.event)
        data = event.data
        if not isinstance(data, dict) or data.get("object") != "chat.completion.chunk":
            continue
        ids.add(data.get("id"))
        for choice in data.get("choices", []):
            fragment = (choice.get("delta") or {}).get("content")
            if isinstance(fragment, str) and fragment:
                chunks.append(fragment)
            if choice.get("finish_reason") is not None:
                finishes.append(choice["finish_reason"])
    return {"ids": sorted(i for i in ids if i), "content_chunks": len(chunks), "finish_reasons": finishes,
            "events": sorted(set(events)), "content": "".join(chunks)}


async def main() -> None:
    inputs = json.load(sys.stdin)
    http = httpx.AsyncClient(headers={"x-uar-workspace-id": inputs["workspace"]}, timeout=inputs["timeout"])
    turns = []
    async with Client(inputs["base_url"], http_client=http) as client:
        for text in inputs["turns"]:
            options = {"model": inputs["model"], "session_id": inputs["session_id"],
                       "agent_id": inputs["agent_id"], "memory_enabled": False}
            turns.append(await turn(client, inputs["mode"], text, options))
    await http.aclose()
    json.dump({"client": "python_sdk", "mode": inputs["mode"], "turns": turns}, sys.stdout)


if __name__ == "__main__":
    asyncio.run(main())
