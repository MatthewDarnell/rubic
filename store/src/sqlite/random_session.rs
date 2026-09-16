//! RANDOM provider sessions and their pre-signed steps.
//!
//! A session is one identity providing entropy at one collateral tier, acting
//! at ticks `first_tick + 3k`. Every step's transaction is signed when the
//! session starts: step 0 is the first commit; steps 1.. each carry a "stay"
//! transaction (reveal, commit next) and a "leave" transaction (reveal, no
//! commit); the last step only leaves. The driver broadcasts steps a few
//! ticks ahead and picks the leave variant once a stop is requested.

use std::collections::HashMap;
use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

/// Providing: steps are broadcast as their ticks come near.
pub const STATUS_RUNNING: i64 = 0;
/// Stop requested: the next unsent step is broadcast as a leave.
pub const STATUS_STOP_REQUESTED: i64 = 1;
/// The leave step is on the wire.
pub const STATUS_LEAVING: i64 = 2;
/// A step was not included in its tick (or was missed while the wallet was down).
pub const STATUS_FAILED: i64 = 3;
/// The leave step was included: collateral returned.
pub const STATUS_STOPPED: i64 = 4;

const SESSION_COLUMNS: [&str; 14] = ["id", "identity", "tier", "first_tick", "status", "broadcast_through", "leave_step", "last_accepted_tick", "reason", "auto_restart", "fail_streak", "continued_by", "failed_tick", "created"];
const STEP_COLUMNS: [&str; 9] = ["session_id", "step", "tick", "reveal_secret", "commit_secret", "stay_txid", "stay_sig", "leave_txid", "leave_sig"];

fn read_row(statement: &sqlite::Statement, columns: &[&str]) -> HashMap<String, String> {
    let mut row = HashMap::new();
    for column in columns {
        row.insert(column.to_string(), statement.read::<String, _>(*column).unwrap_or_default());
    }
    row
}

fn fetch(path: &str, query: &str, binds: &[(&str, &str)], columns: &[&str]) -> Result<Vec<HashMap<String, String>>, String> {
    let _lock = get_db_lock().lock().unwrap();
    let connection = open_database(path, false)?;
    let mut statement = prepare_crud_statement(&connection, query)?;
    statement.bind::<&[(&str, &str)]>(binds).map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    loop {
        match statement.next() {
            Ok(State::Row) => rows.push(read_row(&statement, columns)),
            Ok(State::Done) => break,
            Err(err) => {
                error!("Error reading random sessions: {}", err);
                return Err(err.to_string());
            }
        }
    }
    Ok(rows)
}

fn execute(path: &str, query: &str, binds: &[(&str, &str)]) -> Result<(), String> {
    let _lock = get_db_lock().lock().unwrap();
    let connection = open_database(path, false)?;
    let mut statement = prepare_crud_statement(&connection, query)?;
    statement.bind::<&[(&str, &str)]>(binds).map_err(|e| e.to_string())?;
    match statement.next() {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

/// A step ready to be stored: secrets as hex, transactions as (txid, signature hex).
pub struct NewStep {
    pub step: u32,
    pub tick: u32,
    pub reveal_secret: String,
    pub commit_secret: String,
    pub stay: Option<(String, String)>,
    pub leave: Option<(String, String)>,
}

/// Creates the session and all its steps in one transaction; returns the session id.
pub fn create_session(path: &str, identity: &str, tier: u64, first_tick: u32, auto_restart: bool, fail_streak: u32, steps: &[NewStep]) -> Result<i64, String> {
    let _lock = get_db_lock().lock().unwrap();
    let connection = open_database(path, false)?;
    connection.execute("BEGIN TRANSACTION;").map_err(|e| e.to_string())?;
    let result = (|| -> Result<i64, String> {
        let tier = tier.to_string();
        let first_tick = first_tick.to_string();
        let auto_restart = if auto_restart { "1" } else { "0" };
        let fail_streak = fail_streak.to_string();
        let mut insert = prepare_crud_statement(&connection, "INSERT INTO random_session (identity, tier, first_tick, status, broadcast_through, leave_step, auto_restart, fail_streak) VALUES (:identity, :tier, :first_tick, 0, -1, -1, CAST(:auto_restart AS INTEGER), CAST(:fail_streak AS INTEGER));")?;
        insert.bind::<&[(&str, &str)]>(&[(":identity", identity), (":tier", tier.as_str()), (":first_tick", first_tick.as_str()), (":auto_restart", auto_restart), (":fail_streak", fail_streak.as_str())][..]).map_err(|e| e.to_string())?;
        insert.next().map_err(|e| e.to_string())?;
        let mut id_query = prepare_crud_statement(&connection, "SELECT last_insert_rowid() AS id;")?;
        let id: i64 = match id_query.next() {
            Ok(State::Row) => id_query.read::<i64, _>("id").map_err(|e| e.to_string())?,
            _ => return Err("No session id".to_string()),
        };
        let session_id = id.to_string();
        let mut step_insert = prepare_crud_statement(&connection, "INSERT INTO random_step (session_id, step, tick, reveal_secret, commit_secret, stay_txid, stay_sig, leave_txid, leave_sig) \
            VALUES (:session_id, :step, :tick, :reveal_secret, :commit_secret, :stay_txid, :stay_sig, :leave_txid, :leave_sig);")?;
        for s in steps {
            let step = s.step.to_string();
            let tick = s.tick.to_string();
            let (stay_txid, stay_sig) = s.stay.clone().unwrap_or_default();
            let (leave_txid, leave_sig) = s.leave.clone().unwrap_or_default();
            step_insert.reset().map_err(|e| e.to_string())?;
            step_insert.bind::<&[(&str, &str)]>(&[
                (":session_id", session_id.as_str()), (":step", step.as_str()), (":tick", tick.as_str()),
                (":reveal_secret", s.reveal_secret.as_str()), (":commit_secret", s.commit_secret.as_str()),
                (":stay_txid", stay_txid.as_str()), (":stay_sig", stay_sig.as_str()),
                (":leave_txid", leave_txid.as_str()), (":leave_sig", leave_sig.as_str()),
            ][..]).map_err(|e| e.to_string())?;
            step_insert.next().map_err(|e| e.to_string())?;
        }
        Ok(id)
    })();
    match result {
        Ok(id) => {
            connection.execute("COMMIT;").map_err(|e| e.to_string())?;
            Ok(id)
        },
        Err(err) => {
            let _ = connection.execute("ROLLBACK;");
            error!("Error creating a random session: {}", err);
            Err(err)
        }
    }
}

/// Newest sessions first.
pub fn fetch_sessions(path: &str, limit: u32) -> Result<Vec<HashMap<String, String>>, String> {
    let limit = limit.to_string();
    fetch(path, "SELECT * FROM random_session ORDER BY id DESC LIMIT CAST(:limit AS INTEGER);", &[(":limit", limit.as_str())], &SESSION_COLUMNS)
}

/// Sessions still acting on the network: running, stop requested, or leaving.
pub fn fetch_active_sessions(path: &str) -> Result<Vec<HashMap<String, String>>, String> {
    fetch(path, "SELECT * FROM random_session WHERE status IN (0, 1, 2) ORDER BY id ASC;", &[], &SESSION_COLUMNS)
}

pub fn fetch_session(path: &str, id: i64) -> Result<Option<HashMap<String, String>>, String> {
    let id = id.to_string();
    Ok(fetch(path, "SELECT * FROM random_session WHERE id = CAST(:id AS INTEGER);", &[(":id", id.as_str())], &SESSION_COLUMNS)?.into_iter().next())
}

pub fn set_session_status(path: &str, id: i64, status: i64) -> Result<(), String> {
    let id = id.to_string();
    let status = status.to_string();
    execute(path, "UPDATE random_session SET status = CAST(:status AS INTEGER) WHERE id = CAST(:id AS INTEGER);", &[(":status", status.as_str()), (":id", id.as_str())])
}

/// Ends a session: the tick it failed at, and the reason shown on hover.
pub fn set_session_failed(path: &str, id: i64, failed_tick: u32, reason: &str) -> Result<(), String> {
    let id = id.to_string();
    let status = STATUS_FAILED.to_string();
    let failed_tick = failed_tick.to_string();
    execute(path, "UPDATE random_session SET status = CAST(:status AS INTEGER), failed_tick = CAST(:failed_tick AS INTEGER), reason = :reason WHERE id = CAST(:id AS INTEGER);",
        &[(":status", status.as_str()), (":failed_tick", failed_tick.as_str()), (":reason", reason), (":id", id.as_str())])
}

/// Turns "keep mining" off, so a failure after this ends the chain.
pub fn set_session_auto_restart(path: &str, id: i64, on: bool) -> Result<(), String> {
    let id = id.to_string();
    execute(path, "UPDATE random_session SET auto_restart = CAST(:on AS INTEGER) WHERE id = CAST(:id AS INTEGER);",
        &[(":on", if on { "1" } else { "0" }), (":id", id.as_str())])
}

/// Links a failed session to the one started automatically in its place.
pub fn set_session_continued(path: &str, id: i64, continued_by: i64) -> Result<(), String> {
    let id = id.to_string();
    let continued_by = continued_by.to_string();
    execute(path, "UPDATE random_session SET continued_by = CAST(:continued_by AS INTEGER) WHERE id = CAST(:id AS INTEGER);",
        &[(":continued_by", continued_by.as_str()), (":id", id.as_str())])
}

/// The tick of the last step the contract accepted for this session.
pub fn set_session_accepted(path: &str, id: i64, tick: u32) -> Result<(), String> {
    let id = id.to_string();
    let tick = tick.to_string();
    execute(path, "UPDATE random_session SET last_accepted_tick = CAST(:tick AS INTEGER) WHERE id = CAST(:id AS INTEGER);",
        &[(":tick", tick.as_str()), (":id", id.as_str())])
}

/// What the contract last reported for an identity's provider slots (text as
/// produced by the miner crate), and the wallet's tick when it was asked.
pub fn set_provider_status(path: &str, identity: &str, checked_tick: u32, slots: &str) -> Result<(), String> {
    let checked_tick = checked_tick.to_string();
    execute(path, "INSERT INTO random_provider (identity, checked_tick, slots, updated) VALUES (:identity, :checked_tick, :slots, CURRENT_TIMESTAMP) \
        ON CONFLICT(identity) DO UPDATE SET checked_tick = excluded.checked_tick, slots = excluded.slots, updated = excluded.updated;",
        &[(":identity", identity), (":checked_tick", checked_tick.as_str()), (":slots", slots)])
}

pub fn fetch_provider_status(path: &str, identity: &str) -> Result<Option<(u32, String)>, String> {
    let rows = fetch(path, "SELECT checked_tick, slots FROM random_provider WHERE identity = :identity;", &[(":identity", identity)], &["checked_tick", "slots"])?;
    Ok(rows.into_iter().next().map(|r| (r.get("checked_tick").and_then(|t| t.parse().ok()).unwrap_or(0), r.get("slots").cloned().unwrap_or_default())))
}

/// Records how far the driver has broadcast, and which step (if any) leaves.
pub fn set_session_progress(path: &str, id: i64, broadcast_through: i64, leave_step: i64) -> Result<(), String> {
    let id = id.to_string();
    let through = broadcast_through.to_string();
    let leave = leave_step.to_string();
    execute(path, "UPDATE random_session SET broadcast_through = CAST(:through AS INTEGER), leave_step = CAST(:leave AS INTEGER) WHERE id = CAST(:id AS INTEGER);",
        &[(":through", through.as_str()), (":leave", leave.as_str()), (":id", id.as_str())])
}

pub fn fetch_step(path: &str, session_id: i64, step: u32) -> Result<Option<HashMap<String, String>>, String> {
    let session_id = session_id.to_string();
    let step = step.to_string();
    Ok(fetch(path, "SELECT * FROM random_step WHERE session_id = CAST(:session_id AS INTEGER) AND step = CAST(:step AS INTEGER);",
        &[(":session_id", session_id.as_str()), (":step", step.as_str())], &STEP_COLUMNS)?.into_iter().next())
}

/// Steps `from..=through` of a session, in order.
pub fn fetch_steps_between(path: &str, session_id: i64, from: i64, through: i64) -> Result<Vec<HashMap<String, String>>, String> {
    let session_id = session_id.to_string();
    let from = from.to_string();
    let through = through.to_string();
    fetch(path, "SELECT * FROM random_step WHERE session_id = CAST(:session_id AS INTEGER) AND step >= CAST(:from AS INTEGER) AND step <= CAST(:through AS INTEGER) ORDER BY step ASC;",
        &[(":session_id", session_id.as_str()), (":from", from.as_str()), (":through", through.as_str())], &STEP_COLUMNS)
}

/// The step whose stay or leave transaction has this id.
pub fn fetch_step_by_txid(path: &str, txid: &str) -> Result<Option<HashMap<String, String>>, String> {
    Ok(fetch(path, "SELECT * FROM random_step WHERE stay_txid = :txid OR leave_txid = :txid LIMIT 1;", &[(":txid", txid)], &STEP_COLUMNS)?.into_iter().next())
}

/// Keeps the step table bounded: once it holds more than `keep` rows, the
/// oldest steps of finished sessions are removed until it does not. Steps of
/// running sessions are never touched; they are still to be broadcast.
/// Returns how many rows were removed.
pub fn prune_steps(path: &str, keep: u64) -> Result<u64, String> {
    let total: u64 = fetch(path, "SELECT COUNT(*) AS n FROM random_step;", &[], &["n"])?
        .first().and_then(|r| r.get("n")).and_then(|n| n.parse().ok()).unwrap_or(0);
    if total <= keep {
        return Ok(0);
    }
    let excess = (total - keep).to_string();
    execute(path, "DELETE FROM random_step WHERE rowid IN (         SELECT s.rowid FROM random_step s JOIN random_session ss ON ss.id = s.session_id         WHERE ss.status >= 3 ORDER BY s.session_id ASC, s.step ASC LIMIT CAST(:excess AS INTEGER));",
        &[(":excess", excess.as_str())])?;
    let after: u64 = fetch(path, "SELECT COUNT(*) AS n FROM random_step;", &[], &["n"])?
        .first().and_then(|r| r.get("n")).and_then(|n| n.parse().ok()).unwrap_or(total);
    Ok(total.saturating_sub(after))
}

pub fn count_steps(path: &str, session_id: i64) -> Result<u32, String> {
    let session_id = session_id.to_string();
    let rows = fetch(path, "SELECT COUNT(*) AS n FROM random_step WHERE session_id = CAST(:session_id AS INTEGER);", &[(":session_id", session_id.as_str())], &["n"])?;
    Ok(rows.first().and_then(|r| r.get("n")).and_then(|n| n.parse().ok()).unwrap_or(0))
}
