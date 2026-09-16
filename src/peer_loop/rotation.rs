//! Spreads a per-identity poll evenly over its interval instead of queuing
//! every identity in one burst. A burst of N identities x several peers hits
//! the request backlog cap, and since the identities come in table order the
//! same ones get dropped every round. Taking a few per pass, round robin,
//! refreshes each identity once per interval whatever the wallet size.

use std::time::Duration;

pub struct Rotation {
    cursor: usize,
    /// Poll time earned but not yet spent, in identity-milliseconds: one
    /// identity is due per `interval` of it. Integer, so rounds come out exact.
    credit_ms: u128,
    /// Whether the first list has been seen: it is polled in full right away,
    /// so a freshly started wallet shows balances without waiting a round.
    primed: bool,
}

impl Rotation {
    pub fn new() -> Self {
        Rotation { cursor: 0, credit_ms: 0, primed: false }
    }

    /// The identities due this pass: enough that every one of `list` comes up
    /// once per `interval`, given `elapsed` since the last pass.
    pub fn take(&mut self, list: &[String], interval: Duration, elapsed: Duration) -> Vec<String> {
        if list.is_empty() {
            self.cursor = 0;
            self.credit_ms = 0;
            return vec![];
        }
        let interval_ms = interval.as_millis().max(1);
        if !self.primed {
            self.primed = true;
            self.credit_ms = list.len() as u128 * interval_ms;
        }
        // A stall (no ticks for a while) must not turn into one giant burst
        // afterwards: never owe more than a full round.
        self.credit_ms = (self.credit_ms + list.len() as u128 * elapsed.as_millis()).min(list.len() as u128 * interval_ms);
        let due = (self.credit_ms / interval_ms) as usize;
        self.credit_ms %= interval_ms;
        let mut out = Vec::with_capacity(due);
        for _ in 0..due {
            self.cursor %= list.len();
            out.push(list[self.cursor].clone());
            self.cursor += 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("id{}", i)).collect()
    }

    /// A rotation past its first pass, which polls the whole list at once.
    fn primed(list: &[String]) -> Rotation {
        let mut rotation = Rotation::new();
        assert_eq!(rotation.take(list, Duration::from_secs(10), Duration::from_secs(1)), list, "the first pass covers everything");
        rotation
    }

    #[test]
    fn every_identity_once_per_interval() {
        let list = ids(20);
        let mut rotation = primed(&list);
        let mut seen: Vec<String> = vec![];
        for _ in 0..10 {
            seen.extend(rotation.take(&list, Duration::from_secs(10), Duration::from_secs(1)));
        }
        assert_eq!(seen, list, "ten one-second passes cover the whole list exactly once");
    }

    #[test]
    fn small_wallets_are_not_polled_faster() {
        let list = ids(3);
        let mut rotation = primed(&list);
        let mut taken = 0;
        for _ in 0..30 {
            taken += rotation.take(&list, Duration::from_secs(10), Duration::from_secs(1)).len();
        }
        assert_eq!(taken, 9, "three identities over 30 s is three rounds");
    }

    #[test]
    fn stall_is_capped_to_one_round() {
        let list = ids(5);
        let mut rotation = primed(&list);
        let burst = rotation.take(&list, Duration::from_secs(10), Duration::from_secs(600));
        assert_eq!(burst.len(), 5);
        assert!(rotation.take(&list, Duration::from_secs(10), Duration::from_secs(1)).is_empty());
    }

    #[test]
    fn empty_list_resets() {
        let mut rotation = Rotation::new();
        assert!(rotation.take(&[], Duration::from_secs(10), Duration::from_secs(1)).is_empty());
        let list = ids(2);
        assert_eq!(rotation.take(&list, Duration::from_secs(10), Duration::from_secs(1)), list);
    }
}
