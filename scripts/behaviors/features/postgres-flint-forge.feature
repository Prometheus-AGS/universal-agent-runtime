@postgres @flint-forge
Feature: UAR on the flint-forge PostgreSQL 18 image
  UAR keeps its data in its own `uar` database on a flint-forge PostgreSQL 18
  server and behaves the same across restarts as it does on SurrealDB.

  Scenario: UAR starts on PostgreSQL 18 and keeps its data across a restart
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image
    Then readiness reports the postgres persistence backend as ok
    When I save a skill and an agent
    And UAR restarts on the same database
    Then the skill and the agent are still there

  Scenario: A config-driven setting follows the config file across restarts (#389)
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image with shutdown timeout 31
    When UAR restarts with shutdown timeout 32
    And UAR restarts with shutdown timeout 33
    Then the shutdown timeout setting is 33 from the config file
    When I override the shutdown timeout to 45 through the API
    And UAR restarts with shutdown timeout 34
    Then the shutdown timeout setting stays 45 and is reported as drift
    When I reset the shutdown timeout setting
    And UAR restarts with shutdown timeout 34
    Then the shutdown timeout setting is 34 from the config file

  Scenario: Persisted sessions expire after their TTL (#390)
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image with a 20 second session TTL and the stub model
    When I chat in a new session
    Then the session is listed for its owner
    And the session disappears within 150 seconds
    And it stays gone after UAR restarts

  Scenario: Memory is kept by the memory service, not discarded (#393)
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image with memory enabled
    When I save a memory record
    And UAR restarts on the same database
    Then the memory record is still readable
    And the capability matrix names the memory store

  Scenario: Skills work with a 1024-dimension embedder (#395)
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image with a 1024-dimension embedder
    When I save a skill
    Then skill matching finds it
    When UAR restarts on the same database
    Then skill matching still finds it

  Scenario: Unsupported durable features are reported and refused (#394)
    Given a dedicated UAR on the flint-forge PostgreSQL 18 image
    Then the capability matrix reports the team durability features as unsupported
    And a follow-up request is refused with capability_unsupported
