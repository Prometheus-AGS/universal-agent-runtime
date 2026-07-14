"""Streaming chat completion."""

import asyncio

from universal_agent_runtime_sdk import Client


async def main() -> None:
    async with Client("http://localhost:1906") as client:
        async for event in client.stream_chat("Count to three"):
            print(event.data)


if __name__ == "__main__":
    asyncio.run(main())
