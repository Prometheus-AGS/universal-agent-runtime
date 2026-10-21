//! Retained closure and operation ownership for one SDK engine facade.
//!
//! A caller dropping an SDK future must not drop the database operation it
//! already admitted. This wrapper moves accepted work into a tracked task and
//! answers the caller through a one-shot channel. Closing first closes the
//! admission gate, then joins tracked operations, then asks the underlying
//! engine to release its local resources.

use std::{
	borrow::Cow,
	panic::AssertUnwindSafe,
	path::PathBuf,
	sync::{Arc, Mutex},
};

use async_channel::Sender;
use futures::FutureExt;
use surrealdb_engine_api::{
	DbExportConfig, EngineContext, EngineFuture, MlExportConfig, SurrealEngine,
};
use surrealdb_rpc::{QueryResult, QueryStreamItem, Token};
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use crate::types::{Array, Error, Notification, Object, Value, Variables};

#[derive(Debug)]
struct Admission {
	closed: bool,
}

#[derive(Debug, Default)]
struct CloseState {
	started: bool,
	result: Option<Result<(), String>>,
	waiters: Vec<Sender<Result<(), String>>>,
}

#[derive(Debug)]
struct Inner {
	engine: Arc<dyn SurrealEngine>,
	admission: Mutex<Admission>,
	tasks: TaskTracker,
	failures: Mutex<Vec<String>>,
	close: Mutex<CloseState>,
}

/// A [`SurrealEngine`] facade with explicit admission and retained closure.
#[derive(Debug, Clone)]
pub(crate) struct LifecycleEngine {
	inner: Arc<Inner>,
}

impl LifecycleEngine {
	pub(crate) fn new(engine: Arc<dyn SurrealEngine>) -> Self {
		Self {
			inner: Arc::new(Inner {
				engine,
				admission: Mutex::new(Admission {
					closed: false,
				}),
				tasks: TaskTracker::new(),
				failures: Mutex::new(Vec::new()),
				close: Mutex::new(CloseState::default()),
			}),
		}
	}

	fn execute<T, F, Fut>(&self, operation: &'static str, work: F) -> EngineFuture<'static, T>
	where
		T: Send + 'static,
		F: FnOnce(Arc<dyn SurrealEngine>) -> Fut + Send + 'static,
		Fut: std::future::Future<Output = Result<T, Error>> + Send + 'static,
	{
		let (sender, receiver) = async_channel::bounded(1);
		let admission = self
			.inner
			.admission
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);
		if admission.closed {
			return Box::pin(async { Err(Error::internal("database engine admission is closed".into())) });
		}
		let inner = Arc::clone(&self.inner);
		let task = async move {
			let result = AssertUnwindSafe(work(Arc::clone(&inner.engine)))
				.catch_unwind()
				.await;
			let result = match result {
				Ok(result) => result,
				Err(_) => {
					inner
						.failures
						.lock()
						.unwrap_or_else(std::sync::PoisonError::into_inner)
						.push(format!("database engine operation {operation} panicked"));
					Err(Error::internal(format!("database engine operation {operation} panicked")))
				}
			};
			let _ = sender.send(result).await;
		};
		#[cfg(not(target_family = "wasm"))]
		self.inner.tasks.spawn(task);
		#[cfg(target_family = "wasm")]
		wasm_bindgen_futures::spawn_local(self.inner.tasks.track_future(task));
		drop(admission);
		Box::pin(async move {
			receiver
				.recv()
				.await
				.map_err(|_| Error::internal(format!("database engine operation {operation} ended without a result")))?
		})
	}

	fn close(&self) -> EngineFuture<'static, ()> {
		let inner = Arc::clone(&self.inner);
		Box::pin(async move { Inner::close(inner).await })
	}
}

impl Inner {
	async fn close(inner: Arc<Self>) -> Result<(), Error> {
		let (sender, receiver, start) = {
			let mut close = inner
				.close
				.lock()
				.unwrap_or_else(std::sync::PoisonError::into_inner);
			if let Some(result) = &close.result {
				return result.clone().map_err(Error::internal);
			}
			let (sender, receiver) = async_channel::bounded(1);
			close.waiters.push(sender.clone());
			let start = !close.started;
			close.started = true;
			(sender, receiver, start)
		};
		if start {
			Inner::spawn_finalizer(Arc::clone(&inner));
		}
		drop(sender);
		receiver
			.recv()
			.await
			.map_err(|_| Error::internal("database engine shutdown ended without a result".into()))?
			.map_err(Error::internal)
	}

	fn spawn_finalizer(inner: Arc<Self>) {
		#[cfg(not(target_family = "wasm"))]
		tokio::spawn(async move { Inner::finalize(inner).await });
		#[cfg(target_family = "wasm")]
		wasm_bindgen_futures::spawn_local(async move { Inner::finalize(inner).await });
	}

	async fn finalize(inner: Arc<Self>) {
		{
			let mut admission = inner
				.admission
				.lock()
				.unwrap_or_else(std::sync::PoisonError::into_inner);
			admission.closed = true;
			inner.tasks.close();
		}
		inner.tasks.wait().await;
		let engine_result = match AssertUnwindSafe(inner.engine.shutdown()).catch_unwind().await {
			Ok(Ok(())) => Ok(()),
			Ok(Err(error)) => Err(error.to_string()),
			Err(_) => Err("database engine shutdown panicked".into()),
		};
		let mut failures = inner
			.failures
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner)
			.clone();
		if let Err(error) = engine_result {
			failures.push(error);
		}
		let result = if failures.is_empty() {
			Ok(())
		} else {
			Err(failures.join("; "))
		};
		let waiters = {
			let mut close = inner
				.close
				.lock()
				.unwrap_or_else(std::sync::PoisonError::into_inner);
			close.result = Some(result.clone());
			std::mem::take(&mut close.waiters)
		};
		for waiter in waiters {
			let _ = waiter.send(result.clone()).await;
		}
	}
}

impl SurrealEngine for LifecycleEngine {
	fn query(&self, ctx: EngineContext, query: Cow<'static, str>, variables: Variables) -> EngineFuture<'_, Vec<QueryResult>> {
		self.execute("query", move |engine| async move { engine.query(ctx, query, variables).await })
	}

	fn query_stream(&self, ctx: EngineContext, query: Cow<'static, str>, variables: Variables, items: Sender<QueryStreamItem>) -> EngineFuture<'_, ()> {
		self.execute("query_stream", move |engine| async move { engine.query_stream(ctx, query, variables, items).await })
	}

	fn run(&self, ctx: EngineContext, name: String, version: Option<String>, args: Array) -> EngineFuture<'_, Value> {
		self.execute("run", move |engine| async move { engine.run(ctx, name, version, args).await })
	}

	fn use_ns_db(&self, ctx: EngineContext, namespace: Option<String>, database: Option<String>) -> EngineFuture<'_, (Option<String>, Option<String>)> {
		self.execute("use_ns_db", move |engine| async move { engine.use_ns_db(ctx, namespace, database).await })
	}

	fn set(&self, ctx: EngineContext, key: String, value: Value) -> EngineFuture<'_, ()> {
		self.execute("set", move |engine| async move { engine.set(ctx, key, value).await })
	}

	fn unset(&self, ctx: EngineContext, key: String) -> EngineFuture<'_, ()> {
		self.execute("unset", move |engine| async move { engine.unset(ctx, key).await })
	}

	fn signup(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
		self.execute("signup", move |engine| async move { engine.signup(ctx, credentials).await })
	}

	fn signin(&self, ctx: EngineContext, credentials: Object) -> EngineFuture<'_, Token> {
		self.execute("signin", move |engine| async move { engine.signin(ctx, credentials).await })
	}

	fn authenticate(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
		self.execute("authenticate", move |engine| async move { engine.authenticate(ctx, token).await })
	}

	fn refresh(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, Token> {
		self.execute("refresh", move |engine| async move { engine.refresh(ctx, token).await })
	}

	fn revoke(&self, ctx: EngineContext, token: Token) -> EngineFuture<'_, ()> {
		self.execute("revoke", move |engine| async move { engine.revoke(ctx, token).await })
	}

	fn invalidate(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
		self.execute("invalidate", move |engine| async move { engine.invalidate(ctx).await })
	}

	fn begin(&self, ctx: EngineContext) -> EngineFuture<'_, Uuid> {
		self.execute("begin", move |engine| async move { engine.begin(ctx).await })
	}

	fn commit(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
		self.execute("commit", move |engine| async move { engine.commit(ctx, txn).await })
	}

	fn rollback(&self, ctx: EngineContext, txn: Uuid) -> EngineFuture<'_, ()> {
		self.execute("rollback", move |engine| async move { engine.rollback(ctx, txn).await })
	}

	fn health(&self, ctx: EngineContext) -> EngineFuture<'_, ()> {
		self.execute("health", move |engine| async move { engine.health(ctx).await })
	}

	fn version(&self, ctx: EngineContext) -> EngineFuture<'_, String> {
		self.execute("version", move |engine| async move { engine.version(ctx).await })
	}

	fn shutdown(&self) -> EngineFuture<'_, ()> {
		self.close()
	}

	fn subscribe_live(&self, ctx: EngineContext, uuid: Uuid, notifications: Sender<Result<Notification, Error>>) -> EngineFuture<'_, ()> {
		self.execute("subscribe_live", move |engine| async move { engine.subscribe_live(ctx, uuid, notifications).await })
	}

	fn kill(&self, ctx: EngineContext, uuid: Uuid) -> EngineFuture<'_, ()> {
		self.execute("kill", move |engine| async move { engine.kill(ctx, uuid).await })
	}

	fn export_file(&self, ctx: EngineContext, path: PathBuf, config: Option<DbExportConfig>) -> EngineFuture<'_, ()> {
		self.execute("export_file", move |engine| async move { engine.export_file(ctx, path, config).await })
	}

	fn export_bytes(&self, ctx: EngineContext, bytes: Sender<Result<Vec<u8>, Error>>, config: Option<DbExportConfig>) -> EngineFuture<'_, ()> {
		self.execute("export_bytes", move |engine| async move { engine.export_bytes(ctx, bytes, config).await })
	}

	fn export_ml_file(&self, ctx: EngineContext, path: PathBuf, config: MlExportConfig) -> EngineFuture<'_, ()> {
		self.execute("export_ml_file", move |engine| async move { engine.export_ml_file(ctx, path, config).await })
	}

	fn export_ml_bytes(&self, ctx: EngineContext, bytes: Sender<Result<Vec<u8>, Error>>, config: MlExportConfig) -> EngineFuture<'_, ()> {
		self.execute("export_ml_bytes", move |engine| async move { engine.export_ml_bytes(ctx, bytes, config).await })
	}

	fn import_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
		self.execute("import_file", move |engine| async move { engine.import_file(ctx, path).await })
	}

	fn import_ml_file(&self, ctx: EngineContext, path: PathBuf) -> EngineFuture<'_, ()> {
		self.execute("import_ml_file", move |engine| async move { engine.import_ml_file(ctx, path).await })
	}
}
