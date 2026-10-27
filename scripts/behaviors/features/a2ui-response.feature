@api @agui @a2ui
Feature: Receive a published A2UI surface through AG-UI
  Scenario: A real agent publishes the requested A2UI greeting
    Given a disposable agent can publish A2UI surfaces
    When I ask for a fresh greeting surface through AG-UI
    Then AG-UI carries the published A2UI surface and completes successfully
