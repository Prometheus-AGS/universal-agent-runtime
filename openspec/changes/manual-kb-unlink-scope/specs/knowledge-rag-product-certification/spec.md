## ADDED Requirements

### Requirement: Unlinking a knowledge base removes its implicit agent retrieval access
An explicit agent patch to knowledge-base links or enabled state SHALL persist the corresponding selected or none policy; an enabled empty link list SHALL select no KBs. Unedited legacy definitions retain their existing default selection. An explicit KB policy in the same patch SHALL take precedence, and unrelated policy fields and denied IDs SHALL be preserved. Removing a KB link SHALL NOT delete the KB or its documents.

#### Scenario: Remove the last linked KB
- **GIVEN** an agent has used an indexed private fact from its sole linked knowledge base
- **WHEN** only its knowledge-base ID list is patched to empty and a fresh conversation prompts that agent
- **THEN** it answers an independent question without that fact or KB citations
- **AND** the same document remains available through direct KB retrieval
