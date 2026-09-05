## ADDED Requirements

### Requirement: Project instruction files are discovered from root to cwd
The runtime SHALL discover project instruction files by walking up from the working directory to a configured root marker, SHALL concatenate every matching file from the root down to the working directory in that order, SHALL prefer an override file when present, SHALL load subtree files on first file access in that subtree, and SHALL NOT read above the project root.

#### Scenario: Nested instruction files
- **WHEN** the repository root and a subdirectory both contain an instruction file and the working directory is the subdirectory
- **THEN** the rendered host instructions contain the root file followed by the subdirectory file

#### Scenario: Untrusted workspace
- **WHEN** the workspace is not marked trusted
- **THEN** no project instruction file is read or rendered

### Requirement: Project instructions carry host authority
Project instructions SHALL be rendered as host-authority fragments and SHALL NOT override system or policy fragments.

#### Scenario: Instruction file imitates policy
- **WHEN** an instruction file contains text formatted as a policy directive
- **THEN** it renders inside host markers and the policy fragment is unchanged

### Requirement: World state is rendered in full once and diffed thereafter
The runtime SHALL maintain world-state sections with stable ids for environment, current time, permissions, and active project instructions, SHALL render every section in full on the first turn and after any history rewrite, and SHALL otherwise render only a merge-patch diff of changed sections with replacement or removal text. The current-time section SHALL be compared at a configured granularity (default one minute) from a clock the runtime can substitute in tests, so a turn inside the same granularity bucket counts as unchanged.

#### Scenario: Working directory changes
- **WHEN** the working directory changes between turns
- **THEN** only the environment section is re-sent, stating that it replaces the previous environment

#### Scenario: Nothing changed within the time granularity
- **WHEN** no section changed since the last turn and the current time is in the same granularity bucket as the last render
- **THEN** no world-state fragment is added to history

#### Scenario: Time bucket rolls over
- **WHEN** only the current time has crossed into a new granularity bucket since the last render
- **THEN** only the current-time section is re-sent
