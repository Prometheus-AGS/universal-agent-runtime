# Tasks

## 1. Exact authority binding

- [x] 1.1 Add canonical resource, payload, policy, grant, lease, and budget bindings to prepared invocations.
- [x] 1.2 Persist only redacted lifecycle evidence while retaining exact bindings in memory and paired-host requests.

## 2. Claim-time enforcement

- [x] 2.1 Add paired-host and standalone claim revalidation to the existing admission lifecycle.
- [x] 2.2 Re-evaluate current local policy and runtime posture immediately before claim.
- [x] 2.3 Preserve claim-before-dispatch and terminal/uncertain lifecycle ownership in UAR.

## 3. Entry-point convergence

- [x] 3.1 Route authenticated direct REST tool execution and authenticated actor delegation through descriptor validation and ToolAdmissionRuntime.
- [x] 3.2 Require explicit admission/policy injection for embedded tool-capable execution and reject delegated standalone bypass.

## 4. Governed startup

- [x] 4.1 Reject missing, unreadable, empty, or invalid policy in governed server profiles.
- [x] 4.2 Preserve only the runtime-proven constrained local inactive posture.

## 5. Phase gate

- [x] 5.1 At the completed AFC C02 boundary, exercise direct, managed, embedded, and actor paths with forged identity, policy load failure, revoked approval, and changed payload. No partial test gate is run during implementation.
