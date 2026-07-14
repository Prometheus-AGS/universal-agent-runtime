"""Search a knowledge base."""

import asyncio

from universal_agent_runtime_sdk import Client


async def main() -> None:
    async with Client("http://localhost:1906") as client:
        result = await client.search("default", "What is UAR?")
        for match in result.results:
            print(match.score, match.content)


if __name__ == "__main__":
    asyncio.run(main())
