# Authorized Docker delivery failure — run39523808909

Source: `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`. ARM64 job118973677204 failed with Cargo exit101 and19 errors. AMD64 was still running when this receipt was authored. Retrieved completed-job logs through the GitHub connector; ordinary gh log viewing refused until whole-run completion. This is evidence from the explicitly authorized Docker delivery build, not a routine verification gate. Verification remains CLOSED. No local compiler, test, formatter, lint, typecheck, policy validator or review gate is authorized.

Run: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39523808909/job/118973677204

Repair source only, preserving lifecycle/admission/actual-join contracts. Do not erase or weaken proof capabilities to satisfy a type error. All repaired source, Boss pin and inventory must be renewed before repeating the same approved delivery command. No image was published by this failed job.

```text
error[E0308]: mismatched types
    --> src/uar/runtime/manager.rs:4063:13
     |
4058 |           let (turn_resilience_policy, _) = match crate::uar::settings::persisted_resilience::resolve_persisted_resilience_policy(
     |  _________________________________________________-
4059 | |                 &self.resilience_policy,
4060 | |                 self.settings_manager.as_deref(),
4061 | |                 &artifact.id,
4062 | |             ).await {
     | |___________________- this expression has type `(ResiliencePolicy, PolicySource)`
4063 |               Ok(resolved) => resolved,
     |               ^^^^^^^^^^^^ expected `(ResiliencePolicy, PolicySource)`, found `Result<_, _>`
     |
     = note: expected tuple `(ResiliencePolicy, PolicySource)`
                 found enum `std::result::Result<_, _>`


error[E0308]: mismatched types
    --> src/uar/runtime/manager.rs:4064:13
     |
4058 |           let (turn_resilience_policy, _) = match crate::uar::settings::persisted_resilience::resolve_persisted_resilience_policy(
     |  _________________________________________________-
4059 | |                 &self.resilience_policy,
4060 | |                 self.settings_manager.as_deref(),
4061 | |                 &artifact.id,
4062 | |             ).await {
     | |___________________- this expression has type `(ResiliencePolicy, PolicySource)`
4063 |               Ok(resolved) => resolved,
4064 |               Err(_) => {
     |               ^^^^^^ expected `(ResiliencePolicy, PolicySource)`, found `Result<_, _>`
     |
     = note: expected tuple `(ResiliencePolicy, PolicySource)`
                 found enum `std::result::Result<_, _>`


error[E0308]: mismatched types
    --> src/uar/runtime/manager.rs:4410:21
     |
4403 |                   match super::presentations::RunPresentationSnapshot::capture(
     |  _______________________-
4404 | |                         self.persistence.as_ref(),
4405 | |                         verified_owner.clone(),
4406 | |                         &effective_policy,
4407 | |                         presentation_negotiation,
4408 | |                     ).await
     | |___________________________- this expression has type `(RunPresentationSnapshot, Vec<std::string::String>)`
4409 |                   {
4410 |                       Ok(captured) => captured,
     |                       ^^^^^^^^^^^^ expected `(RunPresentationSnapshot, Vec<String>)`, found `Result<_, _>`
     |
     = note: expected tuple `(RunPresentationSnapshot, Vec<std::string::String>)`
                 found enum `std::result::Result<_, _>`


error[E0308]: mismatched types
    --> src/uar/runtime/manager.rs:4411:21
     |
4403 |                   match super::presentations::RunPresentationSnapshot::capture(
     |  _______________________-
4404 | |                         self.persistence.as_ref(),
4405 | |                         verified_owner.clone(),
4406 | |                         &effective_policy,
4407 | |                         presentation_negotiation,
4408 | |                     ).await
     | |___________________________- this expression has type `(RunPresentationSnapshot, Vec<std::string::String>)`
...
4411 |                       Err(_) => {
     |                       ^^^^^^ expected `(RunPresentationSnapshot, Vec<String>)`, found `Result<_, _>`
     |
     = note: expected tuple `(RunPresentationSnapshot, Vec<std::string::String>)`
                 found enum `std::result::Result<_, _>`


error[E0382]: the type `Arc` does not implement `Copy`
    --> src/llm/orchestrator.rs:756:27
     |
 714 |         if let Some(native) = self.native_skills.get(provider_name).await {
     |                     ------ this move could be avoided by cloning the original `Arc`, which is inexpensive
...
 717 |                 native,
     |                 ------ value moved here
...
 756 |             let display = native.format_result(&value, output_policy, &self.llm_config.model);
     |                           ^^^^^^ value borrowed here after move
     |
     = note: consider using `Arc::clone`
note: consider changing this parameter type in function `execute_native` to borrow instead if owning the value isn't necessary
    --> src/uar/runtime/native_skill.rs:286:12
     |
 285 | pub(crate) async fn execute_native(
     |                     -------------- in this function
 286 |     skill: Arc<dyn NativeSkill>,
     |            ^^^^^^^^^^^^^^^^^^^^ this parameter takes ownership of the value
     = note: borrow occurs due to deref coercion to `dyn NativeSkill`
note: deref defined here
    --> /usr/local/rustup/toolchains/nightly-2026-07-18-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/sync.rs:2444:5
     |
2444 |     type Target = T;
     |     ^^^^^^^^^^^
help: clone the value to increment its reference count
     |
 717 |                 native.clone(),
     |                       ++++++++


error[E0382]: borrow of moved value: `observed.run_id`
    --> src/uar/api/a2a/thread_service.rs:1234:86
     |
1186 |         } else if scope.spawn("a2a-continuation-projection", async move {
     |                                                              ---------- value moved here
...
1209 |                     Ok(Some(current)) if current.run_id == observed.run_id => current,
     |                                                            --------------- variable moved due to use in coroutine
...
1234 |             scope.record_unresolved_effect(format!("a2a-continuation-projection:{}", observed.run_id));
     |                                                                                      ^^^^^^^^^^^^^^^ value borrowed here after move
     |
     = note: move occurs because `observed.run_id` has type `std::string::String`, which does not implement the `Copy` trait


error[E0382]: borrow of moved value: `task_id`
   --> src/uar/api/full_harness.rs:465:75
    |
427 |     async fn monitor(self: &Arc<Self>, task_id: String) {
    |                                        ------- move occurs because `task_id` has type `std::string::String`, which does not implement the `Copy` trait
...
444 |         if scope.spawn("full-harness-receipt", async move {
    |                                                ---------- value moved here
445 |             let mut first_snapshot = true;
446 |             loop {
    |             ---- inside of this loop
...
459 |                 if authority.refresh_receipt(&task_id).await == Some(true) {
    |                                               ------- variable moved due to use in coroutine
...
465 |             scope.record_unresolved_effect(format!("full-harness-receipt:{task_id}"));
    |                                                                           ^^^^^^^ value borrowed here after move
    |
help: consider cloning the value before moving it into the closure
    |
444 ~         if scope.spawn("full-harness-receipt", {
445 +         let value = task_id.clone();
446 ~         async move {
447 |             let mut first_snapshot = true;
...
460 |                 first_snapshot = false;
461 ~                 if authority.refresh_receipt(&value).await == Some(true) {
462 |                     authority.prune();
...
465 |             }
466 ~         }
467 ~         }).is_err() {
    |


error[E0308]: mismatched types
   --> src/uar/memory/service.rs:303:60
    |
303 |                 Ok(Ok(stream)) => Ok(MemoryLiveStream::new(stream, operation)),
    |                                      --------------------- ^^^^^^ expected `Stream<Value>`, found `Stream<Vec<Value>>`
    |                                      |
    |                                      arguments to this function are incorrect
    |
    = note: expected struct `Stream<Value>`
               found struct `Stream<Vec<Value>>`
note: associated function defined here
   --> src/uar/memory/service.rs:217:8
    |
217 |     fn new(
    |        ^^^
218 |         stream: surrealdb::method::Stream<surrealdb::types::Value>,
    |         ----------------------------------------------------------


error[E0433]: cannot find module or crate `controls_store` in this scope
    --> src/uar/memory/service.rs:1039:9
     |
1039 |         controls_store::next_expiry_due_at(self).await
     |         ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `controls_store`
     |
help: to make use of source file src/uar/memory/controls_store.rs, use `mod controls_store` in this file to declare the module
    -->  src/lib.rs:32:1
     |
  32 + mod controls_store;
     |


error[E0308]: `match` arms have incompatible types
   --> src/uar/realtime/surreal_bus.rs:135:55
    |
129 | /             match &source {
130 | |                 SubscriptionSource::Persistence(db) => {
131 | |                     db.query(format!("DEFINE TABLE IF NOT EXISTS {} SCHEMALESS", topic.table()))
132 | |                         .await?.check()?;
133 | |                     db.select::<Vec<surrealdb::types::Value>>(topic.table()).live().await.map_err(anyhow::Error::from)
    | |                     -------------------------------------------------------------------------------------------------- this is found to be of type `std::result::Result<surrealdb::Stream<Vec<surrealdb::surrealdb_types::Value>>, anyhow::Error>`
134 | |                 }
135 | |                 SubscriptionSource::Memory(client) => client.subscribe_memory().await,
    | |                                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<Stream<Vec<Value>>, Error>`, found `Result<MemoryLiveStream, Error>`
136 | |             }
    | |_____________- `match` arms have incompatible types
    |
    = note: expected enum `Result<surrealdb::Stream<Vec<surrealdb::surrealdb_types::Value>>, _>`
               found enum `Result<MemoryLiveStream, _>`

warning: method call on a diverging inference variable
   --> src/uar/realtime/surreal_bus.rs:145:40
    |
145 | ...                   if changed.is_err() { break false; }
    |                                  ^^^^^^
    |
    = help: consider providing a type annotation
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #156047 <https://github.com/rust-lang/rust/issues/156047>
    = note: `#[warn(method_call_on_diverging_infer_var)]` (part of `#[warn(future_incompatible)]`) on by default


error[E0599]: no method named `is_err` found for type `_` in the current scope
   --> src/uar/realtime/surreal_bus.rs:145:40
    |
145 | ...                   if changed.is_err() { break false; }
    |                                  ^^^^^^
    |
help: there is a method `is_dir` with a similar name
    |
145 -                             if changed.is_err() { break false; }
145 +                             if changed.is_dir() { break false; }
    |


error[E0599]: no method named `context` found for enum `std::result::Result<T, E>` in the current scope
   --> src/uar/realtime/postgres_bus.rs:87:35
    |
 87 |                     return result.context("PostgreSQL listener cleanup failed during shutdown");
    |                                   ^^^^^^^ method not found in `std::result::Result<(), anyhow::Error>`
    |
note: the method `context` exists on the type `anyhow::Error`
   --> /usr/local/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/error.rs:357:5
    |
357 | /     pub fn context<C>(self, context: C) -> Self
358 | |     where
359 | |         C: Display + Send + Sync + 'static,
    | |___________________________________________^
    = help: items from traits can only be used if the trait is in scope
help: use the `?` operator to extract the `anyhow::Error` value, propagating a `Result::Err` value to the caller
    |
 87 |                     return result?.context("PostgreSQL listener cleanup failed during shutdown");
    |                                  +
help: the following traits which provide `context` are implemented but not in scope; perhaps you want to import one of them
    |
  2 + use anyhow::Context;
    |
  2 + use quick_error::ResultExt;
    |
  2 + use surrealdb_expr::expr::ControlFlowExt;
    |


error[E0624]: method `spawn_owned_task` is private
   --> src/uar/runtime/instance/lifecycle.rs:343:22
    |
343 |                   self.spawn_owned_task("instance-cancel-after-admission", Box::pin(async move {
    |                        ^^^^^^^^^^^^^^^^ private method
    |
   ::: src/uar/runtime/instance/pump.rs:26:5
    |
 26 | /     fn spawn_owned_task(
 27 | |         &self,
 28 | |         task_name: &'static str,
 29 | |         job: Pin<Box<dyn Future<Output = ()> + Send>>,
 30 | |     ) {
    | |_____- private method defined here


error[E0282]: type annotations needed for `std::option::Option<_>`
  --> src/uar/runtime/schedules/controller.rs:53:30
   |
53 |             .try_fold(None, |next, record| {
   |                              ^^^^
...
56 |                     (Some(current), Some(candidate)) => Some(current.min(candidate)),
   |                                                              ------- type must be known at this point
   |
help: consider giving this closure parameter an explicit type, where the placeholder `_` is specified
   |
53 |             .try_fold(None, |next: std::option::Option<T>, record| {
   |                                  ++++++++++++++++++++++++


error[E0308]: mismatched types
    --> src/mcp/registry.rs:2640:74
     |
2640 |                         .map(|(name, service)| (name.clone(), Arc::clone(service)))
     |                                                               ---------- ^^^^^^^ expected `&Arc<_, _>`, found `&ClientService`
     |                                                               |
     |                                                               arguments to this function are incorrect
     |
     = note: expected reference `&std::sync::Arc<_, _>`
                found reference `&ClientService`
note: method defined here
    --> /usr/local/rustup/toolchains/nightly-2026-07-18-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/clone.rs:236:8
     |
 236 |     fn clone(&self) -> Self;
     |        ^^^^^


error[E0277]: a value of type `std::collections::HashMap<std::string::String, ClientService>` cannot be built from an iterator over elements of type `(std::string::String, std::sync::Arc<_, _>)`
    --> src/mcp/registry.rs:2641:26
     |
2641 |                         .collect(),
     |                          ^^^^^^^ value of type `std::collections::HashMap<std::string::String, ClientService>` cannot be built from `std::iter::Iterator<Item=(std::string::String, std::sync::Arc<_, _>)>`
     |
help: the trait `FromIterator<(String, std::sync::Arc<_, _>)>` is not implemented for `std::collections::HashMap<std::string::String, ClientService>`
      but trait `FromIterator<(String, ClientService)>` is implemented for it
    --> /usr/local/rustup/toolchains/nightly-2026-07-18-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:3005:1
     |
3005 | / impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
3006 | | where
3007 | |     K: Eq + Hash,
3008 | |     S: BuildHasher + Default,
     | |_____________________________^
     = help: for that trait implementation, expected `ClientService`, found `std::sync::Arc<_, _>`
note: the method call chain might not have had the expected associated types
    --> src/mcp/registry.rs:2640:26
     |
2636 |                       connections: bindings
     |  __________________________________-
2637 | |                         .connections
     | |____________________________________- this expression has type `HashMap<String, ClientService>`
2638 |                           .iter()
     |                            ------ `Iterator::Item` is `(&String, &ClientService)` here
2639 |                           .filter(|(name, _)| server_allowed(name))
     |                            ---------------------------------------- `Iterator::Item` remains `(&String, &ClientService)` here
2640 |                           .map(|(name, service)| (name.clone(), Arc::clone(service)))
     |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `Iterator::Item` changed to `(String, Arc<_, _>)` here
note: required by a bound in `std::iter::Iterator::collect`
    --> /usr/local/rustup/toolchains/nightly-2026-07-18-aarch64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2077:19
     |
2077 |     fn collect<B: FromIterator<Self::Item>>(self) -> B
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::collect`


error[E0521]: borrowed data escapes outside of method
   --> src/uar/persistence/scoped.rs:399:9
    |
396 |       async fn publish_ingestion(&self, _job: &ingestion_records::IngestionJob, _publication: &ingestion_records::IngestionPublicat...
    |                                                                                 ------------  - lifetime `'life2` defined here
    |                                                                                 |
    |                                                                                 `_publication` is a reference that is only valid in the method body
...
399 | /         self.dispatch("publish_ingestion", move |inner| async move {
400 | |                 inner.publish_ingestion(&_job, &_publication).await
401 | |             }).await
    | |              ^
    | |              |
    | |______________`_publication` escapes the method body here
    |                argument requires that `'life2` must outlive `'static`


error[E0521]: borrowed data escapes outside of method
    --> src/uar/persistence/scoped.rs:1320:9
     |
1318 |       async fn commit_supervisor_checkpoint_exclusion(&self, _proof: &crate::uar::runtime::supervisor::VerifiedSupervisorExclusion...
     |                                                              ------  - lifetime `'life1` defined here
     |                                                              |
     |                                                              `_proof` is a reference that is only valid in the method body
1319 |                   let _proof = _proof.clone();
1320 | /         self.dispatch("commit_supervisor_checkpoint_exclusion", move |inner| async move {
1321 | |                 inner.commit_supervisor_checkpoint_exclusion(&_proof).await
1322 | |             }).await
     | |              ^
     | |              |
     | |______________`_proof` escapes the method body here
     |                argument requires that `'life1` must outlive `'static`


error: future cannot be sent between threads safely
   --> src/uar/rag/embeddings/failover.rs:649:14
    |
649 |             .spawn("embedding-recover-primary", async move {
    |              ^^^^^ future created by async block is not `Send`
    |
    = help: within `{async block@src/uar/rag/embeddings/failover.rs:649:49: 649:59}`, the trait `std::marker::Send` is not implemented for `std::sync::MutexGuard<'_, FailoverState>`
note: future is not `Send` as this value is used across an await
   --> src/uar/rag/embeddings/failover.rs:583:56
    |
577 |         let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    |             --------- has type `std::sync::MutexGuard<'_, FailoverState>` which is not `Send`
...
583 |             let candidate_cleanup = candidate.retire().await;
    |                                                        ^^^^^ await occurs here, with `mut state` maybe used later
note: required by a bound in `ResourceScope::spawn`
   --> src/uar/runtime/lifecycle.rs:732:50
    |
730 |     pub fn spawn<F>(&self, task_name: impl Into<String>, future: F) -> Result<(), AdmissionClosed>
    |            ----- required by a bound in this associated function
731 |     where
732 |         F: Future<Output = anyhow::Result<()>> + Send + 'static,
    |                                                  ^^^^ required by this bound in `ResourceScope::spawn`

warning: variable does not need to be mutable
   --> src/uar/runtime/lifecycle.rs:925:23
    |
925 |     pub fn unresolved(mut self, effect: impl Into<String>) {
    |                       ----^^^^
    |                       |
    |                       help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

Some errors have detailed explanations: E0277, E0282, E0308, E0382, E0433, E0521, E0599, E0624.
For more information about an error, try `rustc --explain E0277`.
warning: `universal-agent-runtime` (lib) generated 8 warnings
```

