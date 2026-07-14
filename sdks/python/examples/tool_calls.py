"""Function-tool request."""

import asyncio

from universal_agent_runtime_sdk import Client, ToolDefinition


async def main() -> None:
    weather = ToolDefinition(
        function={
            "name": "weather",
            "description": "Get weather",
            "parameters": {
                "type": "object",
                "properties": {"city": {"type": "string"}},
                "required": ["city"],
            },
        }
    )
    async with Client("http://localhost:1906") as client:
        print(await client.call_tools("Weather in Chicago?", [weather]))


if __name__ == "__main__":
    asyncio.run(main())
