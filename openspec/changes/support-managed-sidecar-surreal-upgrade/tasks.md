## 1. Offline migration

- [x] 1.1 Add a native, non-truncating lock command with a host-owned input lifetime.
- [x] 1.2 Add an isolated SurrealKV copy conversion and ordinary reopen check.
- [ ] 1.3 Build the updated sidecar and verify the normal app startup path against a separate copy of a legacy profile.
- [ ] 1.4 Verify managed Windows package behavior, including promotion with the native lock held.

## 2. Desktop release

- [x] 2.1 Carry the Windows service call forward to the current server signature so native RC4 builds share one source.
- [ ] 2.2 Publish the three desktop sidecar payloads from the same source revision.
- [x] 2.3 Rename the historical phase alias that prevented Windows checkout, preserving its target and the archived evidence.
- [x] 2.4 Use the bundled LLVM Mach-O linker on macOS and gate release packaging on warning-free compiler output.
- [x] 2.5 Fix the MSVC C++ standard option and remove the superseded Windows-only server wrapper after observing warnings in the native release build.
