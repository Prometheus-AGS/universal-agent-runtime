## ADDED Requirements

### Requirement: sandbox__code_exec Tool

The system SHALL provide a `sandbox__code_exec` NativeTool that executes code in a sandboxed environment. The tool SHALL accept the following parameters:
- `language: Language` (required) — the language runtime to use.
- `code: String` (required) — the source code to execute.
- `session_id: String` (required) — identifies the session for sandbox reuse.
- `timeout: Option<u64>` — per-execution timeout in seconds.

The tool SHALL create or reuse a sandbox for the given session via `SessionSandboxManager`, execute the code, and return the `ExecutionResult` as a JSON tool response.

#### Scenario: Executing Python code
- **WHEN** `sandbox__code_exec` is called with `language: "python"`, `code: "print(2+2)"`, and `session_id: "s1"`
- **THEN** the tool SHALL return a result with `exit_code: 0` and `stdout` containing `"4"`.

#### Scenario: Code execution timeout
- **WHEN** `sandbox__code_exec` is called with `timeout: 2` and the code runs an infinite loop
- **THEN** the tool SHALL terminate the execution after 2 seconds and return a result indicating a timeout error.

### Requirement: sandbox__shell_exec Tool

The system SHALL provide a `sandbox__shell_exec` NativeTool that executes shell commands in a sandboxed environment. The tool SHALL accept:
- `command: String` (required) — the shell command to execute.
- `session_id: String` (required) — identifies the session for sandbox reuse.
- `timeout: Option<u64>` — per-execution timeout in seconds.

The tool SHALL execute the command via `Language::Bash` in the session's sandbox.

#### Scenario: Running a shell command
- **WHEN** `sandbox__shell_exec` is called with `command: "ls /workspace"` and `session_id: "s1"`
- **THEN** the tool SHALL return a result with the directory listing of `/workspace` in stdout.

#### Scenario: Shell command with non-zero exit
- **WHEN** `sandbox__shell_exec` is called with `command: "exit 42"`
- **THEN** the tool SHALL return a result with `exit_code: 42`.

### Requirement: sandbox__file_read Tool

The system SHALL provide a `sandbox__file_read` NativeTool that reads a file from inside a sandbox. The tool SHALL accept:
- `session_id: String` (required) — identifies the session whose sandbox to read from.
- `path: String` (required) — the absolute path inside the sandbox to read.

The tool SHALL return the file contents as a UTF-8 string in the tool response. For binary files, the contents SHALL be base64-encoded.

#### Scenario: Reading a text file
- **WHEN** `sandbox__file_read` is called with `path: "/workspace/output.txt"` and the file contains `"result: 42"`
- **THEN** the tool SHALL return the string `"result: 42"`.

#### Scenario: Reading a nonexistent file
- **WHEN** `sandbox__file_read` is called with `path: "/workspace/missing.txt"` and the file does not exist
- **THEN** the tool SHALL return an error indicating the file was not found.

### Requirement: sandbox__file_write Tool

The system SHALL provide a `sandbox__file_write` NativeTool that writes content to a file inside a sandbox. The tool SHALL accept:
- `session_id: String` (required) — identifies the session whose sandbox to write to.
- `path: String` (required) — the absolute path inside the sandbox to write.
- `content: String` (required) — the content to write to the file.

#### Scenario: Writing a file and reading it back
- **WHEN** `sandbox__file_write` is called with `path: "/workspace/data.txt"` and `content: "hello"`, followed by `sandbox__file_read` with the same path
- **THEN** the read SHALL return `"hello"`.

#### Scenario: Writing creates parent directories
- **WHEN** `sandbox__file_write` is called with `path: "/workspace/nested/dir/file.txt"` and the intermediate directories do not exist
- **THEN** the tool SHALL create the necessary parent directories and write the file successfully.

### Requirement: Tool Registration at Startup

All four sandbox tools (`sandbox__code_exec`, `sandbox__shell_exec`, `sandbox__file_read`, `sandbox__file_write`) SHALL be registered in the `McpRegistry` at application startup. They SHALL appear as native tools alongside any MCP-discovered tools.

#### Scenario: Tools appear in tool listing
- **WHEN** the application starts with a sandbox runner available
- **THEN** the McpRegistry SHALL contain entries for all four `sandbox__*` tools with proper JSON Schema parameter definitions.

#### Scenario: Tools unavailable without runner
- **WHEN** the application starts and no sandbox runner is available (all fallbacks exhausted)
- **THEN** the four `sandbox__*` tools SHALL NOT be registered in the McpRegistry.
