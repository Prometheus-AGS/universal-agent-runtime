//! Completion of the local transport driver includes its future's destructors.

use std::{future::Future, panic::AssertUnwindSafe, sync::{Arc, Mutex}};
use futures::FutureExt;
use tokio_util::task::TaskTracker;

#[derive(Debug, Default)]
pub(crate) struct DriverScope {
    tasks: TaskTracker,
    failures: Arc<Mutex<Vec<String>>>,
    stopping: tokio_util::sync::CancellationToken,
    admission_closed: Mutex<bool>,
}

#[derive(Debug)]
pub(crate) struct DriverScopeClosed;

impl DriverScope {
    /// A read-only delivery pump may stop locally during client closure.
    /// Unary operations and durable mutations continue under their own leases.
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn spawn_stream(&self, work: impl Future<Output = ()> + Send + 'static) {
        if self.try_spawn_stream(work).is_err() {
            self.failures.lock().unwrap_or_else(|error| error.into_inner())
                .push("delivery pump registered after driver admission closed".to_owned());
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn try_spawn_stream(&self, work: impl Future<Output = ()> + Send + 'static) -> Result<(), DriverScopeClosed> {
        let closed = self.admission_closed.lock().unwrap_or_else(|error| error.into_inner());
        if *closed { return Err(DriverScopeClosed); }
        let stopping = self.stopping.clone();
        self.spawn(async move {
            let work = work.fuse();
            let stop = stopping.cancelled().fuse();
            futures::pin_mut!(work, stop);
            futures::select_biased! { () = stop => {}, () = work => {} }
        });
        Ok(())
    }

    #[cfg(target_family = "wasm")]
    pub(crate) fn spawn_stream(&self, work: impl Future<Output = ()> + 'static) {
        if self.try_spawn_stream(work).is_err() {
            self.failures.lock().unwrap_or_else(|error| error.into_inner())
                .push("delivery pump registered after driver admission closed".to_owned());
        }
    }

    #[cfg(target_family = "wasm")]
    pub(crate) fn try_spawn_stream(&self, work: impl Future<Output = ()> + 'static) -> Result<(), DriverScopeClosed> {
        let closed = self.admission_closed.lock().unwrap_or_else(|error| error.into_inner());
        if *closed { return Err(DriverScopeClosed); }
        let stopping = self.stopping.clone();
        self.spawn(async move {
            let work = work.fuse();
            let stop = stopping.cancelled().fuse();
            futures::pin_mut!(work, stop);
            futures::select_biased! { () = stop => {}, () = work => {} }
        });
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn spawn(&self, work: impl Future<Output = ()> + Send + 'static) {
        let failures = Arc::clone(&self.failures);
        self.tasks.spawn(async move {
            if AssertUnwindSafe(work).catch_unwind().await.is_err() {
                failures.lock().unwrap_or_else(|error| error.into_inner()).push("transport driver panicked".to_owned());
            }
        });
    }

    #[cfg(target_family = "wasm")]
    pub(crate) fn spawn(&self, work: impl Future<Output = ()> + 'static) {
        let failures = Arc::clone(&self.failures);
        wasm_bindgen_futures::spawn_local(self.tasks.track_future(async move {
            if AssertUnwindSafe(work).catch_unwind().await.is_err() {
                failures.lock().unwrap_or_else(|error| error.into_inner()).push("transport driver panicked".to_owned());
            }
        }));
    }

    pub(crate) async fn join(&self) -> crate::Result<()> {
        {
            let mut closed = self.admission_closed.lock().unwrap_or_else(|error| error.into_inner());
            *closed = true;
            self.stopping.cancel();
            self.tasks.close();
        }
        self.tasks.wait().await;
        let failures = self.failures.lock().unwrap_or_else(|error| error.into_inner());
        if failures.is_empty() { Ok(()) } else { Err(crate::Error::internal(failures.join("; "))) }
    }
}
