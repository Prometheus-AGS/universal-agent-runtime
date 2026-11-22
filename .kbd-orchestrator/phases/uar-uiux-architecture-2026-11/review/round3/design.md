# Round 3 review A (design): FIX FIRST, recapture needed

Patch d1c8062c… reviewed by source only, with no rendered build.

| ID | Severity | Finding | Reviewer's fix | Triage |
|---|---|---|---|---|
| D1 | P1 | Tab from a focused row scrolled out of view loses focus | Clear the pin only when focus leaves the list | FIX by deleting `onBlur` (a11y F1): that also covers the command-palette case F3, which this variant would not |
| D2 | P2 | After a width change, scrolling up jumps by the re-wrap difference (cached heights) | Override `shouldAdjustScrollPositionOnItemSizeChange` | DEFER: unobserved, and it overrides a library default; check at the rendered pass |
| D3 | P2 | Row estimate 64 px where a normal row computes to 62, so every mount corrects the scroll position | `ROW_ESTIMATE_PX = 62` | FIX |

Recapture at the boundary or before release: /admin/models at 1440, 1024, 901 and 375; mid-list;
compare bar; filtered empty; last row.
