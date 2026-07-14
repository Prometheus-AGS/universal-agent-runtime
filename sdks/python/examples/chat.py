"""Basic chat completion."""

import asyncio

from universal_agent_runtime_sdk import Client


async def main() -> None:
    async with Client("http://localhost:1906") as client:
        result = await client.chat("Hello from Python")
        print(result.choices[0].message.content)


if __name__ == "__main__":
    asyncio.run(main())
