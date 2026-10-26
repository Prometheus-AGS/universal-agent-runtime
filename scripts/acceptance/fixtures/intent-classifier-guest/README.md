# Intent classifier component producer template

Source only. Gate CLOSED. No Cargo, compiler, import, formatter, test, service,
model call, or review verification ran. Cargo.toml.in is deliberately inert; no
product Cargo.toml or Cargo.lock is changed. Historical source004/build receipts
do not cover these newly authored fixture sources.

## Actual guest

src/lib.rs parses the host's final eligible-skills metadata entry, scores its
actual keywords against the current query with the host rules classifier's
coverage formula, and returns matching current labels sorted by score/label.
It contains no fixed test label, parcel code, assistant response or success flag.
Empty matching yields out-of-scope; malformed metadata yields a closed guest error.
Bounded input/query/skill/keyword/result sizes complement the host16MiB/10million
fuel bounds; actual fuel/memory fit is unverified. No host imports, WASI, file,
network, clock, environment, tool or random generator is used by guest source.

wit/intent-classifier.wit exports the exact intent:classifier/classifier@0.1.0
interface and classify(string,string)->result<record,string> required by
src/uar/runtime/matching/intent/wasm.rs. Its score uses modern WIT f32, the actual
Float32 ABI expected by that host; the host's illustrative float32 spelling is
not copied as an unknown identifier into the current WIT parser.

The generated module is bindings.rs. Its interface path and export macro follow
[the pinned tool's bindings generator](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/bindings.rs)
and [versioned-interface examples](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/tests/build.rs).
Actual generated bindings are derived inputs, not checked-in fixture source.

## Exact dependency/tool basis

Selected installed metadata in ~/.cargo/.crates2.json reports cargo-component0.21.1;
no tool was executed to obtain that version. This tool uses wit-bindgen-rust0.41
and wit-component0.227 according to its
[versioned Cargo source](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/Cargo.toml).
Its generated bindings directly reference wit_bindgen_rt, so the guest pins
wit-bindgen-rt=0.41.0 rather than mixing the unrelated main lock's wit-bindgen0.57.
The runtime package is declared in the
[official0.41 source](https://github.com/bytecodealliance/wit-bindgen/blob/v0.41.0/crates/guest-rust/rt/Cargo.toml),
with version0.41.0 in that release's
[workspace metadata](https://github.com/bytecodealliance/wit-bindgen/blob/v0.41.0/Cargo.toml).
serde_json=1.0.154 matches actual root Cargo.lock and the installed official crate
Cargo.toml. Dependencies are exactly pinned; derived transitive lock remains to
be resolved under the explicitly named future exception.

Cargo component supports local target path/world metadata; bindings.formatfalse
avoids its implicit rustfmt step. These fields come from its
[metadata definition](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/metadata.rs).
The explicit wasm32-unknown-unknown target is supported by
[the pinned build path](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/lib.rs)
and its own upstream source tests. No WASI adapter is needed by this import-free
world. No wasm-tools executable or extra componentizer crate is required.

## Producer copy and future exact commands

Root's future approved producer creates a new private generated package at:

`/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest`

Copy Cargo.toml.in as Cargo.toml; copy src/lib.rs and
wit/intent-classifier.wit preserving those relative paths. Put generated bindings,
Cargo.lock, any Cargo-component.lock and every build output only under this
owned generated package. The standalone workspace table prevents adoption by the
product workspace. Do not run cargo component new/add: they mutate manifests and
may resolve unpinned latest dependencies or create editor/config files.

The producer must name these serial argv explicitly in its future approval:

```
["/Users/gqadonis/.cargo/bin/cargo", "generate-lockfile", "--manifest-path", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/Cargo.toml"]
["/Users/gqadonis/.cargo/bin/cargo-component", "component", "build", "--manifest-path", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/Cargo.toml", "--locked", "--release", "--target", "wasm32-unknown-unknown", "--target-dir", "/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/uar-user-outcomes-001/classifier-guest/cargo-target"]
```

These command descriptions are not execution authorization. Freeze actual cargo,
cargo-component, compiler/toolchain and generated-source/lock closure before final
artifact adoption. Set CARGO to the approved exact cargo path. Unset
RUSTUP_TOOLCHAIN in the component producer's environment to prevent an implicit
rustup target installation if its extra prerequisite is absent. Do not run an
extra target/version/compiler probe as an unapproved preflight.

The uncomfortable tool behavior is explicit: cargo-component0.21.1 calls rustc
--print sysroot and checks wasm32-wasip1 availability even for the unknown target;
with RUSTUP_TOOLCHAIN present it can invoke rustup target add. This is visible in
[its target helper](https://github.com/bytecodealliance/cargo-component/blob/v0.21.1/src/target.rs).
Both required target standard libraries must already be installed, or the named
build fails pending separate approval. The unknown-target artifact still remains
import-free; this compatibility prerequisite is not a WASI host permission grant.

Expected component output:

`classifier-guest/cargo-target/wasm32-unknown-unknown/release/uar_intent_classifier_fixture.wasm`

cargo-component writes the componentized result there. installed.classifier_wasm
must reference the actual copied/frozen output, never a generic skill.wasm or a
source WIT file. Final source/artifact identity must include this template, derived
Cargo.toml/lock/bindings, producer/compiler closure and real output bytes. Missing
output is a blocked fixture, not a passing WASM strategy. U10 still requires actual
component instantiation, current eligible ranking, and genuine private skill-body
consumption; a producer build alone proves none of those runtime outcomes.
