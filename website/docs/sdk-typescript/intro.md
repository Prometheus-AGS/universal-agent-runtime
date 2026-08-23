---
sidebar_position: 1
title: TypeScript SDK
description: Fetch, SSE, and Zod-validated UAR access from the TypeScript SDK source package.
source_records:
  - sdks/typescript/README.md
current_authority: /docs/sdk-typescript/intro
---

# TypeScript SDK

## Boundary statement

**The TypeScript SDK validates the HTTP responses it knows; it does not certify
the server, provider, or npm registry.** Its fetch and SSE paths remain subject
to the deployed UAR authentication, profile, policy, and state boundaries.

## Client surface

`UarClient` supports Node.js 20+, current browsers, serverless runtimes, and
Next.js environments with a compatible `fetch`. Its namespaces cover chat,
tools, embeddings, runs, knowledge bases, and ingestion. JSON responses are
parsed with Zod. Errors preserve the HTTP status and server details when
available.

Streaming methods use `fetch-event-source` and return an `AsyncIterable` of SSE
events. A caller can supply an `AbortSignal` and `lastEventId`. The cursor asks
the server for its retained replay boundary; the SDK does not manufacture
missing history.

## Source checkout

Use the package-local lockfile:

```bash
npm --prefix sdks/typescript ci
npm --prefix sdks/typescript run typecheck
npm --prefix sdks/typescript run build
```

Examples cover chat, streaming, tool calls, structured output, agent runs, and a
Next.js route handler. They require a running server when executed rather than
typechecked.

## Hosted reference

[TypeDoc](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript/)
is generated from `sdks/typescript` and staged into the Pages artifact. Generate
the same source reference locally with:

```bash
npm --prefix sdks/typescript run docs
```

## Registry publication

`package.json` names `@prometheus-ags/universal-agent-runtime-sdk` version
`1.0.0`. Confirm the scoped package, version, integrity, and publisher at npm
before depending on a registry install. TypeDoc staging and local package
metadata do not establish npm availability.

## Profile limits

The SDK targets the HTTP/SSE surface in `minimal` or `server-full`. It does not
embed `embedded-mobile`. Browser and server runtimes also have different
credential-storage and CORS boundaries; do not put long-lived privileged
tokens into browser source.

Next: [Configuration](../configuration.md).
