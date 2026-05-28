# kbd-agent-rules-injector Specification

## Purpose

A `/kbd-inject-agent-rules` skill that idempotently writes a fenced-region block of 8 rules (Karpathy's 4 think-first principles + Boris Cherny's 4 Claude Code workflow principles) into `CLAUDE.md` and/or `AGENTS.md`, refreshable from cached source URLs, with `--dry-run` and `--target` flags. The fenced region is byte-replaceable; everything outside the markers is preserved verbatim.

## Requirements

### Requirement: Skill Surface
The orchestrator SHALL ship a `/kbd-inject-agent-rules` skill at `skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/` with `SKILL.md`, an executable `kbd-inject-agent-rules.sh`, a `references/rules-cache.md` containing the verbatim rule text + source URLs + last-fetched date, and a `references/template.md` defining the fenced-region body.

#### Scenario: Files exist
- **WHEN** the orchestrator skill set is inspected after this change
- **THEN** the four files above MUST exist; the `.sh` MUST be executable.

#### Scenario: Cache file shape
- **WHEN** `references/rules-cache.md` is read
- **THEN** it MUST contain at least one source URL per rule set (Karpathy and Boris Cherny), an ISO-8601 fetch date, and the canonical rule text in numbered list form.

### Requirement: Argument Parsing
The skill SHALL accept `--target`, `--path`, `--refresh`, and `--dry-run` flags with documented defaults.

#### Scenario: Default invocation
- **WHEN** invoked with no flags
- **THEN** the skill MUST act as if invoked with `--target both --path . --no-refresh --no-dry-run`.

#### Scenario: Explicit single target
- **WHEN** invoked with `--target CLAUDE.md`
- **THEN** the skill MUST modify only `CLAUDE.md` at the resolved project root; `AGENTS.md` MUST remain untouched.

#### Scenario: Dry run
- **WHEN** invoked with `--dry-run`
- **THEN** the skill MUST print the diff it WOULD apply to each target file and MUST NOT modify any file.

#### Scenario: Invalid target value
- **WHEN** invoked with `--target` set to something other than `CLAUDE.md`, `AGENTS.md`, or `both`
- **THEN** the skill MUST exit non-zero with a usage error.

### Requirement: Fenced-Region Management
The skill SHALL write a region delimited by `<!-- agent-rules:start v<n> -->` and `<!-- agent-rules:end -->`, where `<n>` is the rules-pack version.

#### Scenario: First write
- **WHEN** the target file does not contain the start marker
- **THEN** the skill MUST append the fenced region to the end of the file, preceded by one blank line to separate it from prior content.

#### Scenario: Subsequent write replaces in place
- **WHEN** the target file already contains a valid start/end marker pair
- **THEN** the skill MUST replace the region between the markers (markers included) with the freshly-generated content; content before the start marker and after the end marker MUST remain byte-identical.

#### Scenario: Missing end marker
- **WHEN** the target file contains a start marker but no end marker (corrupt state)
- **THEN** the skill MUST exit non-zero without modifying the file, naming the offending file and instructing the operator to repair the markers manually.

#### Scenario: Multiple start markers
- **WHEN** the target file contains more than one `<!-- agent-rules:start … -->` marker
- **THEN** the skill MUST exit non-zero without modifying the file, naming the duplication.

### Requirement: Rules Pack Content
The fenced-region body SHALL contain both rule sets in a documented order with explicit attribution.

#### Scenario: Karpathy rules present
- **WHEN** the fenced region is generated
- **THEN** it MUST contain a `### Think-first principles (Karpathy)` heading followed by four numbered items naming exactly: "Think Before Coding", "Simplicity First", "Surgical Changes", "Goal-Driven Execution".

#### Scenario: Boris Cherny principles present
- **WHEN** the fenced region is generated
- **THEN** it MUST contain a `### Workflow principles (Claude Code, Boris Cherny)` heading followed by four numbered items naming exactly: "Plan Mode First", "CLAUDE.md as accumulated knowledge", "Verification + feedback loops", "Code quality matters for AI too".

#### Scenario: Source attribution
- **WHEN** the fenced region is generated
- **THEN** the closing lines MUST include a sentence pointing to `shared/references/rules-cache.md` for verbatim sources and the cached fetch date.

### Requirement: Atomic Writes
The skill SHALL write every file via temp-file + `mv` so an interrupted invocation leaves the original file intact.

#### Scenario: Interruption safety
- **WHEN** the skill is interrupted mid-write
- **THEN** the original target file MUST remain unchanged from its pre-invocation state.

### Requirement: Refresh Flow
With `--refresh`, the skill SHALL re-fetch the cached source URLs, validate that the rule keywords are still present, and update the fetch date.

#### Scenario: Successful refresh
- **WHEN** `--refresh` is supplied and every source URL responds 2xx within 10 s AND the response body contains the documented anchor keywords for that source
- **THEN** the skill MUST update the "Last fetched" date in `rules-cache.md` and proceed to rewrite the fenced regions.

#### Scenario: Source URL unreachable
- **WHEN** `--refresh` is supplied and any source URL fails (4xx/5xx/timeout)
- **THEN** the skill MUST emit a stderr warning naming the failed URL, MUST NOT update the fetch date for that source, and MUST still rewrite the fenced regions using the existing cached content.

#### Scenario: Anchor keyword missing
- **WHEN** a refreshed source body no longer contains its documented anchor keyword (indicating the page changed substantially)
- **THEN** the skill MUST emit a stderr warning, MUST NOT auto-update the rule text from the changed page, and MUST suggest a manual review of `rules-cache.md`.

### Requirement: Idempotency
Running the skill twice with the same inputs MUST produce identical target files.

#### Scenario: Bit-identical second run
- **WHEN** the skill runs successfully against a target file, then runs again with identical flags and no upstream changes
- **THEN** the second run MUST leave the target file byte-identical to its state after the first run.

### Requirement: Orchestrator Documentation
The new skill SHALL be listed in `kbd-process-orchestrator/SKILL.md` "Quick Start Commands" / per-skill list.

#### Scenario: Listed
- **WHEN** the orchestrator `SKILL.md` is read after this change
- **THEN** it MUST list `/kbd-inject-agent-rules` alongside other documented skills.
