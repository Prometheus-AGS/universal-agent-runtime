## ADDED Requirements
### Requirement: Exact GPT-6.1 Sol primary-source catalog entry
The committed catalog SHALL contain openai/gpt-6.1-sol with exact primary-source Standard short-context USD per million token costs input2, output10, cache_read0.10 and cache_write2.50. The maintainer generator SHALL reproduce the addition from checked-in source metadata without Cargo or network access. It SHALL preserve an existing upstream exact identity when supplied, SHALL NOT alias another model or bypass budget checks, and SHALL document long-context/tier limitations.

#### Scenario: Pinned schema lacks selected exact model
- **WHEN** pinned Liter provider/model schemas lack gpt-6.1-sol
- **THEN** generation adds only the documented official entry to provider openai and records primary-source provenance

#### Scenario: Upstream exact identity exists
- **WHEN** a later pinned schema provides openai/gpt-6.1-sol
- **THEN** the generator retains that exact upstream entry instead of overwriting it with the supplement
