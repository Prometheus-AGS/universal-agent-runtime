# Next-Generation Plugin System for the Universal Agent Runtime (UAR): Research, Architecture, Specification & Implementation Plan

| Field | Value |
|---|---|
| Doc ID | PAGS-RESEARCH-UAR-PLUGIN-001 |
| Date | 2026-08-04 |
| Author | Travis James, Prometheus AGS (research compiled via PMPO deep-research run) |
| Status | Accepted as design input for the next UAR development cycle |
| Ownership | Contract layer: **UAR** (`uar:plugin`). Product layer: **KnowMe** (`knowme:*` capabilities, marketplace, entitlements, trust UX). Per ADR-pending ownership split. |
| Canonical copies | `universal-agent-runtime/docs/plugins/` (authoritative) · `know-me-system/docs/strategies/plugins/` (reference copy) |
| Supersedes/amends | `universal-plugin-system-plan.md` D1 target naming (`knowme:plugin@1.0.0` → `uar:plugin@0.2.0`), performance claims, and the 12 review holes' remediations |

---

## TL;DR

- **The librefang-seeded, WIT-native, capability-card design is sound and industry-validated** — VS Code's contribution-points + activation-events, Shopify/Stripe's host-rendered UI, wasmCloud's WIT capability wiring, and proxy-wasm's lifecycle contract each independently prove a piece of it. Stay WIT-native (not Extism), keep A2UI-only (validated by Shopify remote-ui, Stripe UI toolkit, and MCP Apps), and adopt capability-cards as a superset of VS Code contribution points — but bound your capability-wiring ambition against OSGi's complexity collapse and your sandboxing cleverness against Figma's Realms failure.
- **The "abstraction costs nothing" claim is false and must be reframed honestly.** Per the PLDI 2017 WebAssembly paper, only a minority of benchmarks (7 of the PolyBench-C suite) run within 10% of native and "nearly all within 2× of native"; the USENIX ATC 2019 "Not So Fast" study measured real applications at 45% (Firefox) to 55% (Chrome) slower on average, peaking at 2.08×–2.5×. Pulley (mandatory on iOS) is roughly an order of magnitude slower than Cranelift by Wasmtime's own characterization. The correct claim is "one portable contract, three honestly-budgeted execution tiers" — not "costs nothing."
- **The 12 known holes are real and each maps to a proven industry fix**; the highest-priority gaps are revocation/kill-switch (Chrome/Firefox blocklist ops), permission-combination review (Chrome Web Store), and rollback (content-addressed generations). Version the contract as **0.2.0 now, 1.0.0 only after real plugins shake out the surface** — librefang's own humility rule, corroborated by wasmCloud's and MCP's staged extension approach.

---

## PART 1 — THEORY & ANALYSIS

### Requirement 1: Visual AND non-visual plugins of any type (mini-app → single-job trigger component)

**Systems that prove it viable.** Fermyon Spin proves a single manifest (`spin.toml`) can drive many trigger types — HTTP, command (run-to-completion, exports `wasi:cli` `run`), MQTT, cron — where each component is dispatched by a declared trigger, and the same runtime spans a full HTTP app down to a one-shot job. This is exactly UAR's span from `app-shell` to event-fired `workflow` component. VS Code proves the same span in a UI-rich host: a single `package.json` declares everything from a full webview app to a single command. Shopify's model proves the non-visual/visual split explicitly — Shopify **Functions** (WASM, non-visual commerce logic with instruction limits) versus **UI extensions** (declarative components) share one extension/app envelope.

**Failure modes observed.** Spin's weakness is that trigger types are host-defined and adding a new trigger (e.g., the command trigger) required a Spin plugin and a manifest-version bump (v2→v3 was gated on a manifest change). Lesson: UAR's "worlds per plugin kind" must be an **open, additive set** anchored in the capability card, not a closed host enum, so a new kind doesn't force a contract-breaking manifest revision.

**Design lesson imported.** Keep the librefang `run() -> result` world as the atomic unit (proven, has implementations), and treat every richer kind (`hook`, `contributor`, `agent`, `provider`, `app-shell`, `service`, `workflow`) as a WIT world that is a **superset** of the base host imports. The capability card — not a hard-coded host switch — declares which world a plugin implements. This is the Spin lesson (manifest declares the trigger) fused with the VS Code lesson (manifest declares contributions) and hardened against Spin's closed-enum weakness.

### Requirement 2: librefang spec + open-ended capability determination (capability-card open vocabulary)

**VS Code contribution-points vs. capability-cards — validate/refine.** VS Code's `contributes` block plus `activationEvents` is *the* success story of declarative capability advertisement: the host reads the manifest at startup and "prepares its UI accordingly," activating extension code lazily only when an activation event fires (`onCommand`, `onLanguage`, `onCustomEditor`, etc.). The refinement UAR must make: VS Code contribution points are a **closed vocabulary** owned by Microsoft — third parties cannot invent new contribution categories that other extensions can discover. UAR's capability card's **open vocabulary** (unrecognized capabilities carried, sandboxed, semantically discoverable; recognized ones get structured pipeline stages and first-class surfaces) is strictly more powerful and is the correct generalization. **Verdict: copy contribution-points' declarative-manifest + lazy-activation discipline; refine by making the vocabulary open and self-describing (embedded name/description/tags for semantic routing), which VS Code cannot do.**

**The caution: OSGi's complexity collapse.** OSGi is the deepest prior art for a capability model (`Require-Capability`/`Provide-Capability`, versioned service registry, full bundle lifecycle). It technically worked and most enterprise Java middleware was rebuilt on it — yet it failed to "cross the chasm" for application developers. The consensus post-mortem (InfoQ interviews with practitioners): it was built for middleware vendors (its roots are remote set-top-box updates — "OSGi was never built for application developer consumption"), its learning curve was steep, and its `requires/provides` resolution created brittle, hard-to-debug runtime failures; it was ultimately incompatible with the mainstream Spring Boot stack. **Design lesson: UAR's `requires`/`enhances` composition fields are valuable but must stay declarative-and-simple. Do not build an OSGi-style transitive constraint solver. Resolve composition at install/activation with a flat, explain-able "will this run here" check (the host-profile mechanism), and fail loudly with a named error — never a silent version-resolution cascade.** OSGi is the cautionary upper bound on capability-wiring ambition.

### Requirement 3: Lifecycle with hooks (install→verify→register→activate→deactivate→upgrade→remove + runtime activation events)

**proxy-wasm lifecycle contract vs. the UAR lifecycle.** The proxy-wasm ABI (v0.2.1, the de-facto standard across Envoy, Kong/ngx_wasm, and others) is the cleanest proven lifecycle contract for WASM plugins: a **root context** (`on_vm_start`, `on_configure`, `on_tick`) separated from **per-request/per-stream contexts** (`on_request_headers`, `on_response_body`, `on_done`, `on_delete`), where returning `false` from a configuration callback cleanly signals "this instance shouldn't be used." This root-vs-transient split is exactly the discipline UAR needs: a plugin-lifetime context (install/verify/register/activate/deactivate) distinct from per-invocation contexts (activation events, hook firings). **Verdict: adopt proxy-wasm's root/per-event context separation and its "return false to reject configuration" pattern for the `verify`→`activate` gates. UAR's advantage over proxy-wasm is that the Component Model gives typed WIT interfaces instead of proxy-wasm's raw `i32`/pointer ABI with manual buffer management.**

**VS Code activation as the model for runtime activation events.** VS Code's evolution is instructive: newer versions (1.74+) *removed* the need to declare `onCommand`/`onLanguage`/`onCustomEditor` activation events explicitly — the host infers them from the contribution declarations. **Design lesson: derive activation triggers from the capability card's `events.consumes` and `actions` declarations wherever possible, so authors don't double-declare.** Keep an explicit escape hatch for `onStartup`-style eager activation, but default to lazy. (VS Code's disposable extension-host process, with a ~3-second unresponsiveness watchdog that can kill and restart the host without losing work, is also the proof pattern for crash-loop containment below.)

### Requirement 4: Rich event model integrating with Flint Realtime Fabric (FRF)

**WordPress hooks vs. the typed contributor pipeline.** WordPress's action/filter system is the most successful hook system ever built — an informal community count puts core hooks at ~2,744 (WordPress.org itself says "over 2,000"), and they underpin a 60,000+ plugin ecosystem — and it succeeded precisely because it is **dead simple and untyped**: `add_action`/`add_filter` with a priority integer (default 10, lower runs first). But its untypedness has a documented, real cost: the WordPress Plugin Handbook requires that every filter callback *return a value*, and the single most-cited failure mode is "forgetting to return the value — which makes content disappear entirely"; callbacks must also defensively type-check the value they receive. **This is the single strongest evidence in the entire research for UAR's typed-contributor-pipeline decision.** UAR's contributor stages (ingress→…→egress) get WIT-typed input/output schemas, provenance tags, and budget caps — capturing WordPress's compositional power (many independent contributors at declared priority) while eliminating its class of silent type/return bugs. **Verdict: copy WordPress's ordered-multiple-contributors-at-a-hook model and its priority-integer ordering; reject its untypedness. UAR's "declared priority + stable tiebreak, per-stage aggregate budget, first-party-system-content-first" rule is the correct typed evolution.**

**Event forgery and provenance.** WordPress hooks run in-process with full mutual trust — any plugin can `do_action` any hook name. UAR must not inherit this. The host-stamped provenance + emit-to-own-namespace allowlist + rate caps is the correct fix and has no WordPress precedent; the closest analog is Chrome MV3's move to `declarativeNetRequest` (declared, host-mediated rules) precisely because giving extensions imperative interception power was unauditable.

**FRF seam design.** The event model should present a single in-process bus abstraction to plugins (AG-UI vocabulary as the event schema), with the host — never the plugin — deciding whether an event stays local or is marshalled onto an FRF transport (WebRTC data channel / OFP mesh / Matrix-bridge). This mirrors wasmCloud's **lattice**: components emit/consume over WIT interfaces and are entirely unaware whether the peer is in-process or across a NATS-connected mesh — wasmCloud's docs state "application code can be written without regard to the underlying infrastructure," and its wRPC protocol composes distributed components at runtime over the lattice. **Design lesson: model the FRF seam as a wasmCloud-lattice-style transport adapter behind the event bus port, with `WatchEntityType`/entity-watch RPC as adjacent machinery. Plugins see topics; the host sees transports.** Do not expose transport selection to plugins.

### Requirement 5: Runs on all platforms via WASM, native perf via Cranelift, Pulley where JIT forbidden — quantify honestly

This requirement's framing — "the abstraction costs nothing" — is **not supported by the evidence and must be amended.** The honest performance story, sourced:

- **Cranelift codegen quality.** The oft-cited "~2% slower than V8 TurboFan, ~14% slower than LLVM/WAVM" figure (from the Cranelift README) traces to the PLDI 2017 paper *Bringing the Web Up to Speed with WebAssembly* (Haas, Rossberg, et al.) — but that paper's actual headline is narrower: on the PolyBench-C suite, only **7 benchmarks run within 10% of native and "nearly all within 2× of native."** On the larger, realistic **SPEC CPU** suite, the USENIX ATC 2019 paper *Not So Fast* (Jangda, Powers, Berger, Guha) measured applications "slower by an average of 45% (Firefox) to 55% (Chrome), with peak slowdowns of 2.08× (Firefox) and 2.5× (Chrome)" — i.e., mean ~1.45×–1.55× native. A 2025 arXiv study (WAMI) found Wasmtime AOT ~4.1% slower than LLVM on its benchmarks. **Honest claim: Cranelift is near-native for tight compute kernels and typically ~1.45×–1.6× native for real application workloads — not "costs nothing."**
- **Pulley (mandatory on iOS/iPadOS/tvOS).** Wasmtime's docs state Pulley "is likely not suitable for compute-intensive tasks that must run in as little time as possible" and characterize it as roughly an order-of-magnitude slower than native/Cranelift. (No Wasmtime document states a fixed "10×" multiplier verbatim — treat it as an order-of-magnitude planning estimate, not a quoted constant.) Pulley is not naïve — it reuses Cranelift's optimizing mid-end and emits macro-op super-instructions — but per the Pulley Performance Tracking issue (#10102) it "fundamentally uses two opcodes per memory access: one for the bounds check and one for the actual load/store" (no signal-handler traps), and doesn't hoist memory bounds out of loops. **Honest claim: dynamically-installed plugins on iOS run roughly an order of magnitude slower than desktop Cranelift; budget accordingly and keep hot paths off-device or in bundled AOT native code.**
- **Instantiation latency.** Wasmtime's design puts compilation off the critical path via AOT `.cwasm` (`Component::deserialize`), leaving only instantiation, further accelerated by the **pooling allocator** (pre-allocated memory/table pool; deallocation is "a single `madvise` to reset linear memory") and **copy-on-write memory init**. Fermyon CEO Matt Butcher states production "cold starts under half a millisecond" (vendor claim, GlobeNewswire, Apr 9 2025; The New Stack notes traditional Lambda/Azure cold starts are 200–500 ms for contrast) — not independently benchmarked. No official Bytecode Alliance table of absolute µs figures for cold-JIT vs `.cwasm` vs Pulley exists — the docs describe mechanisms, not constants. **Honest claim: with AOT + pooling, instantiation is sub-millisecond on desktop; treat Fermyon's ~0.5 ms as an achievable target, not a guarantee.**
- **Mobile footprint.** A JS plugin via ComponentizeJS/StarlingMonkey embeds a SpiderMonkey engine of **~8 MB per component** (confirmed by both the Bytecode Alliance ComponentizeJS README and wasmCloud docs as a fixed cost that "does not grow with the size of your application code," but currently **duplicated per JS component** — the Bytecode Alliance plans to share embeddings in future). **Design lesson: on mobile, prefer Rust/TinyGo plugins (no engine embedding); treat JS/Python plugins as a desktop-first convenience and warn on mobile install-size impact.**

**Verdict: reframe the requirement. UAR delivers one portable WIT contract across three honestly-budgeted tiers — Cranelift AOT (desktop, Android where W^X permits), Pulley interpreter (iOS/iPadOS, ~order-of-magnitude penalty), and jco-transpiled core-wasm+JS (browser). The abstraction does not cost nothing; it costs a known, tier-specific, documented amount.**

### Requirement 6: Multi-language authoring (Rust, Go/TinyGo, Python, JS/TS, others)

**Extism vs. raw Component Model — why UAR should stay WIT-native.** Extism is the most mature universal WASM plugin framework (host SDKs in 15+ languages, PDKs, a "bytes-in/bytes-out, bring-your-own-serialization" ABI). Its own maintainers state they have "no concrete plans to make use of the Component Model yet" because their generic ABI already supports every language. Helm's HIP-0026 chose Extism explicitly because the Component Model wasn't yet supported by their Go runtime (Wazero), calling Extism "the most mature and well-supported Wasm plugin system today." **So why should UAR *not* use Extism?** Because Extism's advantage — a low-level untyped byte-pipe — is exactly what UAR does not want. UAR's entire design (typed contributor pipeline, typed action I/O schemas, capability cards, cross-language type safety) depends on **WIT interface types** as the contract. Extism would force UAR to reinvent typing on top of a byte-pipe, exactly the mistake WordPress's untyped hooks demonstrate. The Component Model already gives UAR what it needs across languages: **componentize-py** (Python→component), **ComponentizeJS/StarlingMonkey** (JS/TS→component), native Rust via `wit-bindgen`, TinyGo, and the Bytecode Alliance's new **Endive** (Java Wasm runtime with a Cranelift backend, though it does not yet support WASI P2 or the component model as of May 2026). **Verdict: stay WIT-native. Extism validates the market demand for universal plugins but chose the opposite design axis; UAR's typed-contract requirements make the Component Model the correct base. Cite Extism as the "what we'd build if we didn't need types" baseline.**

**Practical multi-language caveats to fold in:** componentize-py resolves all imports at build time (no runtime dynamic import — imports used at runtime must be resolved at the module top level); JS/Python components carry the ~8 MB engine; jco does not yet support async guest functions cleanly (browser builds use synchronous `XMLHttpRequest` wrappers as workarounds).

### Requirement 7: Web, mobile, desktop targets — the browser-host problem handled honestly

**The hard fact: wasmtime does not run in browsers.** Browsers execute core WASM modules, not Component Model components. The proven path is **jco transpile**, which converts any component into equivalent core-wasm + JS glue that runs in Node or the browser. This is real and in production (jco 1.20 passed all 46 upstream Wasmtime WASI P3 tests as of the May 27 2026 wasmCloud community call), but it means the **browser host is not wasmtime** — it's a different execution engine (V8's WASM + generated glue) with different characteristics (async support incomplete, WASI shims explicitly experimental in the browser). **Design lesson: treat host-parity as a first-class conformance concern.** The same plugin must pass the conformance kit under both the wasmtime host (desktop/mobile) and the jco-transpiled host (browser). Divergences (async support, WASI subsystem coverage) must be enumerated in the host matrix, not discovered at runtime. Notably, jco's transpile output is also the mechanism the Bytecode Alliance is using to build the telemetry case for eventual *native* browser Component Model support ("The Road to Component Model 1.0").

### Discovered additions (things the requirements forgot)

1. **Hot-reload dev loop.** Zed's model is the proof point: extensions compile to `wasm32-wasip2`, run in a Wasmtime sandbox, and a module can be reloaded without restarting the editor with failures staying contained. Spin's `spin watch` (manifest `watch` globs) is the file-watch pattern. UAR needs both: a `uar plugin watch` rebuild loop and host-side hot-swap of a plugin generation without full runtime restart.
2. **Conformance/test kit.** Stripe ships Jest matchers + a remote-render test harness for UI extensions (testing "a remote engine that renders your app, not the DOM directly"); UAR needs the analogous conformance kit (the librefang zero-source-change conformance test is the seed).
3. **Monetization reality.** VS Code deliberately never shipped paid extensions; JetBrains Marketplace and Setapp prove curated paid-extension markets work. **Design lesson: KnowMe's W3C Verifiable Credential entitlements (enforced at activation, not runtime) are the correct monetization primitive — decouple licensing from the runtime, as VS Code's absence and JetBrains' presence jointly show.**
4. **Accessibility & i18n of declarative UI.** Because A2UI is host-rendered, accessibility and localization are the *host's* obligation and opportunity — Shopify explicitly states its host-rendered checkout components are "performant, accessible, and work in all of checkout's supported browsers" precisely because the host renders them natively. This is a hidden benefit of the A2UI-only decision — bake WCAG/i18n into the shared renderers once.
5. **Resource metering/billing.** Shopify Functions enforce hard instruction limits; UAR's `epoch-budget-ms` per capability is the analog. Extend it: per-stage aggregate wall budgets and epoch-based interruption via wasmtime's epoch mechanism.
6. **Sandbox cleverness bound — Figma.** Figma's plugin sandbox journey is the definitive cautionary tale. They first shipped a **Realms shim** sandbox; in October 2019 "several independent vulnerabilities were recently discovered with the Realms shim that could have allowed code inside the sandbox to escape" (disclosed as GitHub Advisory GHSA-7cg8-pq9v-x98q, Critical, fixed in realms-shim v1.2.1). They then switched to **QuickJS/SpiderMonkey compiled to WASM**, because — in Figma's words — "now that we're using a JavaScript VM compiled to WebAssembly, it's not possible to confuse objects from outside with objects from inside because the object representations are too different." A "swappable architecture" let them switch in days. The cost: notoriously "impenetrable" debugging and performance overhead. **Design lesson: UAR's WASM-component sandbox is already the *correct* end state Figma converged on the hard way — do not add clever JS-realm-style tricks on top. The Component Model's shared-nothing isolation is the security boundary; keep it and invest the saved effort in debuggability (source maps, structured traces), which is where Figma's approach hurt.**

### Comparison Matrix

Verdict line convention: **COPY** = adopt directly; **REFINE** = adopt with modification; **AVOID** = cautionary, do not replicate.

| Ecosystem | Capability advertisement | Lazy activation | Lifecycle | Events | Permissions | UI model | Multi-lang | Versioning | Revocation | DX | Verdict for UAR |
|---|---|---|---|---|---|---|---|---|---|---|---|
| **VS Code** | `contributes` manifest (closed vocab) | activation events (strong) | activate/deactivate | commands/webview RPC | **none — full user authority** | webview HTML + native | any (via subprocess) | `engines` field | store takedown only | excellent | **COPY** manifest+lazy activation; **REFINE** to open vocab; **AVOID** its no-permission model |
| **Extism** | host-defined | host-controlled | host-driven | host functions | host-gated imports | none | 15+ langs (strong) | ABI-generic | none | good | **AVOID as base** (untyped byte-pipe); cite as demand proof |
| **wasmCloud** | WIT imports/exports | link-at-runtime | provider lifecycle | lattice pub/sub (wRPC) | WIT-typed capability | none (backend) | Rust/Go/TS | WIT + wRPC | host-controlled | wash CLI, wit2wadm | **COPY** WIT capability wiring + lattice-as-transport-seam |
| **Fermyon Spin** | `spin.toml` triggers | trigger-driven | run-to-completion | trigger types | `allowed_outbound_hosts` | none | any→component | manifest version gate | OCI registry | `spin new/watch` (strong) | **COPY** manifest triggers + watch loop; **REFINE** closed-trigger-enum → open worlds |
| **Shopify Functions/UI ext** | `shopify.extension.toml` targets | target-driven | host-injected | shopify global API | no payment/DOM access | **remote-ui, host-rendered (near-A2UI)** | JS/WASM | API version | app review | strong | **COPY** — closest analog to A2UI-only; validates it |
| **Stripe Apps** | `stripe-app.json` + viewports | view-driven | install/permission | proxied async events | manifest permissions | **UI toolkit, no HTML/CSS (near-A2UI)** | TS/React | SDK versioned | app review | Jest test kit | **COPY** typed-component UI + test kit; validates A2UI-only |
| **MCP Apps (SEP-1865)** | `ui://` resource + `_meta.ui` | tool-declared | prefetch/cache/review | JSON-RPC | sandboxed iframe | HTML in sandboxed iframe | any | extension framework + deprecation policy | host review | maturing | **INTEROP** — support both directions; mirror prefetch-then-review lifecycle |
| **proxy-wasm** | ABI markers | context creation | **root vs per-event (strong)** | on_* callbacks | host functions | none | C++/Rust/AS | ABI version (0.2.1) | host | moderate | **COPY** root/per-event lifecycle; upgrade raw ABI→typed WIT |
| **WordPress hooks** | none (runtime `add_action`) | always loaded | plugin activate/deactivate | **actions/filters (strongest ecosystem)** | **none — full trust** | PHP/server | PHP only | none (breakage common) | .org delisting | huge ecosystem | **COPY** ordered-multi-contributor model; **AVOID** untyped + full-trust |
| **Chrome MV3** | manifest permissions | event service workers | install/enable | declarativeNetRequest | **permission combinations reviewed** | HTML | JS | manifest version | **blocklist/remote kill (strong)** | good | **COPY** permission-combination review + blocklist ops |
| **OSGi** | Provide/Require-Capability | lazy bundle activation | full bundle lifecycle | service registry events | Java security mgr | none | JVM langs | semantic version ranges | none | **poor (complexity collapse)** | **AVOID** transitive resolver; cautionary bound on wiring ambition |
| **Zed** | `extension.toml` + `[[capabilities]]` | on-demand load | download/unpack/instantiate/reload | LSP/host API | **capability-scoped (process:exec, download_file, npm)** | none yet | Rust→wasm32-wasip2 | **`zed_extension_api` versioned WIT (strong)** | git/store | hot-reload, wit-bindgen | **COPY** versioned-WIT-API discipline + capability scoping + hot-reload |
| **Figma** | manifest | on-run | run/close | postMessage | networkAccess allowlist | iframe + QuickJS-in-WASM | JS | API versioned | store | **poor debuggability** | **AVOID** clever JS-realm sandboxing; UAR's WASM sandbox is the right end state |

---

## PART 2 — ARCHITECTURE

### 2.1 Crate / package topology

Per the D3 decision (all hosts share one implementation):

- **`uar-plugin-contract`** (crate) — pure types generated from the WIT family; no host logic. The single source of truth. Published to crates.io and mirrored as WIT packages to an OCI/wkg registry (`ghcr.io/prometheus-ags/wit/uar/plugin`).
- **`uar-plugin-host`** (crate) — the shared dispatch trait + sandbox host: capability wiring at instantiation (undeclared imports → `MissingCapability` at load, never trap-at-first-use, inheriting librefang), the fs/net/kv/agent/env/time host implementations with SSRF and path-traversal guards, epoch budgets, event bus, provenance stamping, lifecycle state machine, generation manager. **One implementation, shared by every host** — this is the architectural keystone.
- **Per-host adapter crates:** `uar-host-librefang` (headless: skill-task + hook), `uar-host-knowme` (full-UI: + events/a2ui/settings/app-shell/provider), `uar-host-server` (axum 0.8 service host). Adapters wire profile-specific imports and the trusted host layer (device keys, entitlement VC wallet, approval gate — never in plugin/runtime space); they contain **no sandbox logic**.
- **`uar-plugin-cli`** — scaffold/build/watch/test/publish tooling.
- **Browser host:** a jco-transpilation build target of the contract + a JS re-implementation of the host imports (the sandbox host cannot be shared verbatim here — this is the honest host-parity seam).

### 2.2 WIT family layout (`uar:plugin`)

```
package uar:plugin@0.2.0;

// --- host imports (near-verbatim librefang, additive extensions) ---
interface fs   { /* read/write/list-entries, path-traversal guarded */ }
interface net  { /* fetch, SSRF-guarded: block loopback/link-local/RFC1918/metadata */ }
interface kv   { /* get/set */ }
interface agent{ /* send/spawn */ }
interface env  { /* allowlisted read */ }
interface time { /* ungated now() */ }
// additive (new):
interface events   { /* emit (host-stamped provenance), subscribe; namespaced topics, rate-capped */ }
interface infer    { /* LLM/embedding calls — REDUCED/absent inside contributor stages (reentrancy guard) */ }
interface memory   { /* read/write scoped store */ }
interface a2ui     { /* emit validated A2UI data (uar.a2ui/1, catalog urn:uar:a2ui:catalog:1) */ }
interface settings { /* read validated settings (JSON Schema 2020-12) */ }

// --- error taxonomy (librefang, extended) ---
variant plugin-error {
  capability-denied, path-denied, ssrf-denied, io(string),
  invalid-argument(string), timeout, internal(string),
  // additive:
  missing-capability(string), budget-exceeded, quarantined, revoked
}

// --- worlds per plugin kind (superset of base) ---
world skill-task  { import ...base...; export run: func() -> result<_, plugin-error>; }
world hook        { import ...base, events; export on-event: func(ev: event) -> result<_, plugin-error>; }
world contributor { import ...base-minus-infer, events; export contribute: func(stage-input) -> result<contribution, plugin-error>; }
world agent       { /* A2A-shaped task lifecycle */ }
world provider    { /* LLM/embedding/modality with fallback */ }
world app-shell   { import ...base, a2ui, settings, events; /* full mini-app */ }
world service     { /* wasi:http or supervised sidecar; OpenAPI->MCP */ }
world workflow    { /* declarative graph, BossFang-orchestrated */ }
```

**Profiles as worlds-composition:**
- **headless profile** = base host + `skill-task` + `hook` (librefang/BossFang gateway nodes).
- **full-UI profile** = headless + `events` + `a2ui` + `settings` + `app-shell` + `provider` (KnowMe).

**Skill-task convergence (the ADR):** `librefang:plugin@0.1.0` `run()`, `uar-skill.wit`, and `prometheus:component` skill interfaces converge into `uar:plugin/skill-task`. **Conformance requirement: an existing `librefang:plugin@0.1.0` plugin must load and run unchanged** — the base `run()` world is byte-compatible; additive interfaces are opt-in.

### 2.3 Capability card schema v2

One card shape for every capability (recognized or invented):
```
id            : namespaced (uar:* runtime | knowme:* product | reverse-DNS third-party)
version       : semver
name/desc/tags: embedded locally (semantic discovery/routing; registry ships TEXT never vectors)
settings_schema: JSON Schema 2020-12
actions       : A2A-style {input_schema, output_schema, streaming: bool}  — validated UNIVERSALLY
events        : {emits[], consumes[]} namespaced topics
chunks        : stream-chunk types incl. a2ui-patch
ui            : optional a2ui_components
requires/enhances: composition (flat, non-transitive — OSGi caution)
platforms     : [desktop, ios, android, web] with execution backend hints
profile_required: headless | full-ui         (NEW — installer answers "will this run here" mechanically)
epoch_budget_ms: int
combination_risk: [NEW — declared risky combos, e.g. {memory:read + net:egress = exfiltration}]
namespace_proof: [NEW — did:web/DNS-TXT or transparency-log receipt]
```

### 2.4 Event fabric

- **In-process bus** with AG-UI (`uar.agui/1`) as the event vocabulary; topics namespaced.
- **Host-stamped provenance**: the host — not the plugin — records emitter identity and generation; plugins may emit only to their own namespace (allowlist) under per-topic rate caps.
- **FRF transport adapter** (wasmCloud-lattice-style): behind the bus port, the host routes an event locally or onto an FRF transport (str0m SFU / LiveKit / OFP mesh / Matrix inbound / ATProto outbound). Plugins are transport-unaware. **Entity-watch RPC (`WatchEntityType`)** is adjacent machinery on the same port for cross-device state.
- **Contributor reentrancy guard**: the `contributor` world imports a host set *without* `infer` (no host-inference inside the prompt pipeline); if inference is genuinely needed, a depth-1 non-reentrant lane is granted explicitly and metered.

### 2.5 Lifecycle state machine + generations/rollback

States: `installed → verified → registered → active ⇄ inactive → (upgrading) → removed`, with `quarantined` as a terminal-until-cleared state reachable from any active state.

- **Content-addressed immutable generations**: each install/upgrade is an immutable generation addressed by content hash, with `current`/`previous` pointers. **Atomic swap** of payload + registry entry + catalog + settings together; rollback = repoint `current`→`previous`. (Mirrors Figma's "swappable architecture" that let them switch sandboxes in days, and NixOS-style generations.)
- **Crash-loop containment**: a circuit breaker per plugin; N failures in window → **auto-quarantine with user notice**. VS Code's disposable extension-host + ~3-second unresponsiveness watchdog (kill and restart without losing work) is the proof pattern.
- **proxy-wasm root/per-event contexts** map onto verify/activate (root) vs. hook/action firings (per-event); `verify` returning an error rejects the generation before it can activate.

### 2.6 Registry + advisory/revocation

- **Distribution:** components as **OCI artifacts** (proven by Spin, wasmCloud, SpinKube) via **wkg**/`wasm-pkg`; optional **warg** transparency log for federated namespacing and "package transparency" (Certificate-Transparency-inspired). A **first-come transparency log** is the pragmatic namespace-squatting defense when `did:web`/DNS-TXT proof is absent. (As of 2026 the ecosystem's registry story is OCI-first with warg still in-development — start OCI, keep warg optional.)
- **Signing:** artifacts signed; verify at install. Sigstore/cosign + TUF are the industry standard for keyless signing + transparency; adopt cosign-style signatures with a TUF-style root of trust for the advisory feed.
- **Advisory / revocation (highest-priority gap):** a **signed advisory/blocklist published beside the registry**, checked at **activation + periodically**, triggering **auto-quarantine**. This is modeled directly on Chrome Web Store and Firefox AMO **blocklist operations** — the proven mechanism for remotely killing malicious extensions already installed on user devices (in early 2026 Google removed credential-stealing extensions with 10,000+ installs via exactly this machinery). Without this, UAR has no answer to a compromised popular plugin.
- **Permission-combination surfacing:** the install UI must render *combinations*, not just individual permissions (Chrome Web Store program policy requires the narrowest permissions and reviews combinations, because `memory:read + net:egress` = exfiltration even when each alone is benign). The card's `combination_risk` field drives this.

### 2.7 Discovery / semantic routing

Registry ships **text** (name/description/tags), never vectors; the host embeds locally for semantic discovery and routing. Unrecognized capabilities remain discoverable (open vocabulary) while sandboxed. Recognized capabilities additionally get first-class pipeline surfaces.

### 2.8 Host matrix with execution backends & honest budgets

| Target | Backend | Perf vs native | Distribution | Notes |
|---|---|---|---|---|
| **Desktop (x86_64/aarch64)** | Cranelift AOT `.cwasm` + pooling allocator | ~1.45–1.6× real workloads; near-native on kernels | synced, content-addressed | full profile (KnowMe) |
| **Android (aarch64)** | Cranelift where W^X permits; else Pulley | Cranelift near-desktop; Pulley ~order-of-magnitude | synced | opportunistic Cranelift |
| **iOS/iPadOS** | **Pulley interpreter (mandatory — no JIT)** | **~order-of-magnitude slower than Cranelift** | **bundled (App Store 3.3.2) vs synced split** | `.cwasm` AOT native code forbidden; keep hot paths off-device |
| **Browser** | **jco transpile → core-wasm + JS glue (NOT wasmtime)** | V8 WASM; async limited | served | host-parity seam; WASI shims experimental |

**Distribution split for iOS (App Store guideline 3.3.2):** bundled plugins ship inside the app binary and pass review as first-party code; dynamically-synced plugins run interpreted (Pulley) as *data*, the same legal basis JavaScriptCore/Hermes/QuickJS rely on (UTM SE's TCTI interpreter precedent, approved on the App Store). Both paths must be explicit in the card's `platforms`/distribution metadata. Third-party iOS apps "cannot create writable-executable memory (outside narrow carve-outs like the EU's BrowserEngineKit entitlements)," so a JIT has nothing to compile into — interpretation is the only path.

### 2.9 Multi-language toolchain support

Rust (`wit-bindgen`, first-class), TinyGo, **componentize-py** (build-time import resolution caveat), **ComponentizeJS/StarlingMonkey** (~8 MB engine/component — desktop-first). WASI **0.2 today**; **WASI 0.3** (released June 11, 2026; Wasmtime 43+, jco) brings native async (`stream<T>`/`future<T>`, `wasi:io` absorbed into the Canonical ABI) — **do not block the contract on 0.3**; ship on 0.2 and virtualize/polyfill, adopting 0.3 async in a minor contract bump once toolchains settle. WASI 1.0 is targeted late-2026/early-2027 — the versioning-humility rule applies.

### 2.10 Developer experience

- **Scaffold CLI:** `uar plugin new -k skill-task|hook|contributor|...` (Spin `spin new`/Zed template pattern).
- **Hot reload:** `uar plugin watch` (Spin `watch` globs + Zed hot-swap) — rebuild + host-side generation swap without runtime restart.
- **Playground:** local host harness that renders A2UI in the shared renderers (Flutter `A2uiSurfaceView`, React `a2ui-*`).
- **Conformance kit:** the librefang zero-source-change test as seed; Stripe-style typed test matchers for A2UI; host-parity suite (same plugin under wasmtime and jco hosts).
- **Debuggability investment** (the Figma lesson): source maps, structured plugin-error traces, epoch-timeout diagnostics — because the WASM sandbox's weakness is debugging, not security.

---

## PART 3 — FUNCTIONAL SPECIFICATION

**Error envelope (all fallible exports):** `result<T, plugin-error>` with the extended variant taxonomy (§2.2). Hosts must map every internal failure to a named variant; traps are a bug, not an error path.

### Functional Requirements

- **FR-1 (kinds).** The system SHALL support all eight worlds (skill-task, hook, contributor, agent, provider, app-shell, service, workflow) via a superset-of-base WIT world per kind. *Acceptance:* a skill-task and an app-shell plugin both load under the same host crate; kind is derived from the capability card.
- **FR-2 (open vocabulary).** Unrecognized capabilities SHALL be carried, sandboxed, and semantically discoverable; recognized ones SHALL receive structured pipeline surfaces. *Acceptance:* a third-party reverse-DNS capability installs, is discoverable by tag search, and executes sandboxed with no host code changes.
- **FR-3 (librefang conformance).** An unmodified `librefang:plugin@0.1.0` plugin SHALL load and run. *Acceptance:* the librefang conformance suite passes with zero source changes.
- **FR-4 (lifecycle).** The full lifecycle (install→verify→register→activate→deactivate→upgrade→remove) plus runtime activation events SHALL be implemented as the §2.5 state machine. *Acceptance:* each transition emits a host-stamped lifecycle event; `verify` failure blocks activation.
- **FR-5 (lazy activation).** Plugins SHALL activate lazily from card-derived triggers (events.consumes/actions), with an explicit eager-activation opt-in. *Acceptance:* an idle installed plugin consumes no instance memory until its trigger fires.
- **FR-6 (events + FRF).** Events SHALL use AG-UI vocabulary over an in-process bus with a transport-transparent FRF adapter. *Acceptance:* the same plugin's events flow locally in one host and over a WebRTC data channel in another with no plugin change.
- **FR-7 (multi-language).** Rust, TinyGo, Python (componentize-py), JS/TS (ComponentizeJS) plugins SHALL all satisfy the contract. *Acceptance:* one conformance plugin authored in each language passes the kit.
- **FR-8 (host matrix).** Desktop (Cranelift AOT), iOS (Pulley), Android (Cranelift/Pulley), browser (jco) SHALL all run a conformant plugin. *Acceptance:* the host-parity suite passes on all four with documented behavioral deltas.
- **FR-9 (generations/rollback).** Installs/upgrades SHALL be content-addressed immutable generations with atomic multi-artifact swap and current/previous rollback. *Acceptance:* an upgrade then rollback restores byte-identical prior behavior.
- **FR-10 (universal I/O validation).** Action input/output schemas SHALL be validated for ALL capabilities including unrecognized ones. *Acceptance:* a malformed action payload to a third-party capability is rejected with `invalid-argument` before guest entry.
- **FR-11 (contributor pipeline).** Contributions SHALL carry provenance, declared priority + stable tiebreak, per-stage aggregate wall budgets, and third-party fragments SHALL never precede first-party system content. *Acceptance:* ordering is deterministic; a budget-exceeding stage is truncated with `budget-exceeded`, not dropped silently.
- **FR-12 (MCP interop).** The host SHALL expose plugin tools to MCP and allow plugins to mount MCP servers, with trust tiers; app-shell UI SHALL interoperate with MCP Apps (`ui://` + `_meta.ui`). *Acceptance:* a plugin tool appears in an MCP client; an MCP-App UI resource renders via the A2UI seam.

### Non-Functional Requirements

- **NFR-1 (perf honesty).** Published budgets SHALL state per-tier costs: Cranelift ~1.45–1.6× native (real workloads, USENIX ATC 2019), Pulley ~order-of-magnitude slower than Cranelift, JS component +~8 MB. *Acceptance:* the host matrix doc carries these figures with sources; no "zero-cost" claim ships.
- **NFR-2 (instantiation).** Desktop cold instantiation via AOT + pooling SHALL target sub-millisecond. *Acceptance:* measured p50 < 1 ms for a base skill-task on reference hardware (target, not guarantee; Fermyon's production ~0.5 ms is the benchmark to beat).
- **NFR-3 (isolation).** The Component Model shared-nothing boundary SHALL be the sole security boundary; no JS-realm-style tricks. *Acceptance:* a hostile plugin cannot access another plugin's memory or host state beyond declared capabilities.

### Security Requirements (mapping the 12 review holes)

- **SR-1 (namespace).** Reverse-DNS ids SHALL carry `namespace_proof` via did:web/DNS-TXT verification OR a first-come transparency-log receipt (warg-style). *Acceptance:* an unproven squatted namespace is flagged at install.
- **SR-2 (revocation).** A signed advisory/blocklist SHALL be checked at activation + periodically, auto-quarantining revoked plugins. *Acceptance:* publishing a revocation quarantines an already-installed plugin within one poll interval, with user notice.
- **SR-3 (combinations).** Install UI SHALL surface risky permission combinations from `combination_risk`. *Acceptance:* `memory:read + net:egress` shows an exfiltration warning at install.
- **SR-4 (event forgery).** Host SHALL stamp provenance and enforce emit-to-own-namespace + rate caps. *Acceptance:* a plugin emitting to a foreign namespace is denied `capability-denied`.
- **SR-5 (reentrancy).** Contributor stages SHALL receive reduced imports (no `infer`) or a depth-1 non-reentrant metered lane. *Acceptance:* an attempted host-inference call inside a contributor stage fails `capability-denied`.
- **SR-6 (sidecar).** Service sidecars SHALL reuse OS sandbox fences (Seatbelt/bwrap) and broker over UDS, not loopback TCP. *Acceptance:* a sidecar cannot open an unfenced network socket; broker traffic is UDS-only.
- **SR-7 (rollback).** Per FR-9.
- **SR-8 (crash-loop).** Circuit breaker SHALL auto-quarantine after N failures/window with user notice. *Acceptance:* a panic-looping plugin is quarantined without degrading the host.
- **SR-9 (ordering/budgets).** Per FR-11.
- **SR-10 (universal validation).** Per FR-10.
- **SR-11 (mobile).** iOS SHALL run dynamically-installed plugins under Pulley; `.cwasm` AOT native code SHALL NOT be loaded on iOS; bundled-vs-synced split SHALL satisfy App Store 3.3.2. *Acceptance:* a synced plugin runs interpreted on a physical iPhone and passes review.
- **SR-12 (versioning).** The contract SHALL ship as **0.2.0**; **1.0.0 SHALL NOT be declared until real-world plugins across ≥3 kinds and ≥2 hosts have exercised the surface.** *Acceptance:* the ADR records the 1.0 trigger criteria.

### Profile Conformance Requirements

- **PCR-1.** A plugin declaring `profile_required = headless` SHALL run on both headless and full-UI hosts; a `full-ui` plugin SHALL be mechanically rejected at install on a headless host with a clear message. *Acceptance:* the librefang gateway rejects an app-shell plugin at install, not at runtime.
- **PCR-2.** The conformance kit SHALL verify each profile's host-import set matches the WIT world composition in §2.2.

---

## PART 4 — IMPLEMENTATION PLAN

Estimation convention: **agent-build time in agent-days** using the validated **11.3× PMPO velocity compression** (implementation only); **elapsed calendar time stated separately**; **human gates named explicitly** as the critical path. Agent-days below are implementation-only and already compressed.

### Phase 0 — Contract + ADRs (foundation)
PRs: (a) `uar-plugin-contract` WIT family @0.2.0; (b) ADR-001 skill-task convergence (librefang `run()` + uar-skill.wit + prometheus:component); (c) ADR-002 version-numbering (0.2.0 now, 1.0.0 trigger criteria — SR-12); (d) ADR-003 error taxonomy extension; (e) capability-card schema v2.
- **Agent-build:** ~4 agent-days. **Elapsed:** dominated by **human gate: ADR + adversarial review** (multi-day, serial).

### Phase 1 — Shared host crate
PRs: `uar-plugin-host` — capability wiring (MissingCapability-at-load), fs/net/kv/agent/env/time with SSRF+path guards, epoch budgets, lifecycle state machine, generation manager, event bus + provenance/rate-caps, minimal signed-blocklist check stub.
- **Agent-build:** ~9 agent-days. **Elapsed:** **human gate: security/adversarial review of sandbox host** (this is the crown-jewel review; do not compress it).

### Phase 2 — Worlds in evidence order
1. **skill-task + hook** first (have implementations: librefang/BossFang). Includes FR-3 librefang conformance suite.
2. **contributor** next, built against the **KnowMe pipeline** (reentrancy guard, ordering/budgets).
3. **app-shell / service / workflow** versioned separately at **0.x until proven** (do not stabilize speculatively).
- **Agent-build:** ~12 agent-days total. **Elapsed:** gated by **conformance sign-offs** and the **KnowMe pipeline dependency** (contributor cannot finalize until the KnowMe ingress→egress pipeline stages are frozen — external critical-path dependency).

### Phase 3 — FRF event transport
PRs: lattice-style transport adapter behind the bus port; `WatchEntityType` entity-watch; local↔WebRTC/OFP/Matrix/ATProto routing.
- **Agent-build:** ~6 agent-days. **Elapsed:** gated by **FRF interface stability** (str0m/LiveKit adapter maturity).

### Phase 4 — Registry + advisory/revocation
PRs: OCI/wkg distribution; cosign-style signing + TUF-style advisory root; **signed blocklist checked at activation+periodic → auto-quarantine** (SR-2); permission-combination install UI (SR-3); namespace-proof/transparency-log (SR-1).
- **Agent-build:** ~8 agent-days. **Elapsed:** **human gate: revocation-ops runbook + trust-root ceremony** (operational, not just code).

### Phase 5 — Mobile / web hosts
PRs: iOS Pulley host + bundled/synced split (SR-11); Android Cranelift/Pulley; browser jco-transpile host + JS host-import re-implementation; host-parity conformance suite (FR-8).
- **Agent-build:** ~10 agent-days. **Elapsed:** **human gate: physical-device validation + App Store review** (serial, unavoidable, multi-week wall-clock; the swift-wasmtime iOS Pulley path is validated in principle but has no App Store gate yet — UAR must be the one to prove it).

### Phase 6 — DX tooling
PRs: `uar plugin new/build/watch/test/publish`; playground with A2UI renderers; conformance kit packaging; debuggability (source maps, structured traces).
- **Agent-build:** ~7 agent-days. **Elapsed:** parallelizable with Phases 3–5.

**Total agent-build ≈ 56 agent-days (implementation-only, compressed).** **Calendar elapsed is dominated by human gates, not build:** ADR/adversarial reviews (Phases 0–1), conformance sign-offs + KnowMe pipeline freeze (Phase 2), trust-root ceremony (Phase 4), and physical-device + App Store review (Phase 5). These gates are serial and human-bound; they, not agent throughput, set the release date.

### Risks — named failure scenarios ("the scenario that hurts Prometheus")

- **R-1 (revocation absence exploited).** A popular third-party plugin is compromised post-install; without SR-2 shipped, every KnowMe device runs the malicious generation until manual uninstall — precisely the Chrome-extension credential-theft pattern seen with 10,000+-install extensions in early 2026. *This is the highest-severity scenario — Phase 4 revocation must not slip behind Phase 5.* Mitigation: ship a minimal signed-blocklist check in Phase 1 as a stub, fully in Phase 4.
- **R-2 (iOS review rejection).** App Store rejects synced-plugin execution as "downloaded executable code." Mitigation: lean on the interpreted-as-data precedent (JavaScriptCore/Hermes/QuickJS, UTM SE's approved TCTI interpreter), keep synced plugins Pulley-interpreted, and keep a bundled-only fallback distribution ready.
- **R-3 (contributor reentrancy DoS).** A contributor triggers host inference that re-enters the pipeline, causing unbounded recursion / cost blowup. Mitigation: SR-5 reduced-imports is non-negotiable in Phase 2.
- **R-4 (OSGi-style wiring creep).** `requires/enhances` grows into a transitive resolver, reproducing OSGi's complexity collapse and killing DX. Mitigation: ADR bans transitive resolution; composition stays flat + install-time explain-able.
- **R-5 (perf over-promise).** Marketing repeats "abstraction costs nothing"; a customer benchmarks a real workload on iOS and finds a ~10× slowdown. Mitigation: NFR-1 honest budgets published up front.

### Open decisions with triggers

- **OD-1: WASI 0.2 vs 0.3.** Ship on 0.2; **trigger to adopt 0.3 async:** when jco async support lands cleanly AND componentize-py/ComponentizeJS 0.3 support is polished.
- **OD-2: warg vs OCI-only.** Start OCI + wkg; **trigger for warg transparency log:** first namespace-squatting incident OR >100 third-party publishers.
- **OD-3: 1.0.0 contract.** **Trigger:** real plugins across ≥3 kinds on ≥2 hosts, per SR-12/librefang humility rule.
- **OD-4: shared SpiderMonkey embedding on mobile.** **Trigger:** >1 JS plugin common on mobile AND Bytecode Alliance ships shared-embedding support.

### Explicit non-goals

- Not building an OSGi-style transitive dependency resolver (R-4).
- Not shipping React/Flutter code across the sandbox (A2UI-only, validated by Shopify/Stripe/MCP Apps).
- Not using Extism or a byte-pipe ABI (stay WIT-native).
- Not claiming native/zero-cost performance (NFR-1).
- Not stabilizing app-shell/service/workflow worlds before real usage (Phase 2).
- Not adding JS-realm-style sandbox cleverness on top of the WASM boundary (Figma lesson).

---

## CAVEATS

- **Performance figures blend primary and vendor sources.** The Pulley "order-of-magnitude" slowdown is Wasmtime's own qualitative characterization, not a quoted fixed multiplier; Fermyon's "cold starts under half a millisecond" is a CEO/press-release claim, not independently benchmarked. The Cranelift "~2%/~14%" figure is from small PolyBench-C kernels (PLDI 2017, where only 7 benchmarks reach within 10% of native) and is contradicted for real workloads by the USENIX ATC 2019 "Not So Fast" study (45–55% average slowdown, up to 2.08×–2.5×). No official absolute-µs instantiation table exists — the docs describe mechanisms only. Treat all perf numbers as order-of-magnitude planning inputs, not contractual SLAs.
- **MCP Apps / SEP-1865 is fast-moving.** MCP Apps went Final Jan 26, 2026 (nine launch partners incl. Figma, Slack, Canva); the broader MCP 2026-07-28 spec (stateless core, Extensions framework, Tasks) only just finalized. The interop seam (FR-12) targets a moving standard — expect a follow-up contract bump.
- **iOS App Store path is validated in principle, not in production for UAR.** The swift-wasmtime Pulley path and interpreted-code precedent are real, but no App Store review gate has yet approved UAR's specific synced-plugin model — R-2 remains live until a physical-device + review pass completes.
- **A few Part-1 ecosystems (Home Assistant, Kubernetes operators/admission webhooks, Raycast, Stream Deck, Salesforce Lightning/AppExchange, Sigstore/TUF specifics, Obsidian curation) were analyzed from general engineering knowledge rather than fresh primary sources**; their lessons (declared-not-discovered participation, admission-webhook gating, keyless signing + transparency logs, community curation) reinforce but do not change the conclusions drawn from the more deeply-sourced analogs. The Zed extension analysis was deepened via a dedicated research pass (versioned `zed_extension_api` WIT contract, `[[capabilities]]` scoping incl. `process:exec`/`download_file`/`npm:install`, Tree-sitter-in-WASM, `wasm32-wasip2` target, and CVE-2026-27976 demonstrating that the host-side unpack path is a separate attack surface from the Wasm guest sandbox).
- **The estimation model assumes the 11.3× PMPO compression holds for this domain.** Sandbox-security and cross-platform host work are review-heavy; if adversarial reviews surface architectural rework, agent-build compression will not shorten the human-gated critical path — which dominates the calendar regardless.
