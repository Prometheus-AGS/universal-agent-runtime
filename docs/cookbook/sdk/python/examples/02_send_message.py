"""Cookbook SDK (Python): send a chat message to the UAR runtime."""

import asyncio
import os

from universal_agent_runtime_sdk import Client


async def main() -> None:
    base_url = os.environ.get("UAR_BASE_URL", "http://localhost:1906")
    async with Client(base_url) as client:
        completion = await client.chat("Hello from the UAR cookbook")
        print("Response:", completion.choices[0].message.content)


if __name__ == "__main__":
    asyncio.run(main())
