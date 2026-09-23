## Purpose

Lets a host tell UAR, per run, which directory the agent works in and how much reasoning effort the model should spend, so an embedding application keeps those agent settings without UAR configuration changes.

## ADDED Requirements

### Requirement: A run accepts a validated working directory
`POST /api/uar/runs` SHALL accept an optional `working_directory`. The value SHALL be an absolute path that exists and resolves, after following symbolic links, to a directory that is not a filesystem root. Otherwise the request SHALL be rejected with HTTP 422 and code `working_directory_invalid`, and no run SHALL start. The run SHALL use the resolved path, not the submitted string.

#### Scenario: Relative path
- **WHEN** a run is created with `working_directory: "project"`
- **THEN** the response is 422 with `working_directory_invalid`

#### Scenario: Path to a file
- **WHEN** `working_directory` names an existing regular file
- **THEN** the response is 422 with `working_directory_invalid`

#### Scenario: Symbolic link to a directory
- **WHEN** `working_directory` is a symbolic link to a directory
- **THEN** the run starts and its recorded working directory is the link's resolved target

### Requirement: The working directory governs file and terminal tools for the run
For a run with a working directory, UAR SHALL use it as the run's world-state directory, as the default working directory of `terminal_exec` when the model supplies none, and as the base for relative paths in `terminal_exec` and file tools. File tools SHALL be confined to the working directory for that run. When the operator has configured file-tool roots, the working directory SHALL lie inside one of them, or the request SHALL be rejected with HTTP 422 and code `working_directory_not_allowed`. When no roots are configured, a working directory SHALL become the run's only file-tool root only on a launch-token-authenticated sidecar request; elsewhere file tools SHALL keep their configured behavior. Child runs SHALL inherit the parent's working directory and roots. Two concurrent runs with different working directories SHALL NOT read or write inside each other's directory through file tools.

#### Scenario: Terminal default directory
- **WHEN** a run with working directory `/w/a` calls `terminal_exec` with `pwd` and no `working_dir`
- **THEN** the output is `/w/a`

#### Scenario: Concurrent runs in different workspaces
- **WHEN** a run in `/w/a` calls `file_read` on `../b/secret.txt` and on `/w/b/secret.txt` while a concurrent run in `/w/b` reads `secret.txt`
- **THEN** both reads by the first run are refused and the second run reads its own file

#### Scenario: Directory outside configured roots
- **WHEN** the operator configured file roots `/srv/work` and a run supplies `/home/u/project`
- **THEN** the response is 422 with `working_directory_not_allowed`

### Requirement: A run accepts a reasoning effort
`POST /api/uar/runs` SHALL accept an optional `reasoning_effort` whose value is one of `none`, `low`, `medium`, `high`, `max`. Any other value SHALL be rejected with HTTP 422 and code `reasoning_effort_invalid`. When present it SHALL replace the configuration-derived reasoning setting for every model request of the run and its child runs: `none` SHALL send no reasoning parameters, and the other values SHALL map to the dialect's reasoning parameter so that a higher value never requests less reasoning than a lower one. When absent, behavior SHALL be unchanged.

#### Scenario: High effort on a reasoning dialect
- **WHEN** a run with `reasoning_effort: "high"` targets a model whose dialect supports extended thinking
- **THEN** the provider request carries that dialect's reasoning parameter at a level no lower than for `medium`

#### Scenario: None
- **WHEN** a run sets `reasoning_effort: "none"` and UAR's configuration enables a thinking budget
- **THEN** the provider request carries no reasoning parameter

#### Scenario: Invalid value
- **WHEN** a run sets `reasoning_effort: "ultra"`
- **THEN** the response is 422 with `reasoning_effort_invalid`

### Requirement: Host context is recorded on the run
UAR SHALL record the resolved working directory and the reasoning effort, when supplied, in the run record returned by the runs API, so a caller can confirm what the run used.

#### Scenario: Read back
- **WHEN** a run is created with a working directory and `reasoning_effort: "low"`
- **THEN** the run record shows the resolved directory and `low`
