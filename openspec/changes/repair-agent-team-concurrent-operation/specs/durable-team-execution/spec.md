# Spec Delta

## ADDED Requirements

### Requirement: Independent specialist work overlaps before dependent integration

UAR SHALL execute eligible independent tasks assigned to distinct team members concurrently within configured capacity, and supply both actual completed outputs to the dependent writer task. It SHALL preserve honest terminal results and accounting without replaying work on an ambiguous persistence result.

#### Scenario: TEAM-01 mathematics and editorial work merge
- **WHEN** a team containing the saved math and writing agents receives independent calculation and outline tasks followed by a dependent article
- **THEN** distinct real member runs have a proven overlapping active interval and successful individual results
- **AND** the math result matches an independent calculation
- **AND** the final writer consumes both actual output artifacts and produces an article containing the result, editorial headline and saved byline
- **AND** each response is capped at64,000tokens through the outgoing provider request, with a separate host input allowance; durable accounting reserves the explicit team allocation rather than the model catalog maximum, retains actual usage and unknown liability, and does not claim exact input counting for a settings-only endpoint
- **AND** acceptance evaluates actual final answers outside reasoning blocks, permits a single unambiguous fenced JSON math answer with surrounding explanatory prose, and rejects reasoning-only editorial output; the fixture model maximum matches the authorized 64,000-token response ceiling
