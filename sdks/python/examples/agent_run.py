"""Create and stream an agent run."""

import asyncio

from universal_agent_runtime_sdk import Client

ARTIFACT = {"id": "example", "name": "Example", "version": "1", "nodes": [], "edges": []}


async def main() -> None:
    async with Client("http://localhost:1906") as client:
        run = await client.create_run(ARTIFACT, "Complete the task")
        async for event in client.stream_run(run.run_id):
            print(event.event, event.data)


if __name__ == "__main__":
    asyncio.run(main())
