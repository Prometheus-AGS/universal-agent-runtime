## ADDED Requirements

### Requirement: A fallback request fits the fallback's context window
On failover, UAR SHALL resolve each fallback model's context window for the run and fit the request to it before the fallback request is prepared. The input allowance SHALL be the window minus the output reservation and tool-definition tokens. Optional history SHALL be displaced by the agent's declared context strategy, keeping the system message and tool groups intact. The attempt manifest SHALL report the fit as `context_fit`.

#### Scenario: A smaller fallback receives a displaced request
- **WHEN** the primary is unavailable, the fallback's window is 4096 tokens, the conversation holds history larger than that window and the agent keeps the first and latest messages
- **THEN** the fallback answers, its recorded request keeps the first message, the latest message and the question, a middle message is removed, and `context_fit.displaced` is true.

### Requirement: A fallback request that cannot fit is refused before send
A fallback request that still exceeds its window after displacement SHALL be refused locally without counting against provider health, and the next fallback SHALL be tried. When no fallback fits, the run SHALL end with error code `context_overflow` and no fallback request SHALL be sent.

#### Scenario: No fallback fits
- **WHEN** the same run uses context strategy `none`
- **THEN** the run ends with `context_overflow` and the fallback receives no request.
