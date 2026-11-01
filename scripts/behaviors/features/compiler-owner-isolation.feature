Feature: Compiler specs and authoring sessions belong to one verified owner (#387)

  Compiler specs, reports and authoring sessions are stored per verified owner.
  The local owner is the loopback caller on an installation with the opt-in
  security.local_loopback_owner. The proxied owner reaches the same installation
  through the external identity proxy and is a different principal. Neither
  owner can list, read, compile, change or remove the other's records, and a
  record that belongs to someone else answers exactly as an absent one does.
  Without the opt-in, a loopback caller without credentials has no verified owner
  and creates nothing.

  Scenario Outline: A second owner cannot reach the first owner's compiler records on <backend>
    Given a dedicated <backend> UAR with local_loopback_owner on
    When the local owner creates a compiler spec and an authoring session
    And the proxied owner creates its own compiler spec and authoring session
    Then the local owner lists and reads its own spec and session but not the proxied owner's
    And the proxied owner lists and reads its own spec and session but not the local owner's
    And each owner gets not found when it compiles, deletes or cancels the other owner's records
    But the local owner still reads its spec and its session is still active

    Examples:
      | backend  |
      | postgres |
      | surreal  |

  Scenario: Without the opt-in, a loopback caller without credentials cannot create compiler records
    Given a dedicated postgres UAR with local_loopback_owner off
    Then creating a compiler spec or an authoring session on the loopback address without credentials is refused
