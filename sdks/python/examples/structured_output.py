"""Pydantic-validated structured output."""

import asyncio

from pydantic import BaseModel

from universal_agent_runtime_sdk import Client


class Answer(BaseModel):
    summary: str
    confidence: float


async def main() -> None:
    async with Client("http://localhost:1906") as client:
        print(await client.structured_output("Summarize UAR", Answer))


if __name__ == "__main__":
    asyncio.run(main())
