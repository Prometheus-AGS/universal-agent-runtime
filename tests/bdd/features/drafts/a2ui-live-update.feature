Feature: A2UI surface updates fan out to live clients and survive late join
  As a user with an agent-driven UI surface open in more than one place
  I want every connected client to see the same surface updates
  So that a UI surface reflects reality no matter when or where I'm watching it

  Scenario: Multi-client convergence
    Given a fresh run with two connected SSE clients
    When an A2UI surface update is triggered for that run
    Then both connected clients receive the identical state-patch event

  @pending-flint-realtime-fabric
  Scenario: Late-join reattach
    Given a fresh run with an A2UI surface update already published before any client connects
    When a new client connects to that run after the update was published
    Then the newly connected client can replay the surface's full update history
