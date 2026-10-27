## ADDED Requirements

### Requirement: Registered-agent selection tolerates inline model reasoning
When the orchestrator classifies a request against its live eligible registered-agent roster, it SHALL exclude inline reasoning before matching: closed `<think>`/`<thinking>` blocks, and an unclosed block through the end of the reply. If the operator-approved normalization is enabled, it SHALL then remove one layer of surrounding whitespace, backticks or quotes and one trailing period. The remaining answer SHALL equal exactly one eligible agent ID; no substring, case-folded, fuzzy or first-candidate fallback is permitted. Errors and logs SHALL NOT include classifier reply text.

#### Scenario: Valid ID after closed reasoning
- **WHEN** the classifier reply is a closed reasoning block followed by an eligible agent ID
- **THEN** that agent is selected, the selection event precedes the answer, and the delegated agent runs

#### Scenario: Classifier reply ends inside reasoning
- **WHEN** the classifier reply contains an unclosed reasoning block or no answer text
- **THEN** routing fails with the error message `graph_routing_classifier_incomplete` and no agent is selected

#### Scenario: Answer is not an eligible ID
- **WHEN** the reply's answer part, after reasoning is excluded and any approved normalization is applied, is not exactly one eligible agent ID
- **THEN** routing fails with the error message `graph_routing_selection_invalid` and no other agent is selected as a fallback

### Requirement: A delegated contribution is answer text only
A delegated child turn's contribution SHALL be its output with inline reasoning excluded. If that answer part is empty and the output contained reasoning, the delegation SHALL fail with the error message `delegated_answer_missing: reasoning_only` rather than continue as a successful contribution.

#### Scenario: Child output is only unclosed reasoning
- **WHEN** a routed child turn streams only an unclosed reasoning block and then completes
- **THEN** the parent run ends with a run error whose message is `delegated_answer_missing: reasoning_only`, not a success outcome

#### Scenario: Child output has closed reasoning and an answer
- **WHEN** a routed child turn streams a closed reasoning block followed by answer text
- **THEN** the delegated contribution streamed and recorded by the parent contains the answer text and no reasoning block
