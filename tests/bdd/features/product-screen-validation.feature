@ui @product-screen
Feature: Every shipped product screen performs its primary function
  As a release reviewer
  I want one recorded browser scenario per shipped screen
  So that route presence cannot be mistaken for working product behavior

  Background:
    Given a verified browser subject named "screen-validator"

  Scenario: Chat returns and persists a deterministic answer
    When I exercise the primary function of "/threads"
    Then the "/threads" screen validation is visibly complete

  Scenario: About reports product and runtime identity
    When I exercise the primary function of "/about"
    Then the "/about" screen validation is visibly complete

  Scenario: Runtime cockpit consumes a live event
    When I exercise the primary function of "/admin/runtime"
    Then the "/admin/runtime" screen validation is visibly complete

  Scenario: Runs opens replayed trace detail
    When I exercise the primary function of "/admin/runs"
    Then the "/admin/runs" screen validation is visibly complete

  Scenario: Approvals denies a pending tool call
    When I exercise the primary function of "/admin/approvals"
    Then the "/admin/approvals" screen validation is visibly complete

  Scenario: Protocols shows live event families
    When I exercise the primary function of "/admin/protocols"
    Then the "/admin/protocols" screen validation is visibly complete

  Scenario: Providers exposes the configured stub provider
    When I exercise the primary function of "/admin/providers"
    Then the "/admin/providers" screen validation is visibly complete

  Scenario: Credentials stores and removes a user secret
    When I exercise the primary function of "/admin/credentials"
    Then the "/admin/credentials" screen validation is visibly complete

  Scenario: Models filters the live catalog
    When I exercise the primary function of "/admin/models"
    Then the "/admin/models" screen validation is visibly complete

  Scenario: Skills completes the lifecycle
    When I exercise the primary function of "/admin/skills"
    Then the "/admin/skills" screen validation is visibly complete

  Scenario: Agents creates a selectable agent
    When I exercise the primary function of "/admin/agents"
    Then the "/admin/agents" screen validation is visibly complete

  Scenario: Tools finds the native echo tool
    When I exercise the primary function of "/admin/tools"
    Then the "/admin/tools" screen validation is visibly complete

  Scenario: Auth mints and revokes an API key
    When I exercise the primary function of "/admin/auth"
    Then the "/admin/auth" screen validation is visibly complete

  Scenario: Knowledge indexes and searches a document
    When I exercise the primary function of "/admin/knowledge"
    Then the "/admin/knowledge" screen validation is visibly complete

  Scenario: Memory filters verified-user state
    When I exercise the primary function of "/admin/memory"
    Then the "/admin/memory" screen validation is visibly complete

  Scenario: Compiler creates a session
    When I exercise the primary function of "/admin/compiler"
    Then the "/admin/compiler" screen validation is visibly complete

  Scenario: Settings saves and restores a value
    When I exercise the primary function of "/admin/settings"
    Then the "/admin/settings" screen validation is visibly complete

  Scenario: A2UI testing previews an artifact surface
    When I exercise the primary function of "/admin/a2ui-testing"
    Then the "/admin/a2ui-testing" screen validation is visibly complete

  Scenario: MCP health refreshes server status
    When I exercise the primary function of "/admin/mcp-health"
    Then the "/admin/mcp-health" screen validation is visibly complete

  Scenario: Cost displays known usage
    When I exercise the primary function of "/admin/cost"
    Then the "/admin/cost" screen validation is visibly complete
