---
sidebar_position: 4
title: Corrections and Reversals
description: Preserve the decisions UAR changed after new evidence or a changed product boundary.
source_records:
  - docs/adr/0002-dual-license-agpl-mit.md
  - docs/adr/0017-relicense-runtime-to-mit.md
  - docs/adr/ADR-007-react-first-frontend.md
  - .prometheus/decisions.md
current_authority: /docs/history/corrections
---

# Corrections and reversals

The project changed these positions explicitly. They are not embarrassing
footnotes to erase; they are the points where evidence or product direction made
the prior answer wrong.

| Earlier position | Replacement | Why it changed | Uncomfortable consequence |
|---|---|---|---|
| Keep the runtime under **AGPL** plus a commercial exception | ADR-0017 licenses repository code under MIT | Peer-to-peer and individually operated nodes made the copyleft boundary an adoption tax | Competitors may host forks; commercial durability must come from service value |
| Treat **HTMX** and Web Components as the primary product UI | ADR-007 makes React 19 the canonical first-party UI | The shipped UI and desktop shell were already React, so old prose contradicted the product | Historical designs remain in the repo and can still mislead if their banners are ignored |
| Allow the stale **purple** Material-style guide and competing terminal theme to read as authority | The frontend authority selects the current Flat 2.0 token system and records scoped divergences | Three incompatible “authoritative” systems produced measurable implementation debt | The migration disclosed hundreds of legacy exceptions rather than claiming instant compliance |
| Let missing generated API references fall back to **placeholder** Pages output | One Docusaurus artifact must contain real Rust and TypeScript references or fail | A successful deployment could otherwise publish the wrong site or invented reference output | Documentation publication now fails when generators or staging are unavailable |
| Continue the interrupted **AWS-LC** JWT spike | Pin `jsonwebtoken` 11 to RustCrypto and let UAR own first installation | RustCrypto fit the server/iOS/Android dependency graph; upstream provider identity was not observable | Any provider installed before UAR—including RustCrypto—must be treated as a conflict |
| Run routine checks in **GitHub Actions** | Run development verification locally; Actions are deployment-only | Remote queues and persistently red workflows obscured rather than improved development evidence | Local evidence must be retained deliberately; a green deployment workflow is not a product test verdict |
| Count a three-hour **synthetic soak** as inference/resilience evidence | Require bounded functional requests through UAR to a genuine loaded model | Thousands of deterministic proxy-double requests never crossed the model boundary | Real inference costs money and is nondeterministic; unavailable prerequisites leave the claim unverified |

## What a correction proves

A correction proves that the project changed its stated position and retained the
reason. It does not retroactively invalidate every artifact created under the old
position, and it does not certify the replacement across every profile. Current
behavior still requires current source and requirement evidence.

## Why retain the old record

Deleting old guidance would make later constraints look arbitrary. Retention
shows, for example, why first-owner JWT initialization exists, why Pages refuses
placeholder output, and why a long-running provider double is not accepted as
model inference. The cost is ongoing maintenance: historical material must keep
its supersession banner and current-authority link.
