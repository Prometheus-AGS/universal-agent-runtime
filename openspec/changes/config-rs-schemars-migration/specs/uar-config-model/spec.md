# UAR config model

## Purpose

Give `AppConfig` a live, introspectable JSON Schema, and stop the
JWT signing secret from being representable as a plain, accidentally
loggable/serializable `String`.

## ADDED Requirements

### Requirement: jwt_secret is a SecretString
`SecurityConfig::jwt_secret` MUST be `secrecy::SecretString`, not
`String`. Every call site that needs the plaintext value MUST call
`secrecy::ExposeSecret::expose_secret()` explicitly.

#### Scenario: A call site needs the plaintext JWT secret
- **WHEN** code needs to sign or verify a JWT
- **THEN** it calls `config.security.jwt_secret.expose_secret()`
  to get the plaintext `&str`
- **AND** any attempt to `serde_json::to_value`/`Serialize` the
  `SecretString` directly fails to compile (by design — the value
  cannot be serialized without an explicit `expose_secret()` call)

### Requirement: Canonical JSON Schema for AppConfig
`AppConfig::json_schema() -> serde_json::Value` MUST exist,
generated via `schemars::schema_for!(AppConfig)` from the live
struct definitions (not a hand-maintained duplicate). Every struct
and enum reachable from `AppConfig` MUST derive
`schemars::JsonSchema`.

#### Scenario: The schema is generated
- **WHEN** `AppConfig::json_schema()` is called
- **THEN** it returns a JSON Schema document whose top-level
  `properties` include every field of `AppConfig` (`server`,
  `security`, `llm`, `context_strategy`, etc.)

### Requirement: Schema endpoint
`GET /.well-known/uar-config` MUST serve `AppConfig::json_schema()`
as the response body, additively alongside existing routes.

#### Scenario: A client fetches the config schema
- **WHEN** a client sends `GET /.well-known/uar-config`
- **THEN** the server responds with the JSON Schema document
- **AND** no existing route's behavior changes

### Requirement: Secrets are opaque strings in the schema
Fields backed by `SecretString` (currently `jwt_secret`) MUST be
represented in the generated schema as a plain `{"type": "string"}`
— the schema describes shape, never the actual secret value, and
does not require `SecretString` itself to implement `JsonSchema`.

#### Scenario: The schema is inspected for the JWT secret field
- **WHEN** a schema consumer resolves `security.jwt_secret` in the
  generated schema (directly or via a `$ref`)
- **THEN** it finds `{"type": "string"}`
- **AND** no actual secret value appears anywhere in the schema
  document
