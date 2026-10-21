//! Embedded workers register before execution and retain their terminal outcomes.

use std::future::Future;
use std::sync::{Arc, Mutex};
use futures::FutureExt as _;
use surrealdb_types::Error;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[derive(Clone, Default)]
pub(crate) struct TaskScope(Arc<Scope>);

#[derive(Default)]
struct Scope {
	closed: Mutex<bool>,
	tasks: TaskTracker,
	errors: Mutex<Vec<String>>,
	stop: CancellationToken,
}

impl TaskScope {
	pub(crate) fn stop(&self) -> CancellationToken { self.0.stop.clone() }
	pub(crate) fn cancel(&self) { self.0.stop.cancel(); }

	#[cfg(not(target_family = "wasm"))]
	pub(crate) fn spawn<F>(&self, name: &'static str, future: F) -> Result<(), Error>
	where F: Future<Output = Result<(), Error>> + Send + 'static {
		let closed = self.0.closed.lock().expect("local task admission poisoned");
		if *closed { return Err(Error::internal("embedded task admission closed".into())); }
		let scope = self.clone();
		let tracked = self.0.tasks.track_future(async move {
			scope.record(name, std::panic::AssertUnwindSafe(future).catch_unwind().await);
		});
		drop(tokio::spawn(tracked));
		Ok(())
	}

	#[cfg(target_family = "wasm")]
	pub(crate) fn spawn<F>(&self, name: &'static str, future: F) -> Result<(), Error>
	where F: Future<Output = Result<(), Error>> + 'static {
		let closed = self.0.closed.lock().expect("local task admission poisoned");
		if *closed { return Err(Error::internal("embedded task admission closed".into())); }
		let scope = self.clone();
		let tracked = self.0.tasks.track_future(async move {
			scope.record(name, std::panic::AssertUnwindSafe(future).catch_unwind().await);
		});
		wasm_bindgen_futures::spawn_local(tracked);
		Ok(())
	}

	fn record(&self, name: &str, outcome: std::thread::Result<Result<(), Error>>) {
		let error = match outcome {
			Ok(Ok(())) => return,
			Ok(Err(error)) => format!("{name}: {error}"),
			Err(_) => format!("{name}: worker panicked"),
		};
		self.0.errors.lock().expect("local task outcomes poisoned").push(error);
	}

	pub(crate) async fn close(&self) -> Vec<String> {
		{
			let mut closed = self.0.closed.lock().expect("local task admission poisoned");
			*closed = true;
			self.0.tasks.close();
		}
		self.0.tasks.wait().await;
		self.0.errors.lock().expect("local task outcomes poisoned").clone()
	}
}
