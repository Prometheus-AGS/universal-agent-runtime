# universal-agent-runtime Python SDK

Python SDK for universal-agent-runtime.

## Installation

```bash
pip install universal-agent-runtime-sdk
# or
uv add universal-agent-runtime-sdk
```

## Usage

```python
import asyncio
from universal_agent_runtime_sdk import Client

async def main():
    client = Client("http://localhost:3000")
    
    # Chat API
    response = await client.chat("Hello!")
    print(f"Session: {response.session_id}")
    
    # Knowledge Base API
    kbs = await client.list_knowledge_bases()
    for kb in kbs:
        print(f"KB: {kb.name} ({kb.id})")
    
    # Search
    results = await client.search("kb-id", "query")
    for result in results.results:
        print(f"Score: {result.score:.2f} - {result.content}")
    
    await client.close()

asyncio.run(main())
```

### Context Manager

```python
async with Client("http://localhost:3000") as client:
    response = await client.chat("Hello!")
```

## License

MIT
