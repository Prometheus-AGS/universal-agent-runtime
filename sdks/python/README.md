# Universal Agent Runtime Python SDK

Stable, MIT-licensed, typed async client for UAR. It covers chat and SSE streaming, tools, Pydantic structured output, embeddings, the run lifecycle, knowledge bases, and ingestion.

```bash
pip install universal-agent-runtime-sdk
```

```python
import asyncio
from universal_agent_runtime_sdk import Client

async def main() -> None:
    async with Client("http://localhost:1906", api_key="...") as client:
        completion = await client.chat("Explain replayable agent runs in one sentence.")
        print(completion.choices[0].message.content)

asyncio.run(main())
```

Use `stream_chat` for chat SSE; use `create_run`, `stream_run`, `cancel_run`, `list_checkpoints`, and `resume_run` for agent execution. Six complete workflows live in `examples/`. API reference sources are in `docs/` and build with `sphinx-build -W -b html docs docs/_build/html`.

See `BREAKING.md` when upgrading from 0.1. The package supports Python 3.10+.
