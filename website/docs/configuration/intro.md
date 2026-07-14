# Configuration

UAR is configured through a layered config system: defaults, YAML files, environment variables, and (optionally) a Vault backend.

## Quick start

Copy `example.config.yaml` to `config.yaml` and edit it for your environment. Sensitive values such as `JWT_SECRET` and `LLM__API_KEY` should be provided through environment variables or a secret store.

## Key files

- `example.config.yaml` — full reference configuration
- `config.embedded.yaml` — embedded/SurrealDB preset
- `config.remote.postgres.yaml` — remote PostgreSQL preset
- `config.remote.surreal.yaml` — remote SurrealDB preset
- `config.test.yaml` — test preset

## Topics

- Layering and precedence
- Hot reload
- Secret handling with `secrecy`
- JSON Schema export via `GET /.well-known/uar-config`
