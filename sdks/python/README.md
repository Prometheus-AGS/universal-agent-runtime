# Universal Agent Runtime Python SDK

> **Current authority:** [Python SDK guide](/docs/sdk-python/intro). The source
> package is checked in at version 1.0.0; registry availability is release evidence
> and is not inferred from this README.

The Python SDK is a typed asynchronous HTTP/SSE client for UAR. It covers chat,
tools, structured output, embeddings, runs and checkpoints, knowledge bases,
documents, search, and ingestion. It requires Python 3.10 or newer.

## Use the checked-in source

```bash
cd sdks/python
uv sync --locked
UAR_BASE_URL=http://127.0.0.1:1906 uv run python examples/chat.py
```

Examples that make network requests require a running UAR server and valid
credentials. Build the package-local Sphinx reference with:

```bash
uv sync --locked --extra dev
uv run sphinx-build -W -b html docs docs/_build/html
```

`pyproject.toml` names the registry project
`universal-agent-runtime-sdk`. Before using `pip install`, verify the exact
version, files, integrity, and publisher on PyPI. Local metadata is not proof
that the registry artifact exists.

This SDK targets HTTP/SSE server profiles. It does not embed or certify the
transport-free `embedded-mobile` profile.
