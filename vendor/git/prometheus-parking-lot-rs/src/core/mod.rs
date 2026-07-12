//! Core scheduling abstractions and capacity accounting.

pub mod audit;
pub mod cancellation;
pub mod error;
pub mod executor;
pub mod hooks;
pub mod resource_pool;
#[cfg(feature = "tokio-runtime")]
pub mod shutdown;
pub mod worker_pool;

pub use audit::{build_audit_event, AuditEvent, AuditSink, InMemoryAuditSink, PostgresAuditSink};
pub use cancellation::{
    CancelReason, CancellationChannel, CancellationConfig, CancellationSink, CancelledTask,
    LoggingCancellationSink, RecordingCancellationSink, SinkOutcome,
};
pub use error::{AppResult, SchedulerError};
pub use executor::{TaskExecutor, TaskPayload, WorkerExecutor};
pub use hooks::{
    emit_cancelled, emit_completed, emit_rejected, emit_submitted, AuditHookAdapter,
    CancelledReason, Hook, HookBus, LifecycleEvent, RecordingHook, RejectionReason, TracingHook,
};
pub use resource_pool::{
    sync_wake_worker_loop, Mailbox, PoolLimits, ResourcePool, ScheduledTask, Spawn, TaskMetadata,
    TaskQueue, TaskStatus, WakeState,
};
#[cfg(feature = "tokio-runtime")]
pub use shutdown::{wait_for_signal, ShutdownHandle, ShutdownPolicy};
pub use worker_pool::{PoolError, PoolStats, WorkerPool};
