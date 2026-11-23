---
type: SessionRecord
id: karpathy-session-0f705706743ece7d
title: Karpathy session 0f705706743e
tags:
- karpathy
- session-learning
sources:
- resource: session:f1444227-3c6c-46fa-b137-2dd9075d415f
generated:
  by: pk/1.11.0
  at: 2026-11-22T22:53:33.165171+00:00
created_at: 2026-11-22T22:53:33.165171+00:00
updated_at: 2026-11-22T22:53:33.165171+00:00
revision: 0
---

## Delta

Agreed. I haven't started the run. Here's the proposal.

**1. Finish the code first.** These are the known user-facing defects still open after round 4:
- **D1:** deleting a thread removes it only in the browser. The server keeps it, so on postgres or remote stores it can come back.
- **P2:** on those stores, a renamed thread goes back to "New conversation".
- **P3:** only the first thread in each page load gets an automatic title.
- A failed rename or delete closes the dialog without telling the user. Save and Delete also stay clickable while saving.
- A second send is still refused while another thread's reply is streaming, or during an A2UI action.

**2. Test only what users do, once each, in a real browser.** No mutant rebuilds, no old-baseline runs, no triple repeats. About an hour, plus one build:
1. Change the system default model, and the next reply uses it.
2. Agents without their own model follow the default; agents with one keep it.
3. A conversation's model choice wins, and clearing it goes back to the default.
4. Disabled providers and refused models can't be picked or used.
5. Each reply shows which model answered it.
6. The model picker is usable with the full catalog.
7. Renaming and deleting a thread work and survive a reload.
8. Sending again right after a reply works.

**3. Then build the image once and smoke-test it.**

**What we give up:** we lose the proof that each test would catch a broken build. Mutants and baseline runs gave us that, and they found real test bugs this round. A test that passes for the wrong reason would now go unnoticed. Cutting them also needs one more signed amendment.

The older known limits stay out of the code list unless you add them. Examples: saved rate-limit settings don't affect the limiter, and store-less servers can still persist keys from the environment.

Do you approve both the code list and the eight tests, or should I add the older limits to the code list too?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: f1444227-3c6c-46fa-b137-2dd9075d415f
- Captured: 2026-11-22T22:53:25.145552Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
