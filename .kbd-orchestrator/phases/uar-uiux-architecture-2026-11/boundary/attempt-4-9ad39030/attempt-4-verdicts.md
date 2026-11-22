# Boundary attempt 4: final record (2026-11-22 12:30 CST)

Candidate 9ad39030d67b9fa52528107ab8dcfceb8cbf1f9c. Contract 45f441e0 to command 22, 215f15bd from 24.
Ended by the operator's handoff instruction during command 38, which was interrupted and has no record.
Commands 38-49 have no result. The final rebind (49) did not run, so each PASS below is provisional.

| Scenario | Verdict | Evidence and cause |
|---|---|---|
| SCN-SYS-001 | PASS (provisional) | 23: 3/3; control 24: 9/9 |
| SCN-SYS-007 | PASS (provisional) | 27 and control 28 exit 0 |
| SCN-SYS-003 | FAIL | 25: 0/3, harness reply counter (innerText); control 26 exit 0 |
| SCN-SYS-011 | FAIL | 29, 30: thread Rename is dead (product) |
| SCN-SYS-012 | FAIL | 31, 32: same |
| SCN-SYS-013 | FAIL | 33, 34: same; a further harness fault is predicted behind it (H1) |
| SCN-SYS-014 | FAIL | 35, 36: same |
| SCN-SYS-015 | FAIL | 37: same; mutant control 38 interrupted |
| SCN-SYS-016 | NOT_EXERCISED | 39, 40 not run |
| SCN-SYS-017 | NOT_EXERCISED | 41, 42 not run |
| SCN-SET-003 | NOT_EXERCISED | 43, 44 not run |
| SCN-IDN-001 | NOT_EXERCISED | 45, 46 not run |
| SCN-PICK-002 | NOT_EXERCISED | 47, 48 not run |

Totals: 2 PASS (provisional), 6 FAIL, 5 NOT_EXERCISED. Any repair changes frontend/src and tests/bdd, which
invalidates all of it, so there is no value in resuming this attempt.
