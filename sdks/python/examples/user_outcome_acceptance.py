"""Actual Python SDK two-turn acceptance client; no runtime or mock startup.

Called only by the frozen operator-approved packet. Missing inputs/auth fail.
"""
from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path

from universal_agent_runtime_sdk import Client


def require(condition, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


async def acceptance(inputs: dict) -> dict:
    authorization = os.environ[inputs["authorization_env"]]
    require(authorization.startswith("Bearer ") and len(authorization) > 7, "bearer credential required")
    turns = []
    async with Client(inputs["base_url"], api_key=authorization[7:], timeout=inputs["timeout_seconds"]) as client:
        for text, expected in zip(inputs["turns"], inputs["expected"]):
            options = {"model": inputs["model"], "session_id": inputs["session_id"],
                       "agent_id": inputs["agent_id"], "memory_enabled": False}
            if inputs["mode"] == "json":
                value = await client.chat(text, **options)
                require(len(value.choices) == 1 and value.choices[0].finish_reason == "stop", "SDK completion failed")
                require(value.model == inputs["model"], "SDK completion model mismatch")
                content, identifier = value.choices[0].message.content, value.id
            else:
                chunks, identifiers = [], set()
                terminal = 0
                async for event in client.stream_chat(text, **options):
                    data = event.data
                    if data == "[DONE]":
                        continue
                    require(isinstance(data, dict) and not data.get("error"), "SDK stream error")
                    require(event.event not in ("error", "agui.error", "agui.cancelled"), "SDK stream failure terminal")
                    if data.get("object") != "chat.completion.chunk":
                        continue
                    identifiers.add(data.get("id"))
                    for choice in data.get("choices", []):
                        fragment = choice.get("delta", {}).get("content")
                        if isinstance(fragment, str):
                            chunks.append(fragment)
                        if choice.get("finish_reason") is not None:
                            require(choice["finish_reason"] == "stop", "SDK stream did not stop successfully")
                            terminal += 1
                    require(sum(len(chunk.encode()) for chunk in chunks) <= inputs["max_bytes"], "SDK stream output exceeded bound")
                require(terminal == 1 and len(identifiers) == 1, "SDK stream lacks unique successful terminal")
                content, identifier = "".join(chunks), identifiers.pop()
            require(isinstance(content, str) and json.loads(content) == expected, "SDK useful-result predicate failed")
            require(isinstance(identifier, str) and identifier.startswith("chatcmpl-"), "SDK native run correlation missing")
            turns.append({"content": content, "run_id": identifier.removeprefix("chatcmpl-"),
                          "session_id": inputs["session_id"], "protocol_terminal": "success"})
    return {"status": "PASS", "client": "python_sdk", "mode": inputs["mode"], "turns": turns}


def main() -> int:
    output = Path(os.environ["UAR_ACCEPTANCE_CLIENT_OUTPUT"])
    try:
        inputs = json.loads(os.environ["UAR_ACCEPTANCE_CLIENT_INPUT"])
        require(inputs.get("mode") in ("json", "stream") and len(inputs["turns"]) == len(inputs["expected"]) == 2,
                "exact two-turn client input required")
        receipt = asyncio.run(acceptance(inputs))
        code = 0
    except Exception as error:
        receipt, code = {"status": "FAIL", "client": "python_sdk", "error_type": type(error).__name__,
                         "error": str(error)[:8192], "http_status": getattr(error, "status", None)}, 1
    with output.open("x", encoding="utf-8") as stream:
        json.dump(receipt, stream, allow_nan=False)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
