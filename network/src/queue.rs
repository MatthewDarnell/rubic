//! The work queue shared by every peer's worker thread.
//!
//! Requests are taken from the front; routine requests join at the back and
//! urgent ones (transaction broadcasts) jump to the front, so a transaction the
//! user just signed goes out on the next free socket instead of waiting behind
//! a backlog of tick polls and order-book fetches. That matters most for RANDOM
//! rounds, whose reveal must reach the tick leader a few ticks ahead of time.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use api::request::QubicApiPacket;

pub struct RequestQueue {
    items: Mutex<VecDeque<QubicApiPacket>>,
    ready: Condvar,
}

impl RequestQueue {
    pub fn new() -> Self {
        RequestQueue { items: Mutex::new(VecDeque::new()), ready: Condvar::new() }
    }

    /// Routine request: served after everything already queued.
    pub fn push_back(&self, request: QubicApiPacket) {
        self.items.lock().unwrap().push_back(request);
        self.ready.notify_one();
    }

    /// Urgent request: served before everything already queued.
    pub fn push_front(&self, request: QubicApiPacket) {
        self.items.lock().unwrap().push_front(request);
        self.ready.notify_one();
    }

    /// The next request, waiting up to `timeout` for one to arrive.
    pub fn pop(&self, timeout: Duration) -> Option<QubicApiPacket> {
        let mut items = self.items.lock().unwrap();
        if items.is_empty() {
            let (guard, _) = self.ready.wait_timeout(items, timeout).unwrap();
            items = guard;
        }
        items.pop_front()
    }
}

impl Default for RequestQueue {
    fn default() -> Self {
        Self::new()
    }
}
