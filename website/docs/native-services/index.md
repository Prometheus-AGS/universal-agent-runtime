---
sidebar_position: 1
title: Native services
description: Install UAR 1.0 as a supervised local service on macOS, Linux, or Windows.
source_records:
  - packaging/native
  - docs/DEPLOYMENT.md
current_authority: /docs/native-services
---

# Native services

The native package installs the `server-full` release and React application as
one supervised process. HTTP `1906` and A2A gRPC `50051` are loopback-only by
default. Provider credentials are copied once from an operator shell into a
least-privilege service environment; the supervisor never sources a full login
profile.

Choose [macOS](./macos.md), [Linux](./linux.md), or [Windows](./windows.md).
Every installer preserves existing configuration and mutable state, backs up
configuration before an additive merge, and keeps operator logs under the
platform `.prometheus` path.

```mermaid
flowchart LR
    Shell[Operator shell] --> Generator[Allowlist generator]
    Generator --> Env[Service environment]
    YAML[Preserved YAML] --> Service[UAR server-full]
    Env --> Service
    Service --> HTTP[HTTP 127.0.0.1:1906]
    Service --> GRPC[A2A gRPC 127.0.0.1:50051]
    Service --> Logs[.prometheus logs]
```

A health probe proves process health, not inference. A provider is operational
only after a genuine request crosses the installed UAR boundary and returns a
model-produced response.
