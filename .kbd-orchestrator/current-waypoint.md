# Current Waypoint

**Phase**: `perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion`
**Stage**: `plan_complete`
**Updated**: 2026-07-13T19:40:00Z
**Previous phase**: `uar-final-production-hardening-2026-07`

## Summary

Assessment found UAR not customer-ready (6 CRITICAL / 7 HIGH / 8 MEDIUM). Operator
decision D1: 1.0 is multi-tenant — user-isolation fixes come first. Plan complete:
14 ordered changes (10 new, openspec-validated; 4 carried certification changes that
must be rerun after source changes land).

## Next action

```
/kbd-execute
```

First change: `fix-user-isolation-sessions-memory-kb` (C1 threads, C2 memory IDOR,
C3 global KBs).

## References

- [plan.md](phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md)
- [assessment.md](phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md)
- [decision-log.md](phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/decision-log.md)
