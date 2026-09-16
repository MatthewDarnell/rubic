//! Wire format of the RANDOM contract's `RevealAndCommit` procedure, from
//! `src/contracts/Random.h` in the Qubic core repository.
//!
//! The input is always `bit_4096 reveal` (512 bytes) followed by `id commit`
//! (32 bytes). The transaction amount selects the collateral tier and must be
//! exactly 1, 10, 100 … 1 000 000 000 QU; anything else is refunded and ignored.
//! Providers belong to stream `tick % 3` and are expected at every tick of
//! their stream: each call reveals the previous secret and commits the next
//! one, and a call with an empty commit reveals and leaves.
//!
//! A session is a chain of such calls at ticks T0, T0 + 3, T0 + 6 …:
//! - step 0: reveal nothing, commit K12(d1)
//! - step k: reveal d(k), commit K12(d(k+1))
//! - leaving at step k: reveal d(k), commit nothing
//! Every step's transaction (and its leaving variant) is signed when the session
//! starts, so no seed has to be kept while the session runs.

use crypto::hash::k12_bytes;
use crypto::random::random_bytes;

/// Identity of the RANDOM contract (contract index 3).
pub const RANDOM_CONTRACT_IDENTITY: &str = "DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANMIG";
pub const RANDOM_CONTRACT_INDEX: u32 = 3;
/// `REGISTER_USER_PROCEDURE(RevealAndCommit, 1)`.
pub const REVEAL_AND_COMMIT: u16 = 1;
/// `REGISTER_USER_FUNCTION(GetProviderStatus, 2)`: input is the provider `id`.
pub const GET_PROVIDER_STATUS: u16 = 2;
/// `GetProviderStatus_output`: count, then five arrays of 32 entries. The
/// `uint64` collateral array is 8-byte aligned, so 4 padding bytes follow the
/// tier array and the reply is 680 bytes, not 676.
pub const PROVIDER_STATUS_LEN: usize = 680;
const PS_STREAMS: usize = 4;
const PS_TIERS: usize = 4 + 128;
const PS_LOCKED: usize = 264;
const PS_CONTRIBUTED: usize = 264 + 256;
const PS_LAST_UPDATE: usize = 264 + 256 + 32;
/// `bit_4096`: a reveal is exactly this many bytes.
pub const REVEAL_LEN: usize = 512;
/// `id`: the commit is a 32-byte K12 digest.
pub const COMMIT_LEN: usize = 32;
/// Streams rotate with `tick % 3`; a provider acts once per stream tick.
pub const STREAM_TICKS: u32 = 3;
/// The collateral tiers the contract accepts, in QU.
pub const TIERS: [u64; 10] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000];
/// Steps signed ahead when a session starts; the last one always leaves.
pub const MAX_STEPS: u32 = 1000;

pub fn is_tier(amount: u64) -> bool {
    TIERS.contains(&amount)
}

/// One (stream, tier) slot the contract holds for a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSlot {
    pub stream: u32,
    pub tier: u32,
    pub locked: u64,
    pub contributed: bool,
    /// Tick of the last `RevealAndCommit` accepted for this slot.
    pub last_update_tick: u32,
}

impl ProviderSlot {
    pub fn tier_amount(&self) -> u64 {
        TIERS.get(self.tier as usize).copied().unwrap_or(0)
    }

    /// Compact text form for storage: `stream:tier:locked:contributed:last;...`.
    pub fn list_to_text(slots: &[ProviderSlot]) -> String {
        slots.iter()
            .map(|s| format!("{}:{}:{}:{}:{}", s.stream, s.tier, s.locked, u8::from(s.contributed), s.last_update_tick))
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn parse_list(text: &str) -> Vec<ProviderSlot> {
        text.split(';').filter(|t| !t.is_empty()).filter_map(|t| {
            let f: Vec<&str> = t.split(':').collect();
            if f.len() != 5 { return None; }
            Some(ProviderSlot {
                stream: f[0].parse().ok()?,
                tier: f[1].parse().ok()?,
                locked: f[2].parse().ok()?,
                contributed: f[3] == "1",
                last_update_tick: f[4].parse().ok()?,
            })
        }).collect()
    }
}

/// Decodes a `GetProviderStatus` reply.
pub fn parse_provider_status(data: &[u8]) -> Option<Vec<ProviderSlot>> {
    if data.len() < PROVIDER_STATUS_LEN {
        return None;
    }
    let u32_at = |o: usize| u32::from_le_bytes(data[o..o + 4].try_into().unwrap());
    let u64_at = |o: usize| u64::from_le_bytes(data[o..o + 8].try_into().unwrap());
    let count = u32_at(0).min(32) as usize;
    Some((0..count).map(|k| ProviderSlot {
        stream: u32_at(PS_STREAMS + 4 * k),
        tier: u32_at(PS_TIERS + 4 * k),
        locked: u64_at(PS_LOCKED + 8 * k),
        contributed: data[PS_CONTRIBUTED + k] != 0,
        last_update_tick: u32_at(PS_LAST_UPDATE + 4 * k),
    }).collect())
}

/// A fresh 512-byte secret whose digest is committed.
pub fn new_secret() -> Vec<u8> {
    random_bytes(REVEAL_LEN as u32)
}

/// What the contract stores for a commit: K12 over the full 512-byte reveal.
pub fn commitment(secret: &[u8]) -> [u8; COMMIT_LEN] {
    let mut padded = secret.to_vec();
    padded.resize(REVEAL_LEN, 0);
    k12_bytes(&padded).as_slice().try_into().expect("K12 digest is 32 bytes")
}

/// The procedure input: `reveal` (empty for a first commit) padded to 512
/// bytes, then `commit` (`None` to leave).
pub fn reveal_and_commit_input(reveal: &[u8], commit: Option<&[u8; COMMIT_LEN]>) -> Vec<u8> {
    let mut input = reveal.to_vec();
    input.resize(REVEAL_LEN, 0);
    match commit {
        Some(digest) => input.extend_from_slice(digest),
        None => input.extend_from_slice(&[0u8; COMMIT_LEN]),
    }
    input
}

/// First call of a session: nothing to reveal, commit the digest.
pub fn commit_input(digest: &[u8; COMMIT_LEN]) -> Vec<u8> {
    reveal_and_commit_input(&[], Some(digest))
}

/// Reveal-and-leave: reveal the secret with an empty commit. The contract
/// refunds the locked collateral and drops the provider at the end of the tick.
pub fn reveal_and_leave_input(secret: &[u8]) -> Vec<u8> {
    reveal_and_commit_input(secret, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs_have_the_contract_layout() {
        let d1 = new_secret();
        let d2 = new_secret();
        assert_eq!(d1.len(), REVEAL_LEN);
        let first = commit_input(&commitment(&d1));
        assert_eq!(first.len(), REVEAL_LEN + COMMIT_LEN);
        assert!(first[..REVEAL_LEN].iter().all(|b| *b == 0));
        assert_eq!(&first[REVEAL_LEN..], &commitment(&d1));
        let step = reveal_and_commit_input(&d1, Some(&commitment(&d2)));
        assert_eq!(&step[..REVEAL_LEN], d1.as_slice());
        assert_eq!(&step[REVEAL_LEN..], &commitment(&d2));
        let leave = reveal_and_leave_input(&d1);
        assert_eq!(&leave[..REVEAL_LEN], d1.as_slice());
        assert!(leave[REVEAL_LEN..].iter().all(|b| *b == 0));
        // The contract checks K12(reveal) against the stored commit.
        assert_eq!(commitment(&step[..REVEAL_LEN]), commitment(&d1));
        assert!(is_tier(1) && is_tier(1_000_000_000) && !is_tier(5));
    }

    #[test]
    fn provider_status_round_trips() {
        let mut data = vec![0u8; PROVIDER_STATUS_LEN];
        data[0] = 2; // two slots
        data[4..8].copy_from_slice(&1u32.to_le_bytes()); // stream of slot 0
        data[132..136].copy_from_slice(&2u32.to_le_bytes()); // tier of slot 0
        data[264..272].copy_from_slice(&100u64.to_le_bytes()); // locked of slot 0 (after 4 padding bytes)
        data[520] = 1; // contributed
        data[552..556].copy_from_slice(&80_000_000u32.to_le_bytes()); // last update tick
        let slots = parse_provider_status(&data).unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0], ProviderSlot { stream: 1, tier: 2, locked: 100, contributed: true, last_update_tick: 80_000_000 });
        assert_eq!(slots[0].tier_amount(), 100);
        assert_eq!(ProviderSlot::parse_list(&ProviderSlot::list_to_text(&slots)), slots);
    }
}
