//! Runtime-free admission policy shared by durable ingestion and its callers.

use prometheus_parking_lot_core::{
    SchedulerError,
    dispatch::{Admission, Dispatched, Dispatcher, Parkable},
    semaphore::{Permit, UnitSemaphore},
};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Strict FIFO admission with a separately bounded parked queue.
///
/// Grants own capacity until the host joins execution and settles its durable
/// outcome. Closing admission returns descriptors, never document source bytes.
pub struct IngestionAdmission<T: Parkable> {
    dispatcher: Dispatcher<T>,
    closed: Mutex<bool>,
}

impl<T: Parkable> IngestionAdmission<T> {
    /// Construct a positive slot capacity and an independently bounded queue.
    pub fn new(slots: usize, max_queue_depth: usize) -> Result<Self, SchedulerError> {
        let slots = u32::try_from(slots)
            .ok().filter(|slots| *slots > 0)
            .ok_or_else(|| SchedulerError::invalid_config("ingestion slot count must fit a positive u32"))?;
        Ok(Self {
            dispatcher: Dispatcher::in_memory("uar-ingestion", UnitSemaphore::new(slots), max_queue_depth)
                .with_max_head_skips(0),
            closed: Mutex::new(false),
        })
    }

    /// Arrange a host wake when an exceptional grant drop returns capacity.
    pub fn install_release_wake(&self, wake: Arc<Notify>) {
        self.dispatcher.install_release_hook(|| move || wake.notify_one());
    }

    /// Admit or park a descriptor; oversized demand is rejected before queuing.
    pub fn submit(&self, item: T, now_ms: u128) -> Result<Admission<T>, SchedulerError> {
        let closed = self.closed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *closed {
            return Err(SchedulerError::PoolShutdown { pool: "uar-ingestion".to_owned() });
        }
        self.dispatcher.submit(item, now_ms)
    }

    /// Return capacity after actual completion and admit the next descriptors.
    pub fn complete(&self, permit: Permit, now_ms: u128) -> Dispatched<T> {
        let closed = self.closed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *closed { drop(permit); Dispatched::default() }
        else { self.dispatcher.complete(permit, now_ms) }
    }

    /// Drain after an exceptional permit drop without a new submission.
    pub fn poke(&self, now_ms: u128) -> Dispatched<T> {
        let closed = self.closed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if *closed { Dispatched::default() } else { self.dispatcher.poke(now_ms) }
    }

    /// Close once and remove waiting descriptors for later durable discovery.
    pub fn close(&self) -> Vec<T> {
        let mut closed = self.closed.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *closed = true;
        self.dispatcher.take_parked()
    }

    /// Number of granted slots whose permits have not returned.
    pub fn active_jobs(&self) -> usize { self.dispatcher.running() as usize }

    /// Number of lightweight descriptors waiting for a slot.
    pub fn waiting_jobs(&self) -> usize { self.dispatcher.parked() }
}
