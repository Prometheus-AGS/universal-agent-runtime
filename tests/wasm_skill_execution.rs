//! S1: a wasm skill component actually **executes** in UAR.
//!
//! # Why this test is the one that matters
//!
//! `wasm_runtime.rs` previously returned a placeholder:
//!
//! ```ignore
//! Ok(format!("<wasm skill '{skill_id}' loaded but binding not yet generated; …>"))
//! ```
//!
//! Everything downstream of that — discovery, registration, dispatch wiring —
//! worked. A test that only checked "the call completed" would have passed
//! against the stub, which is why the acceptance criterion is explicitly
//! **assert on the returned value**.
//!
//! The whole mobile portability story rests on this: components are the answer
//! for skills that cannot spawn a process. Until one has run in the host, that
//! answer is a design, not a capability. Well-formed is not working.

#![cfg(feature = "wasm-runtime")]

use std::path::PathBuf;

use universal_agent_runtime::uar::runtime::skills::wasm_runtime::WasmSkillRuntime;

/// The reference component built by `change-msp-006`.
///
/// Located relative to the pack submodule so the test does not depend on the
/// working directory. Skips **loudly** when absent — a silent skip would let
/// this protection rot exactly where it matters most.
fn reference_component() -> Option<PathBuf> {
    let candidates = [
        "crates/prometheus-skill-system/skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm",
        "../prometheus-skill-pack/skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm",
    ];
    candidates.iter().map(PathBuf::from).find(|p| p.exists())
}

/// THE ASSERTION: the component returns **its own output**, not a placeholder.
#[tokio::test]
async fn a_wasm_skill_component_executes_and_returns_its_own_output() {
    let Some(path) = reference_component() else {
        eprintln!(
            "SKIPPED wasm execution: no reference component found. This test \
             proves S1 — that a component EXECUTES rather than merely loads — \
             and must not be reported as passing when it never ran. Build it \
             with the msp-006 reference skill."
        );
        return;
    };

    let runtime = WasmSkillRuntime::new().expect("construct the wasm runtime");
    runtime
        .register("entity-graph-optimize", &path)
        .await
        .expect("register the reference component");

    assert!(
        runtime.has("entity-graph-optimize").await,
        "registration must make the skill discoverable"
    );

    let output = runtime
        .run("entity-graph-optimize", "{\"nodes\":3}")
        .await
        .expect("the component's `run` export must execute");

    // The stub's exact shape. If this ever reappears, the runtime has
    // regressed to not-executing while still returning Ok.
    assert!(
        !output.contains("binding not yet generated"),
        "the runtime returned the PLACEHOLDER, not component output: {output:?}. \
         The call completing is not evidence of execution."
    );
    assert!(
        !output.is_empty(),
        "the component returned an empty string; `run` must produce output"
    );

    eprintln!("wasm component returned: {output:?}");
}

/// An unregistered skill must fail, not return something plausible.
#[tokio::test]
async fn running_an_unregistered_skill_is_an_error() {
    let runtime = WasmSkillRuntime::new().expect("construct the wasm runtime");

    let result = runtime.run("no-such-skill", "{}").await;
    assert!(
        result.is_err(),
        "an unregistered skill must error; returning Ok would let a caller \
         treat 'nothing ran' as 'it ran and produced this'"
    );
}
