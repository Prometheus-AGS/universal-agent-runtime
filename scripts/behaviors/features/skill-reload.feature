@api @skills @reload
Feature: Standard skill discovery and reload change usable behavior
  Scenario: Discover a skill pack, change one body, reload, and use the new body
    Given a disposable agent and an owned skill pack with top-level and nested skills plus non-skill files
    When I reload skills
    Then exactly the pack's two manifest skills are discovered once and non-skills are excluded
    And a request with the top-level skill attached follows its original private rule
    When I change that skill's body on disk and reload skills
    Then a new request with the skill attached follows the changed rule, not the cached one
