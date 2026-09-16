//! The work queue shared by every peer's worker thread.
//!
//! Routine requests sit in one shared line, taken from the front by whichever
//! worker is free. A transaction's first broadcast is different: every peer
//! must send it, and as its very next request. Those copies are therefore
//! addressed to a peer and kept in that peer's own lane, which its worker
//! drains before touching the shared line. That matters most for RANDOM
//! session steps, whose reveal must reach the tick leader a few ticks ahead
//! of time, and it guarantees no peer is skipped while another sends twice.

use std::collections::{HashMap, VecDeque};
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use api::request::QubicApiPacket;

struct Lanes {
    shared: VecDeque<QubicApiPacket>,
    /// Per peer id: requests only that peer may send, ahead of the shared line.
    addressed: HashMap<String, VecDeque<QubicApiPacket>>,
}

pub struct RequestQueue {
    lanes: Mutex<Lanes>,
    ready: Condvar,
}

impl RequestQueue {
    pub fn new() -> Self {
        RequestQueue {
            lanes: Mutex::new(Lanes { shared: VecDeque::new(), addressed: HashMap::new() }),
            ready: Condvar::new(),
        }
    }

    /// Routine request: served after everything already in the shared line.
    pub fn push_back(&self, request: QubicApiPacket) {
        self.lanes.lock().unwrap().shared.push_back(request);
        self.ready.notify_one();
    }

    /// Served before everything already in the shared line, by any free worker.
    pub fn push_front(&self, request: QubicApiPacket) {
        self.lanes.lock().unwrap().shared.push_front(request);
        self.ready.notify_one();
    }

    /// For one peer only, ahead of anything in the shared line.
    pub fn push_urgent(&self, peer: &str, request: QubicApiPacket) {
        self.lanes.lock().unwrap().addressed.entry(peer.to_string()).or_default().push_back(request);
        // The peer's own worker must wake, not just any worker.
        self.ready.notify_all();
    }

    /// The next request for `peer`'s worker: its own lane first, then the shared
    /// line, waiting up to `timeout` for something to arrive.
    pub fn pop(&self, peer: &str, timeout: Duration) -> Option<QubicApiPacket> {
        let mut lanes = self.lanes.lock().unwrap();
        for waited in [false, true] {
            if let Some(request) = lanes.addressed.get_mut(peer).and_then(|lane| lane.pop_front()) {
                return Some(request);
            }
            if let Some(request) = lanes.shared.pop_front() {
                return Some(request);
            }
            if waited {
                break;
            }
            let (guard, _) = self.ready.wait_timeout(lanes, timeout).unwrap();
            lanes = guard;
        }
        None
    }

    /// Drops what was addressed to a peer that is gone; returns how many.
    pub fn discard(&self, peer: &str) -> usize {
        self.lanes.lock().unwrap().addressed.remove(peer).map(|lane| lane.len()).unwrap_or(0)
    }
}

impl Default for RequestQueue {
    fn default() -> Self {
        Self::new()
    }
}
