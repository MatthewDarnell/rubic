use rocket::{get, post};
use rocket::serde::{Deserialize, json::Json};
use logger::{error, info};
use miner::random::{is_tier, MAX_STEPS, STREAM_TICKS};
use store::get_db_path;
use store::sqlite::random_session::{self, STATUS_RUNNING, STATUS_STOP_REQUESTED};
use store::sqlite::{identity, tick};
use crate::routes::MINPASSWORDLEN;

/// Ticks ahead of the latest known tick the first commit is scheduled when
/// the caller does not choose; the same default the UI uses for transfers.
const DEFAULT_TICK_OFFSET: u32 = 30;

/// Body of `POST /miner/random/start`. `password` may be omitted when the
/// wallet is unlocked or the seed is stored unencrypted; `tier` is the
/// collateral in QU (1, 10, 100 … 1e9); `tick` is the first commit tick, 0 (or
/// omitted) for latest + 30.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StartRandomSessionRequest {
    pub identity: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub tier: u64,
    #[serde(default)]
    pub tick: u32,
    /// Keep mining: replace a failed session with a new one automatically
    /// (the identity's seed stays in memory while the session runs).
    #[serde(default)]
    pub auto_restart: bool,
}

/// Body of `POST /miner/random/stop`.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct StopRandomSessionRequest {
    pub session_id: i64,
}

/// The wallet's RANDOM sessions, newest first, each with `steps` (signed) and
/// `step_ticks` (every 3 ticks from `first_tick`).
#[get("/miner/random")]
pub fn random_sessions() -> String {
    match random_session::fetch_sessions(get_db_path().as_str(), 50) {
        Ok(mut sessions) => {
            for session in sessions.iter_mut() {
                // Every session is signed with the same chain length; the step rows of
                // finished sessions may have been pruned, so this is not a row count.
                session.insert("steps".to_string(), (MAX_STEPS + 1).to_string());
                session.insert("step_ticks".to_string(), STREAM_TICKS.to_string());
                // Steps sit at first_tick + 3k, so the contract's last accepted tick
                // says how many reveal steps went through (each returned the previous
                // stake); anything sent beyond it whose tick has passed did not.
                let first_tick: i64 = session.get("first_tick").and_then(|v| v.parse().ok()).unwrap_or(0);
                let through: i64 = session.get("broadcast_through").and_then(|v| v.parse().ok()).unwrap_or(-1);
                let accepted_tick: i64 = session.get("last_accepted_tick").and_then(|v| v.parse().ok()).unwrap_or(0);
                let status: i64 = session.get("status").and_then(|v| v.parse().ok()).unwrap_or(0);
                // `accepted` counts the steps the contract recorded (the first commit
                // included); `sent` the steps the wallet has put on the wire.
                let enrolled = accepted_tick >= first_tick && accepted_tick > 0;
                let accepted = if enrolled { (accepted_tick - first_tick) / STREAM_TICKS as i64 + 1 } else { 0 };
                let sent = through + 1;
                // What became of the stake: locked while providing, returned on a clean
                // leave, lost when evicted after enrolling, none when the identity never
                // got in (a rejected or unsent first commit costs nothing), unknown for
                // sessions that ended before the contract report existed.
                let reported = enrolled || !session.get("reason").map(|r| r.is_empty()).unwrap_or(true);
                let stake = match status {
                    0 | 1 | 2 => if enrolled { "locked" } else { "pending" },
                    4 => "returned",
                    3 if enrolled => "lost",
                    3 if reported => "none",
                    _ => "unknown",
                };
                session.insert("accepted".to_string(), accepted.to_string());
                session.insert("sent".to_string(), sent.to_string());
                session.insert("stake".to_string(), stake.to_string());
            }
            format!("{:?}", sessions)
        },
        Err(err) => err,
    }
}

// POST /miner/random/start  {"identity": "...", "password": "...", "tier": 1, "tick": 0}
// Starts a provider session at the given tier (see `miner::start_session`):
// every step is signed now and broadcast a few ticks ahead, so no seed is kept
// while the session runs - unless `auto_restart` asks for failed sessions to be
// replaced automatically, which needs the seed. Replies with the session id.
#[post("/miner/random/start", format = "json", data = "<body>")]
pub fn start_random_session(body: Json<StartRandomSessionRequest>) -> String {
    let StartRandomSessionRequest { identity: address, password, tier, tick, auto_restart } = body.into_inner();
    if address.len() != 60 {
        return "Invalid Identity!".to_string();
    }
    let tier = if tier == 0 { 1 } else { tier };
    if !is_tier(tier) {
        return "Invalid Tier! Use 1, 10, 100 ... 1000000000 QU".to_string();
    }
    let mut id = match identity::fetch_identity(get_db_path().as_str(), address.as_str()) {
        Ok(id) => id,
        Err(_) => return "Unknown Identity".to_string(),
    };
    if id.encrypted && protocol::wallet_unlock::is_wallet_unlocked().unwrap_or(false) {
        id = match id.decrypt_identity_unlocked_wallet() {
            Ok(id) => id,
            Err(_) => return "Failed To Decrypt Identity With The Unlocked Wallet".to_string(),
        };
    }
    if id.encrypted {
        if password.len() < MINPASSWORDLEN {
            return "Must Enter A Password!".to_string();
        }
        id = match id.decrypt_identity(password.as_str()) {
            Ok(id) => id,
            Err(_) => return "Invalid Password".to_string(),
        };
    }
    // One session per identity at a time: the contract keys providers by identity and tier.
    if let Ok(active) = random_session::fetch_active_sessions(get_db_path().as_str()) {
        if active.iter().any(|s| s.get("identity").map(|i| i == &id.identity).unwrap_or(false)) {
            return "This identity already has a running session".to_string();
        }
    }

    let latest: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
        Ok(t) => t.parse().unwrap_or(0),
        Err(_) => 0,
    };
    if latest == 0 {
        return "No tick from peers yet".to_string();
    }
    let first_tick = if tick > latest { tick } else { latest + DEFAULT_TICK_OFFSET };
    match crate::miner::start_session(&id, tier, first_tick, auto_restart, 0) {
        Ok(session_id) => session_id.to_string(),
        Err(err) => {
            error!("Failed To Start RANDOM Session: {}", err);
            format!("Failed To Start RANDOM Session: {}", err)
        }
    }
}

// POST /miner/random/stop  {"session_id": 1}
// The next step not yet on the wire goes out as a reveal-and-leave, which
// returns the collateral. Steps already broadcast cannot be recalled. Also
// switches "keep mining" off for the session, so nothing restarts after it.
#[post("/miner/random/stop", format = "json", data = "<body>")]
pub fn stop_random_session(body: Json<StopRandomSessionRequest>) -> String {
    let session_id = body.into_inner().session_id;
    let session = match random_session::fetch_session(get_db_path().as_str(), session_id) {
        Ok(Some(session)) => session,
        _ => return "Unknown Session".to_string(),
    };
    let status: i64 = session.get("status").and_then(|s| s.parse().ok()).unwrap_or(-1);
    if status != STATUS_RUNNING {
        return "This session is not running".to_string();
    }
    match random_session::set_session_status(get_db_path().as_str(), session_id, STATUS_STOP_REQUESTED) {
        Ok(_) => {
            // Stop means stop: a "keep mining" session must not be replaced by a
            // new one if something fails on the way out, and its seed is no longer needed.
            let _ = random_session::set_session_auto_restart(get_db_path().as_str(), session_id, false);
            crate::miner::forget_seed(session_id);
            info!("RANDOM session {}: stop requested", session_id);
            "Stop Requested".to_string()
        },
        Err(err) => format!("Failed To Stop Session: {}", err),
    }
}
