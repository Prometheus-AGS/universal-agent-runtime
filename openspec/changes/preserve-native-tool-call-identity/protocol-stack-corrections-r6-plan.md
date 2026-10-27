# Complete r6 ordinary request admission correction

KBD remains Execute in the isolated repair child, supporting Kubo Execute/FIR-9. Ordinary verification is CLOSED. The separately approved r5 protocol diagnostic is consumed; this plan grants no execution exception.

## Assessment and analysis

The exact approved r5 first command passed826 library tests with1 existing ignored provisioning case, including all three corrected lifecycle scenarios, and passed all6 admission integration tests. Its protocol target announced11 tests and aborted with a stack overflow in `readonly_mcp_hint_does_not_bypass_ask_approval` before reporting a protocol aggregate. Source/control drift was empty. Commands2/3 never started. Retain the original logs, source inventories and unsuccessful receipts unchanged.

The one-shot diagnostic on that existing protocol executable captured51 crashing-thread frames. The fault maps to `execute_request_inner` entry, followed by polling its existing `Pin<Box<...>>`, the inline admission block in `execute_request_with_run_id`, the ordinary compatibility adapters and the real approval regression. There is no repeated recursive chain. LLDB initially labels the async symbol at `slice.rs:0`; source resolution maps it to manager.rs:3794. The diagnostic measured neither individual frame/future sizes nor a required byte limit. Debugger exit0 is diagnostic completion, not a passing test. The exact owned inferior was confirmed gone, its fresh temporary directory removed and source drift remained empty.

Source inspection establishes that boxing the kernel alone leaves admission's graph/ordinary branch state and its construction composed through the ordinary async wrapper chain. The selected test exercises the ordinary branch with a scripted driver, a synthetic local MCP descriptor and persistence None. It asserts real Ask approval before effects; its backend never invokes an operator store, provider account, remote MCP process or shell. This regression must remain unchanged rather than hiding the failure with test-only boxing or a larger stack.

## Entire implementation batch

The existing specialist owns exactly `src/uar/runtime/manager.rs`. Root owns this plan, proposal/design/tasks, task packet, append-only history and derived inventory. Preserve every earlier integrated edit; no other producer path is assigned.

1. Keep `execute_request_with_run_id`'s existing async signature and compatibility entry points. Replace its inline admission construction with a call to one private synchronous factory returning `futures::future::BoxFuture<'_, String>`.
2. Mark that factory `#[inline(never)]` so construction returns its boxed future before the compatibility wrapper polls it. Move the existing complete admission body, including graph preparation/failure recording and ordinary execution, into that boxed future without changing branch selection or effects.
3. Preserve the current request/run-ID clones and the async wrapper's original parameter lifetime. Preserve the existing inner `Box::pin`, graph-root cancellation token, exact run identity, all error recording, authority, policy, admission, cancellation, budget and receipt behavior. This is one allocation boundary, not an opportunity to refactor adjacent paths.
4. Leave hosted-root and captured-child entry boundaries and all11 protocol tests unchanged. No test runtime/thread flavor, stack size, test-body boxing, new size scan, timeout increase, assertion weakening, dependency, API/schema, catalog, route or vendored change.
5. Finish all required source/documentation, stop the writer, inspect integration as source, then capture the complete expanded manifest/patch/preimages and typed canonical source completion. No testing, builds, format/lint/compiler/diff checks, validators, generators, runtime probes, debugging, publication or Kubo cumulative verification before renewed explicit operator instruction on the final source.

## Limits and falsification

The trace supports separating composed admission construction from later polling; it does not prove an exact frame size, a universal stack bound or that the unchanged inner poll frame cannot itself overflow. The real protocol target must falsify or confirm this correction under its existing assertions after renewed approval. R5's successful library/admission results are historical evidence for r5, not substitutes for verification of changed r6 source. The pre-existing ignored provisioning case remains unverified. Customer build, extended client gate, protected native receipt flow, immutable artifact/pin integration, independent reviews and Kubo cumulative verification remain pending. No PR readiness or publication is claimed.

## Complete source handoff

The single assigned specialist implemented the entire one-file batch and stopped writing. The async wrapper passes its same request/run clones to the private non-inlined synchronous factory. The prior complete admission body is preserved inside its returned box, including graph failure recording and inner boxing. Hosted-root/captured-child entry and all11 protocol tests are unchanged. Root inspected the final source diff as integration, not independent review. No known scoped implementation item or extra source path remains. Complete source/documentation totals30 paths; root records the exact expanded manifest, patch and typed canonical boundary separately. No tests, builds, checks, formatting, probes or diagnostics ran after these edits. All causal and verification limits above remain in force.
