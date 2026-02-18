# Configuration Guide

The Universal Agent Runtime (UAR) uses a hierarchical configuration system that allows you to configure the application via:
1.  **Command Line Arguments** (highest priority)
2.  **Environment Variables**
3.  **Configuration File** (`config.yaml`)
4.  **Defaults** (lowest priority)

## 1. Command Line Arguments

Quickly override active settings when running the binary.

| Argument | Environment Variable | Description |
| :--- | :--- | :--- |
| `--config <PATH>` | `CONFIG_FILE` | Path to the configuration file. Defaults to `$HOME/.uar/config.yaml`. |
| `--port <PORT>` | `PORT` | Override the server listening port. |
| `--jwt-required` | `JWT_REQUIRED` | explicit boolean flag (e.g. `--jwt-required=false`). |
| `--rate-limit-enabled` | `RATE_LIMIT_ENABLED` | explicit boolean flag. |

Example:
```bash
./universal-agent-runtime --port 8080 --config ./my-config.yaml
```

## 2. Environment Variables

Settings are mapped to environment variables using double underscores (`__`) to separate sections.

| Config Key | Environment Variable | Default |
| :--- | :--- | :--- |
| `server.port` | `UAR_SERVER__PORT` | `3000` |
| `server.host` | `UAR_SERVER__HOST` | `0.0.0.0` |
| `security.jwt_required` | `UAR_SECURITY__JWT_REQUIRED` | `true` |
| `security.jwt_secret` | `UAR_SECURITY__JWT_SECRET` | `secret...` |
| `resilience.rate_limit_enabled` | `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | `true` |
| `persistence.provider` | `UAR_PERSISTENCE__PROVIDER` | `postgres` |
| `persistence.database_url` | `UAR_PERSISTENCE__DATABASE_URL` | `postgres://...` |

### LLM Settings (Special)
LLM configuration currently resides outside the main config structure and **must** be set via these environment variables:

- `LLM_BASE_URL` (Required): API endpoint (e.g., `https://api.openai.com/v1`).
- `LLM_MODEL` (Required): Model name (e.g., `gpt-4o`).
- `LLM_API_KEY`: API Key for the provider.
- `LLM_PROTOCOL`: Client behavior (`auto`, `chat`, `responses`).

These env vars are automatically imported as the **default** provider in the Provider Registry at startup.

### Multi-Provider Configuration

You can configure additional LLM providers via the `providers` section in your config file. Each provider supports per-agent selection and fallback chains.

```yaml
providers:
  - id: "openai"
    display_name: "OpenAI"
    base_url: "https://api.openai.com"
    api_key: "sk-..."
    protocol: auto
    default_model: "gpt-4o"
    enabled: true
    models:
      - id: "gpt-4o"
        context_window: 128000
        supports_vision: true
        supports_tools: true

  - id: "groq"
    display_name: "Groq"
    base_url: "https://api.groq.com/openai"
    api_key: "gsk_..."
    protocol: chat
    default_model: "llama-3.3-70b-versatile"
    enabled: true
```

Agents reference providers by ID in their artifact YAML:

```yaml
policy:
  provider:
    default: { provider: "groq", model: "llama-3.3-70b-versatile" }
    fallbacks:
      - { provider: "openai", model: "gpt-4o-mini" }
```

### Providers REST API

Providers can also be managed at runtime via the REST API:

| Method | Path | Description |
| :--- | :--- | :--- |
| `GET` | `/api/uar/providers` | List all providers + default ID |
| `GET` | `/api/uar/providers/{id}` | Get a single provider |
| `POST` | `/api/uar/providers` | Register a new provider |
| `PUT` | `/api/uar/providers/{id}` | Update a provider |
| `DELETE` | `/api/uar/providers/{id}` | Remove a provider |
| `GET` | `/api/uar/providers/{id}/models` | List models for a provider |
| `POST` | `/api/uar/providers/{id}/default` | Set a provider as default |


You can also place a `.env` file in the current directory to set environment variables. Default `.env` loading is supported.

## 3. Configuration File

The application looks for a configuration file in the following order:
1.  Specified via `--config` or `CONFIG_FILE`.
2.  `./config.yaml` (Current working directory).
3.  `~/.uar/config.yaml` (User home directory).

You can provide a YAML, JSON, or TOML file.

See `example.config.yaml` in the root of the repository for a complete reference.

```yaml
server:
  port: 8080
security:
  jwt_required: false
persistence:
  provider: "surreal"
  database_url: "ws://localhost:8000"
```
