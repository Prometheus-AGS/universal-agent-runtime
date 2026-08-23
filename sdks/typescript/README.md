# Universal Agent Runtime TypeScript SDK

> **Current authority:** [TypeScript SDK guide](/docs/sdk-typescript/intro). The
> source package is checked in at version 1.0.0; registry availability is release
> evidence and is not inferred from this README.

The TypeScript SDK is a fetch/SSE client for Node.js 20+, current browsers,
serverless runtimes, and compatible Next.js environments. Zod validates JSON
responses. Client namespaces cover chat, tools, embeddings, runs, knowledge
bases, and ingestion.

Use the package-local npm lockfile:

```bash
npm --prefix sdks/typescript ci
npm --prefix sdks/typescript run typecheck
npm --prefix sdks/typescript run build
npm --prefix sdks/typescript run docs
```

Network examples require a running UAR server and valid credentials. Streaming
methods return an `AsyncIterable`, accept cancellation, and may supply a last
event ID to the server's retained replay boundary; the SDK does not manufacture
missing history.

`package.json` names `@prometheus-ags/universal-agent-runtime-sdk`. Before using
`npm install`, verify the scoped package, exact version, integrity, and publisher
on npm. Local TypeDoc output and package metadata are not registry evidence.

The SDK targets HTTP/SSE server profiles and does not embed
`embedded-mobile`. Browser credential storage and CORS also remain deployment
responsibilities.
