//! Drives RANDOM provider sessions. Every step of a session was signed when it
//! started (see `random_session`); this loop puts each step on the wire a few
//! ticks ahead of its tick, chooses the leaving variant once a stop was
//! requested, and settles the session against what the contract itself
//! reports through `GetProviderStatus`. It also rebuilds a step's transaction
//! for the broadcaster's resends.
//!
//! Inclusion in a tick is not acceptance: a step the contract rejects is still
//! a valid transaction, it just refunds its amount. The contract's
//! `lastUpdateTick` for the provider's slot is the one signal that says which
//! step was the last one accepted, and a slot that vanishes while the session
//! runs means the provider was evicted (a missed reveal, stake burned).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use api::customtransfer::CustomTransferTransaction;
use crypto::qubic_identities::get_public_key_from_identity;
use logger::{error, info};
use miner::random::{commitment, new_secret, reveal_and_commit_input, ProviderSlot, GET_PROVIDER_STATUS, MAX_STEPS, RANDOM_CONTRACT_INDEX, RANDOM_CONTRACT_IDENTITY, REVEAL_AND_COMMIT, STREAM_TICKS};
use std::collections::HashSet;
use network::peers::PeerSet;
use once_cell::sync::Lazy;
use protocol::identity::Identity;
use store::get_db_path;
use store::sqlite::random_session::{self, NewStep, STATUS_LEAVING, STATUS_RUNNING, STATUS_STOPPED, STATUS_STOP_REQUESTED};
use store::sqlite::{tick, transfer};

/// Seeds of sessions started with "keep mining": a failed session is replaced
/// by a fresh one, and that needs a signature. Held only while such a session
/// is active, and gone with the process, so a restart of Rubic ends the chain.
static SESSION_SEEDS: Lazy<Mutex<HashMap<i64, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
/// A "keep mining" chain stops after this many failures in a row without a
/// single accepted step: the identity is not getting in, and each attempt
/// locks a stake.
const MAX_FAIL_STREAK: u32 = 3;
/// Ticks ahead of the latest known tick a session's first commit is scheduled.
pub const FIRST_COMMIT_LEAD_TICKS: u32 = 30;
/// How often the step transfers are settled from the contract's report.
const SETTLE_INTERVAL: Duration = Duration::from_secs(5);
/// Ticks after a leave's tick to wait for peers to report whether the leave
/// was included before the session is given up as evicted.
const LEAVE_VERIFY_TICKS: u32 = 30;
/// Peers that must agree before a session is ended on the contract's report.
/// One peer serving stale state would otherwise halt a healthy chain, and the
/// provider then really is evicted for the steps that stop going out.
const CORROBORATING_PEERS: usize = 2;
/// A report dated further behind the latest tick than this says nothing about
/// the present.
const REPORT_MAX_AGE_TICKS: u32 = 60;
/// How often a balance report is asked for while a leave is being verified.
const LEAVE_CHECK_INTERVAL: Duration = Duration::from_secs(3);
/// Identities whose provider status should be fetched on the next pass, asked
/// for by the start route's pre-flight check.
static STATUS_CHECKS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Asks the driver to fetch the contract's provider status for an identity on
/// its next pass (within about half a second plus a round trip).
pub fn queue_provider_status_check(identity: &str) {
    STATUS_CHECKS.lock().unwrap().insert(identity.to_string());
}

const PASS_INTERVAL: Duration = Duration::from_millis(500);
/// A step is broadcast once its tick is at most this far ahead of the latest
/// known tick: early enough to survive a stalled tick view, late enough that a
/// stop request takes effect within a few steps.
const BROADCAST_LEAD_TICKS: u32 = 24;
/// The tick leader publishes a tick's transactions two ticks early; a step
/// this close to its tick can no longer make it.
const TOO_LATE_TICKS: u32 = 2;
/// How often the contract is asked about an active session's provider slot.
const PROVIDER_STATUS_INTERVAL: Duration = Duration::from_secs(3);
/// The step table is trimmed to this many rows (about 10 sessions' worth),
/// oldest finished sessions first, checked this often.
const MAX_STEP_ROWS: u64 = 10_000;
const PRUNE_INTERVAL: Duration = Duration::from_secs(60);

fn get(row: &HashMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}

fn num<T: std::str::FromStr>(row: &HashMap<String, String>, key: &str) -> Option<T> {
    row.get(key)?.parse().ok()
}

/// The stay or leave transaction of a stored step, signature included.
fn step_transaction(session: &HashMap<String, String>, step: &HashMap<String, String>, leave: bool) -> Option<CustomTransferTransaction> {
    let source = get_public_key_from_identity(&get(session, "identity")).ok()?;
    let dest = get_public_key_from_identity(&RANDOM_CONTRACT_IDENTITY.to_string()).ok()?;
    let amount: u64 = num(session, "tier")?;
    let tick: u32 = num(step, "tick")?;
    let reveal = hex::decode(get(step, "reveal_secret")).ok()?;
    let (sig_key, commit) = if leave {
        ("leave_sig", None)
    } else {
        let next = hex::decode(get(step, "commit_secret")).ok()?;
        ("stay_sig", Some(commitment(&next)))
    };
    let sig = hex::decode(get(step, sig_key)).ok()?;
    if sig.is_empty() {
        return None;
    }
    Some(CustomTransferTransaction::from_signed_parts(&source, &dest, amount, tick, REVEAL_AND_COMMIT, &reveal_and_commit_input(&reveal, commit.as_ref()), &sig))
}

/// The transaction of a session step with this id, if the id belongs to one.
pub fn rebuild_step_transaction(txid: &str) -> Option<CustomTransferTransaction> {
    let path = get_db_path();
    let step = random_session::fetch_step_by_txid(path.as_str(), txid).ok()??;
    let session = random_session::fetch_session(path.as_str(), num(&step, "session_id")?).ok()??;
    let leave = get(&step, "leave_txid") == txid;
    step_transaction(&session, &step, leave)
}

/// Status of a transfer row: "-1" pending, "0" confirmed, "1" failed.
fn transfer_status(txid: &str) -> Option<String> {
    transfer::fetch_transfer_by_txid(get_db_path().as_str(), txid).ok()?
        .into_iter().next()?
        .get("status").cloned()
}

/// Records the step's transaction as a transfer: the broadcaster sends it at
/// once (ahead of the queue) and resends it now and then until its tick passes.
fn broadcast_step(session: &HashMap<String, String>, step: &HashMap<String, String>, leave: bool) -> Result<(), String> {
    let tx = step_transaction(session, step, leave).ok_or("step has no signed transaction")?;
    let txid = get(step, if leave { "leave_txid" } else { "stay_txid" });
    transfer::create_transfer(get_db_path().as_str(), &get(session, "identity"), RANDOM_CONTRACT_IDENTITY, tx._amount, tx._tick, &hex::encode(&tx._signature), &txid)
}

/// Signs a whole session (first commit plus `steps` reveal steps, each with a
/// stay and a leave variant), records it, and puts the first commit on the
/// wire. Returns the session id. With `auto_restart` the seed is kept in
/// memory so a failed session can be replaced by a new one.
pub fn start_session(id: &Identity, tier: u64, first_tick: u32, auto_restart: bool, fail_streak: u32, steps: u32) -> Result<i64, String> {
    let steps = steps.clamp(1, MAX_STEPS);
    // Secrets d1..dN: step k reveals d(k) and commits K12(d(k+1)); step N only leaves.
    let secrets: Vec<Vec<u8>> = (0..steps).map(|_| new_secret()).collect();
    let sign = |step_tick: u32, reveal: &[u8], commit: Option<&[u8; 32]>| -> (String, String) {
        let tx = CustomTransferTransaction::from_vars(id, RANDOM_CONTRACT_IDENTITY, tier, REVEAL_AND_COMMIT, step_tick, &reveal_and_commit_input(reveal, commit));
        (tx.txid(), hex::encode(&tx._signature))
    };
    let mut chain: Vec<NewStep> = Vec::with_capacity(steps as usize + 1);
    for k in 0..=steps {
        let step_tick = first_tick + STREAM_TICKS * k;
        let reveal: &[u8] = if k == 0 { &[] } else { &secrets[(k - 1) as usize] };
        let next = if k < steps { Some(&secrets[k as usize]) } else { None };
        chain.push(NewStep {
            step: k,
            tick: step_tick,
            reveal_secret: hex::encode(reveal),
            commit_secret: next.map(|s| hex::encode(s)).unwrap_or_default(),
            stay: next.map(|s| sign(step_tick, reveal, Some(&commitment(s)))),
            leave: if k == 0 { None } else { Some(sign(step_tick, reveal, None)) },
        });
    }
    let first = chain[0].stay.clone().unwrap();
    let path = get_db_path();
    let session_id = random_session::create_session(path.as_str(), id.identity.as_str(), tier, first_tick, auto_restart, fail_streak, steps, &chain)?;
    // The first commit goes out now; the driver sends the rest as their ticks come near.
    transfer::create_transfer(path.as_str(), id.identity.as_str(), RANDOM_CONTRACT_IDENTITY, tier, first_tick, &first.1, &first.0)?;
    let _ = random_session::set_session_progress(path.as_str(), session_id, 0, -1);
    if auto_restart {
        SESSION_SEEDS.lock().unwrap().insert(session_id, id.seed.clone());
    }
    info!("RANDOM session {} started by {} at tier {} QU: first commit {} at tick {}, {} steps signed", session_id, id.identity, tier, first.0, first_tick, chain.len());
    Ok(session_id)
}

/// Forgets a session's seed once it can no longer be restarted.
pub fn forget_seed(id: i64) {
    if let Some(mut seed) = SESSION_SEEDS.lock().unwrap().remove(&id) {
        unsafe { std::ptr::write_bytes(seed.as_mut_ptr(), 0, seed.len()); }
    }
}

/// Ends a session with the reason shown to the user. A "keep mining" session
/// is replaced by a fresh one from the same identity, unless the chain keeps
/// failing before a single step is accepted, or the seed is no longer held.
fn fail(path: &str, session: &HashMap<String, String>, failed_tick: u32, reason: String) {
    let id: i64 = num(session, "id").unwrap_or(0);
    let auto_restart = get(session, "auto_restart") == "1";
    let progressed = num::<u32>(session, "last_accepted_tick").unwrap_or(0) >= num::<u32>(session, "first_tick").unwrap_or(u32::MAX);
    let streak = if progressed { 0 } else { num::<u32>(session, "fail_streak").unwrap_or(0) + 1 };
    let mut reason = reason;
    if auto_restart {
        let seed = SESSION_SEEDS.lock().unwrap().get(&id).cloned();
        match seed {
            None => reason.push_str("; not restarted: the seed is no longer in memory (Rubic was restarted)"),
            Some(_) if streak >= MAX_FAIL_STREAK => reason.push_str(&format!("; not restarted after {} failures in a row without an accepted step", streak)),
            Some(seed) => {
                let latest: u32 = tick::fetch_latest_tick(path).ok().and_then(|t| t.parse().ok()).unwrap_or(0);
                let tier: u64 = num(session, "tier").unwrap_or(0);
                let steps: u32 = num(session, "total_steps").unwrap_or(miner::random::DEFAULT_STEPS);
                match start_session(&Identity::new(&seed), tier, latest + FIRST_COMMIT_LEAD_TICKS, true, streak, steps) {
                    Ok(new_id) => {
                        let _ = random_session::set_session_continued(path, id, new_id);
                        reason.push_str(&format!("; continued as session {}", new_id));
                    },
                    Err(err) => reason.push_str(&format!("; could not start a new session: {}", err)),
                }
            },
        }
    }
    error!("RANDOM session {}: {}", id, reason);
    let _ = random_session::set_session_failed(path, id, failed_tick, &reason);
    forget_seed(id);
}

/// Asks the contract about the identity's provider slots (answered into the
/// `random_provider` table by the response handler).
fn request_provider_status(peer_set: &Arc<Mutex<PeerSet>>, identity: &str) {
    let Ok(pub_key) = get_public_key_from_identity(&identity.to_string()) else { return };
    let request = api::QubicApiPacket::request_contract_function(RANDOM_CONTRACT_INDEX, GET_PROVIDER_STATUS, &pub_key);
    // Sessions settle on this report; a routine request would be dropped
    // whenever the queue is busy, so it goes ahead of the routine polls.
    if let Err(err) = peer_set.lock().unwrap().make_request_high_priority(request) {
        error!("{}", err);
    }
}

/// A session's slot is gone from the contract, on enough peers' word.
#[derive(Debug, PartialEq)]
pub struct Gone {
    /// The last sent step the reports are late enough to judge.
    pub judged: i64,
    /// The newest report's tick.
    pub newest: u32,
    /// How many peers reported the slot gone.
    pub peers: usize,
}

/// What several peers' GetProviderStatus reports say about a session.
#[derive(Debug, PartialEq)]
pub struct Verdict {
    /// The latest tick the contract accepted a step at (at least the value passed in).
    pub last_accepted: u32,
    pub gone: Option<Gone>,
    /// The slot is there but a step's tick is well past without being
    /// accepted: `(that step's tick, peers reporting so)`.
    pub not_accepted: Option<(u32, usize)>,
}

/// Judges a session on the reports of several peers (`(peer, tick the report
/// is dated, slots)`). A single peer's word never ends a session: one serving
/// stale state would halt a healthy chain, and the steps that then stop going
/// out get the provider evicted for real. `needed` peers must agree, and
/// reports of an empty slot must span a full stream period, unless they all
/// come from after the session's own leave.
pub fn judge_reports(reports: &[(String, u32, Vec<ProviderSlot>)], tier: u64, first_tick: u32, through: i64, leave_step: i64, last_accepted: u32, latest: u32, needed: usize) -> Verdict {
    let step_tick = |k: i64| first_tick + STREAM_TICKS * k as u32;
    let slot_at_tier = |slots: &[ProviderSlot]| slots.iter().find(|s| s.tier_amount() == tier).cloned();
    let mut verdict = Verdict { last_accepted, gone: None, not_accepted: None };
    let mut newest_present: Option<u32> = None;
    for (_, checked, slots) in reports {
        if let Some(slot) = slot_at_tier(slots) {
            newest_present = Some(newest_present.map_or(*checked, |t| t.max(*checked)));
            if slot.last_update_tick > verdict.last_accepted && slot.last_update_tick >= first_tick {
                verdict.last_accepted = slot.last_update_tick;
            }
        }
    }
    // Recent reports of no slot, newer than any report that still saw it.
    let absent: Vec<(&str, u32)> = reports.iter()
        .filter(|(_, checked, slots)| checked + REPORT_MAX_AGE_TICKS >= latest && newest_present.map_or(true, |p| *checked > p) && slot_at_tier(slots).is_none())
        .map(|(peer, checked, _)| (peer.as_str(), *checked)).collect();
    let absent_peers = absent.iter().map(|(p, _)| *p).collect::<HashSet<&str>>().len();
    let absent_oldest = absent.iter().map(|(_, c)| *c).min().unwrap_or(0);
    let absent_newest = absent.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let leave_tick = if leave_step >= 0 { Some(step_tick(leave_step)) } else { None };
    let gone = absent_peers >= needed
        && (absent_newest >= absent_oldest + STREAM_TICKS || leave_tick.map_or(false, |t| absent_oldest >= t + STREAM_TICKS));
    // Every sent step whose tick (plus the tick the contract needs to evict) is
    // behind a report should have been accepted by then.
    let judged_by = |tick: u32| (0..=through).filter(|k| step_tick(*k) + STREAM_TICKS <= tick).max();
    if gone {
        if let Some(judged) = judged_by(absent_newest) {
            verdict.gone = Some(Gone { judged, newest: absent_newest, peers: absent_peers });
        }
    } else if let Some(present) = newest_present {
        if let Some(k) = judged_by(present) {
            let due_tick = step_tick(k);
            if verdict.last_accepted < due_tick {
                let stale_peers = reports.iter()
                    .filter(|(_, checked, slots)| *checked >= due_tick + 2 * STREAM_TICKS && slot_at_tier(slots).map_or(false, |s| s.last_update_tick < due_tick))
                    .map(|(peer, _, _)| peer.as_str()).collect::<HashSet<&str>>().len();
                if stale_peers >= needed {
                    verdict.not_accepted = Some((due_tick, stale_peers));
                }
            }
        }
    }
    verdict
}

#[cfg(test)]
mod judge_tests {
    use super::*;

    const FIRST: u32 = 1000;
    const TIER: u64 = 1;

    fn slot(last_update: u32) -> Vec<ProviderSlot> {
        vec![ProviderSlot { stream: 0, tier: 0, locked: 1, contributed: true, last_update_tick: last_update }]
    }

    fn report(peer: &str, checked: u32, slots: Vec<ProviderSlot>) -> (String, u32, Vec<ProviderSlot>) {
        (peer.to_string(), checked, slots)
    }

    #[test]
    fn one_peer_saying_gone_is_not_enough() {
        let reports = vec![report("a", 1030, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1030, 2);
        assert_eq!(v.gone, None);
        assert_eq!(v.not_accepted, None);
    }

    #[test]
    fn two_peers_over_a_stream_period_mean_gone() {
        let reports = vec![report("a", 1030, vec![]), report("b", 1033, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1033, 2);
        assert_eq!(v.gone, Some(Gone { judged: 10, newest: 1033, peers: 2 }));
    }

    #[test]
    fn two_peers_at_the_same_tick_must_wait() {
        let reports = vec![report("a", 1030, vec![]), report("b", 1030, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1030, 2);
        assert_eq!(v.gone, None);
    }

    #[test]
    fn a_newer_sighting_of_the_slot_outranks_older_empty_reports() {
        let reports = vec![report("a", 1030, vec![]), report("b", 1033, vec![]), report("c", 1036, slot(1033))];
        let v = judge_reports(&reports, TIER, FIRST, 12, -1, 1021, 1036, 2);
        assert_eq!(v.gone, None);
        assert_eq!(v.last_accepted, 1033, "the sighting also advances the accepted tick");
    }

    #[test]
    fn stale_empty_reports_do_not_count() {
        let reports = vec![report("a", 1030, vec![]), report("b", 1033, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1033 + REPORT_MAX_AGE_TICKS + 1, 2);
        assert_eq!(v.gone, None);
    }

    #[test]
    fn after_a_leave_no_span_is_needed() {
        // Leave at step 10 (tick 1030): reports from tick 1033 on settle it.
        let reports = vec![report("a", 1033, vec![]), report("b", 1033, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, 10, 1027, 1033, 2);
        assert_eq!(v.gone, Some(Gone { judged: 10, newest: 1033, peers: 2 }));
    }

    #[test]
    fn a_step_left_unrecorded_needs_two_peers_too() {
        // Step 8 (tick 1024) is due; reports dated 1030 or later still show 1018
        // as the last accepted tick, two stream periods on.
        let one = vec![report("a", 1031, slot(1018))];
        let v = judge_reports(&one, TIER, FIRST, 8, -1, 1018, 1031, 2);
        assert_eq!(v.not_accepted, None);
        let two = vec![report("a", 1031, slot(1018)), report("b", 1030, slot(1018))];
        let v = judge_reports(&two, TIER, FIRST, 8, -1, 1018, 1031, 2);
        assert_eq!(v.not_accepted, Some((1024, 2)));
    }

    #[test]
    fn a_lagging_peer_cannot_call_a_step_missing() {
        // Peer b is dated well behind: its old last-accepted tick is no news.
        let reports = vec![report("a", 1031, slot(1018)), report("b", 1022, slot(1018))];
        let v = judge_reports(&reports, TIER, FIRST, 8, -1, 1018, 1031, 2);
        assert_eq!(v.not_accepted, None);
    }

    #[test]
    fn a_single_peer_wallet_judges_on_one() {
        let reports = vec![report("a", 1033, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1033, 1);
        assert_eq!(v.gone, None, "one report cannot span a stream period");
        let reports = vec![report("a", 1036, vec![]), report("a", 1030, vec![])];
        let v = judge_reports(&reports, TIER, FIRST, 10, -1, 1021, 1036, 1);
        assert!(v.gone.is_some());
    }
}

pub fn run_random_sessions(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut last_status_request: HashMap<String, Instant> = HashMap::new();
        // Session id -> when a balance report was last asked for to verify its leave.
        let mut leave_checks: HashMap<i64, Instant> = HashMap::new();
        let mut last_prune = Instant::now() - PRUNE_INTERVAL;
        let mut last_settle = Instant::now() - SETTLE_INTERVAL;
        // The tick found in the store at start is whatever was known when Rubic
        // last ran; nothing is sent, or judged too late, until a peer has answered.
        let mut tick_at_start: Option<u32> = None;
        loop {
            std::thread::sleep(PASS_INTERVAL);
            let path = get_db_path();
            if last_prune.elapsed() >= PRUNE_INTERVAL {
                last_prune = Instant::now();
                match random_session::prune_steps(path.as_str(), MAX_STEP_ROWS) {
                    Ok(0) => {},
                    Ok(n) => info!("RANDOM: removed {} steps of finished sessions to keep the table under {} rows", n, MAX_STEP_ROWS),
                    Err(err) => error!("RANDOM: could not prune steps: {}", err),
                }
            }
            let latest: u32 = tick::fetch_latest_tick(path.as_str()).ok().and_then(|t| t.parse().ok()).unwrap_or(0);
            if latest == 0 {
                continue;
            }
            let tick_moved = match tick_at_start {
                None => {
                    tick_at_start = Some(latest);
                    false
                },
                Some(at_start) => latest > at_start,
            };
            if last_settle.elapsed() >= SETTLE_INTERVAL {
                last_settle = Instant::now();
                match random_session::confirm_accepted_steps(path.as_str(), RANDOM_CONTRACT_IDENTITY) {
                    Ok(0) => {},
                    Ok(confirmed) => info!("RANDOM: {} step transfers confirmed from the contract's report", confirmed),
                    Err(err) => error!("RANDOM: could not confirm step transfers: {}", err),
                }
            }
            // Pre-flight checks asked for by the start route.
            let checks: Vec<String> = STATUS_CHECKS.lock().unwrap().drain().collect();
            for identity in checks {
                request_provider_status(&peer_set, &identity);
                last_status_request.insert(identity, Instant::now());
            }
            let Ok(sessions) = random_session::fetch_active_sessions(path.as_str()) else { continue };
            last_status_request.retain(|_, at| at.elapsed() < Duration::from_secs(600));
            for session in sessions {
                let Some(id) = num::<i64>(&session, "id") else { continue };
                let identity = get(&session, "identity");
                let tier: u64 = num(&session, "tier").unwrap_or(0);
                let status: i64 = num(&session, "status").unwrap_or(STATUS_RUNNING);
                let first_tick: u32 = num(&session, "first_tick").unwrap_or(0);
                let mut through: i64 = num(&session, "broadcast_through").unwrap_or(-1);
                let mut leave_step: i64 = num(&session, "leave_step").unwrap_or(-1);
                let mut last_accepted: u32 = num(&session, "last_accepted_tick").unwrap_or(0);
                // Chain length: the signed reveal steps plus the first commit.
                let total: u32 = num::<u32>(&session, "total_steps").unwrap_or(miner::random::DEFAULT_STEPS) + 1;
                let step_tick = |k: i64| first_tick + STREAM_TICKS * k as u32;

                // Keep the contract's view of this provider fresh.
                if last_status_request.get(&identity).map_or(true, |at| at.elapsed() >= PROVIDER_STATUS_INTERVAL) {
                    request_provider_status(&peer_set, &identity);
                    last_status_request.insert(identity.clone(), Instant::now());
                }

                // Settle against the contract's reports, one per answering peer: which
                // step was last accepted, and whether the slot is still there. Ending a
                // session takes the agreement of several peers; on one peer's word the
                // steps keep going out, which costs nothing if that peer was right.
                let needed = CORROBORATING_PEERS.min(peer_set.lock().unwrap().num_peers().max(1));
                let reports: Vec<(String, u32, Vec<ProviderSlot>)> = random_session::fetch_provider_reports(path.as_str(), &identity).unwrap_or_default()
                    .into_iter().map(|(peer, checked, slots)| (peer, checked, ProviderSlot::parse_list(&slots))).collect();
                let verdict = judge_reports(&reports, tier, first_tick, through, leave_step, last_accepted, latest, needed);
                if verdict.last_accepted > last_accepted {
                    last_accepted = verdict.last_accepted;
                    let _ = random_session::set_session_accepted(path.as_str(), id, last_accepted);
                }
                let leave_tick = if leave_step >= 0 { Some(step_tick(leave_step)) } else { None };
                if let Some(Gone { judged: k, newest: absent_newest, peers: absent_peers }) = verdict.gone {
                    {
                        // After a leave the slot is gone either way. It was a clean leave
                        // when the leave itself was seen included in its tick, or when the
                        // contract had accepted every step before it. Otherwise the peers'
                        // entity report decides (it names the identity's latest outgoing
                        // transfer): ask for one and wait a little before giving up.
                        if let Some(leave_tick) = leave_tick.filter(|_| k >= leave_step) {
                            let leave_status = random_session::fetch_step(path.as_str(), id, leave_step as u32).ok().flatten()
                                .and_then(|step| transfer_status(&get(&step, "leave_txid")));
                            let included = leave_status.as_deref() == Some("0");
                            if included || last_accepted >= step_tick(leave_step - 1).max(first_tick) {
                                info!("RANDOM session {}: left at step {} (tick {}), collateral returned", id, leave_step, leave_tick);
                                let _ = random_session::set_session_accepted(path.as_str(), id, leave_tick);
                                let _ = random_session::set_session_status(path.as_str(), id, STATUS_STOPPED);
                                leave_checks.remove(&id);
                                forget_seed(id);
                                continue;
                            }
                            let not_included = leave_status.as_deref() == Some("1");
                            if !not_included && absent_newest < leave_tick + LEAVE_VERIFY_TICKS {
                                if leave_checks.get(&id).map_or(true, |at| at.elapsed() >= LEAVE_CHECK_INTERVAL) {
                                    leave_checks.insert(id, Instant::now());
                                    if let Err(err) = peer_set.lock().unwrap().make_request(api::QubicApiPacket::get_identity_balance(&identity)) {
                                        error!("{}", err);
                                    }
                                }
                                continue;
                            }
                            leave_checks.remove(&id);
                            let reason = if not_included {
                                format!("the reveal-and-leave at tick {} was not included in its tick; the provider was evicted and the stake of {} QU forfeited", leave_tick, tier)
                            } else {
                                format!("the reveal-and-leave at tick {} could not be verified: peers did not report on it within {} ticks", leave_tick, LEAVE_VERIFY_TICKS)
                            };
                            fail(path.as_str(), &session, leave_tick, reason);
                            continue;
                        }
                        let (failed_tick, reason) = if last_accepted < first_tick {
                            (first_tick, format!("the first commit at tick {} was not accepted by the contract (wrong amount, stream full, or not included)", first_tick))
                        } else {
                            let missed = last_accepted + STREAM_TICKS;
                            (missed, format!("evicted by the contract: the step at tick {} was not accepted (reported by {} peers), the stake of {} QU was forfeited", missed, absent_peers, tier))
                        };
                        fail(path.as_str(), &session, failed_tick, reason);
                        continue;
                    }
                } else if let Some((due_tick, stale_peers)) = verdict.not_accepted {
                    // Slot still there but the contract did not record a step whose tick is
                    // well behind: it was rejected, and the next stream tick evicts the
                    // provider. Again only on the word of enough peers.
                    fail(path.as_str(), &session, due_tick, format!("the step at tick {} was not accepted by the contract (last accepted tick {}, reported by {} peers)", due_tick, last_accepted, stale_peers));
                    continue;
                }
                if leave_step >= 0 || !tick_moved {
                    continue;
                }

                // Broadcast the steps whose ticks are coming up.
                let mut next = (through + 1) as u32;
                while next < total {
                    let this_tick = step_tick(next as i64);
                    if this_tick > latest + BROADCAST_LEAD_TICKS {
                        break;
                    }
                    if next > 0 && this_tick <= latest + TOO_LATE_TICKS {
                        // Its tick is here before it went out (the wallet was down or its
                        // tick view stalled): the provider is a no-show and the chain ends.
                        fail(path.as_str(), &session, this_tick, format!("step {} (tick {}) could not be sent in time; the latest tick was already {}", next, this_tick, latest));
                        break;
                    }
                    let Ok(Some(step)) = random_session::fetch_step(path.as_str(), id, next) else { break };
                    let leave = next > 0 && (status == STATUS_STOP_REQUESTED || next + 1 == total);
                    if let Err(err) = broadcast_step(&session, &step, leave) {
                        error!("RANDOM session {}: could not record step {}: {}", id, next, err);
                        break;
                    }
                    through = next as i64;
                    if leave {
                        leave_step = next as i64;
                        info!("RANDOM session {}: leaving at step {} (tick {})", id, next, this_tick);
                        let _ = random_session::set_session_status(path.as_str(), id, STATUS_LEAVING);
                    } else {
                        info!("RANDOM session {}: step {} broadcast for tick {}", id, next, this_tick);
                    }
                    let _ = random_session::set_session_progress(path.as_str(), id, through, leave_step);
                    if leave {
                        break;
                    }
                    next += 1;
                }
            }
        }
    });
}
