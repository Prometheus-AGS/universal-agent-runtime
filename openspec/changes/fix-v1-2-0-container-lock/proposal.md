# Repair v1.2.0 container dependency resolution

## Why

Deployment run 39683357804 failed on both Linux architectures because the
committed Cargo.lock does not satisfy the committed Liter manifest. Liter needs
etcd-client 0.21, jsonschema 0.58 and OpenDAL >=0.59.4; the lock contains
0.20.0, 0.57.0 and 0.59.3 respectively.

## What changes

Reconcile only the incompatible locked dependency families and their transitive
edges. Preserve the pinned submodules, manifests and Dockerfile's --locked build.

## Capabilities

- offline-reproducible-build: require the committed lock to satisfy pinned path
  dependencies for both published Linux architectures.

## Risk

A valid resolution does not prove compilation. Both native deployment builds
and the merged published manifest must succeed before claiming image delivery.
This repair does not certify outstanding lifecycle behavior.
