## ADDED Requirements

### Requirement: Configurable log output format
The server SHALL support configurable log output format via `UAR_LOG_FORMAT` environment variable with values `json`, `compact`, and `pretty`.

#### Scenario: JSON format in production
- **WHEN** `UAR_LOG_FORMAT=json` is set
- **THEN** all log lines are emitted as single-line JSON objects with fields: `timestamp`, `level`, `target`, `message`, `span`, `fields`

#### Scenario: Pretty format for development
- **WHEN** `UAR_LOG_FORMAT=pretty` is set
- **THEN** log lines use human-readable colored format with indentation

#### Scenario: Default format
- **WHEN** `UAR_LOG_FORMAT` is not set
- **THEN** the server defaults to `json` format

### Requirement: JSON logs are K8s-compatible
Each JSON log line SHALL be parseable by standard Kubernetes log aggregators (Fluentd, Fluent Bit, Loki, CloudWatch).

#### Scenario: Single-line JSON output
- **WHEN** a log event is emitted in JSON format
- **THEN** the output is a single line (no embedded newlines outside of JSON string values)

#### Scenario: Timestamp format
- **WHEN** a log event is emitted in JSON format
- **THEN** the `timestamp` field uses RFC 3339 format
