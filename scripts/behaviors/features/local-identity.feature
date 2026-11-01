Feature: A local caller without credentials has one stable owner (#392)

  With optional JWT on the TrustedLocal profile and the opt-in
  security.local_loopback_owner, a caller on the loopback address that sends no
  credentials is the local-host principal. Its sessions and memory belong to that
  one owner, which is distinct from a proxied owner. The flag is off by default.

  Scenario: Local no-credential chat and memory share one owner
    Given a dedicated UAR with local_loopback_owner on, memory enabled and the stub model
    When I save a memory record on the loopback address without credentials
    And I chat on the loopback address without credentials
    Then the memory record is readable on the loopback address without credentials
    And the session is listed on the loopback address without credentials
    But the proxied owner sees neither the memory record nor the session

  Scenario: Without the opt-in, a local caller without credentials stays anonymous
    Given a dedicated UAR with memory enabled and local_loopback_owner off
    Then saving a memory record on the loopback address without credentials is refused
