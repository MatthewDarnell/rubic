use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use base64::Engine;
use base64::engine::general_purpose;
use crypto::qubic_identities::{get_identity, get_public_key_from_identity};
use logger::error;
use network::peers::PeerSet;
use store::get_db_path;
use store::sqlite::{identity, tick, transfer};

/// How many ticks past a transfer's tick a peer report must be before "no
/// outgoing transfer at that tick" is taken to mean the transfer failed.
const ENTITY_REPORT_GRACE_TICKS: u32 = 5;

/// Settles a pending transfer from the peers' entity reports for its source
/// identity: they carry the tick of the identity's latest executed outgoing
/// transfer. Returns `true` when the transfer's status was decided here.
/// Outcome of a settlement attempt.
enum Settled {
    Confirmed,
    Failed,
    Undecided,
}

/// A confirmed transaction changed what the source identity holds: ask for its
/// balance and possessed assets right away, ahead of the routine passes (up to
/// a minute apart for holdings, and skipped while the request queue is busy).
fn refresh_source(peer_set: &Arc<Mutex<PeerSet>>, source: &str) {
    let mut lock = peer_set.lock().unwrap();
    if let Err(err) = lock.make_request_high_priority(api::QubicApiPacket::get_identity_balance(source)) {
        error!("{}", err);
    }
    if let Ok(pub_key) = get_public_key_from_identity(&source.to_string()) {
        if let Err(err) = lock.make_request_high_priority(api::QubicApiPacket::request_possessed_assets(&pub_key)) {
            error!("{}", err);
        }
    }
}

fn settle_from_entity_report(source: &str, txid: &str, tx_tick: u32) -> Settled {
    let (report_tick, latest_out, peers) = match identity::fetch_latest_entity_report(get_db_path().as_str(), source) {
        Ok(Some(report)) => report,
        _ => return Settled::Undecided,
    };
    // Need a report from after the tick, backed by more than one peer.
    if peers < 2 || report_tick <= tx_tick {
        return Settled::Undecided;
    }
    if latest_out == tx_tick {
        match transfer::set_broadcasted_transfer_as_success(get_db_path().as_str(), txid) {
            Ok(_) => println!("Transaction <{}> confirmed (peers report an outgoing transfer at tick {}).", txid, tx_tick),
            Err(err) => println!("Failed To Confirm Transaction {} ({})", txid, err),
        }
        return Settled::Confirmed;
    }
    if latest_out < tx_tick && report_tick > tx_tick + ENTITY_REPORT_GRACE_TICKS {
        match transfer::set_broadcasted_transfer_as_failure(get_db_path().as_str(), txid) {
            Ok(_) => println!("Transaction <{}> failed (peers report no outgoing transfer at tick {} by tick {}).", txid, tx_tick, report_tick),
            Err(err) => println!("Failed To Set Failed Transaction {} ({})", txid, err),
        }
        return Settled::Failed;
    }
    // latest_out > tx_tick: a later transfer executed; this one is ambiguous here.
    Settled::Undecided
}

/// Tick data for a given tick is asked for at most this often; the requests are
/// exempt from the backlog limiter, so they must not be fired every pass.
const TICK_DATA_REQUEST_INTERVAL: Duration = Duration::from_secs(10);

fn request_tick_data_throttled(peer_set: &mut PeerSet, last: &mut HashMap<u32, Instant>, tick: u32) -> Result<(), String> {
    if last.get(&tick).map_or(false, |at| at.elapsed() < TICK_DATA_REQUEST_INTERVAL) {
        return Ok(());
    }
    last.insert(tick, Instant::now());
    peer_set.make_request(api::QubicApiPacket::request_tick_data(tick))
}

pub fn confirm_transactions(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut last_tick_data_request: HashMap<u32, Instant> = HashMap::new();
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            last_tick_data_request.retain(|_, at| at.elapsed() < Duration::from_secs(3600));
            /*
            *
            *   SECTION <Look For Broadcasted Transfers That Are Executed To Confirm>
            *
            */
            let latest_tick: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
                Ok(tick) => {
                    tick.parse::<u32>().unwrap()
                },
                Err(_) => {
                    0 as u32
                }
            };

            match transfer::fetch_expired_and_broadcasted_transfers_with_unknown_status(get_db_path().as_str(), latest_tick) {
                Ok(transfers) => {
                    for transfer in transfers {
                        //std::thread::sleep(std::time::Duration::from_millis(100));
                        let _tick = transfer.get("tick").unwrap();
                        let txid = transfer.get("txid").unwrap();
                        //println!("looking for tx {} at tick {}", txid, _tick);
                        let tick = u32::from_str(_tick.as_str()).unwrap();

                        // First the cheap, peer-reported signal; the quorum tick data
                        // below is the full verification when it is available.
                        let to_contract = transfer.get("destination").map(|d| d == miner::random::RANDOM_CONTRACT_IDENTITY).unwrap_or(false);
                        if let Some(source) = transfer.get("source") {
                            match settle_from_entity_report(source.as_str(), txid.as_str(), tick) {
                                Settled::Confirmed => {
                                    // A RANDOM step settles every 3 ticks and only moves the stake:
                                    // not worth a balance and holdings round trip each time.
                                    if !to_contract {
                                        refresh_source(&peer_set, source.as_str());
                                    }
                                    continue;
                                },
                                Settled::Failed => continue,
                                Settled::Undecided => {},
                            }
                        }

                        if latest_tick - tick > 35000 {
                            match transfer::set_broadcasted_transfer_as_failure(get_db_path().as_str(), txid.as_str()) {
                                Ok(_) => {
                                    println!("Transaction <{}> Too Old. Marking Failed.", txid);
                                    continue;
                                },
                                Err(err) => {
                                    println!("Failed To Set Old Transaction As Failed {} ({})", txid.as_str(), err);
                                    continue;
                                }
                            }
                        }
                        
                        
                        match tick::fetch_tick(get_db_path().as_str(), tick) {
                            Ok(tick_result) => {
                                let _valid = tick_result.get(&"valid".to_string()).unwrap();
                                let valid: i32 = _valid.to_lowercase().parse::<i32>().unwrap();
                                if valid > 0 {
                                    let tx_digests_hash = tick_result.get(&"transaction_digests_hash".to_string()).unwrap();
                                    let tx_digests = tick_result.get(&"transaction_digests".to_string()).unwrap();
                                    if tx_digests_hash.len() < 8 {  //We have the Tick but not the tx_digests hash. Fetch Tick
                                        //println!("We Have Tick But No Digest Hash. Fetching Tick {}!", tick);
                                        {
                                            let mut _lock = peer_set.lock().unwrap();
                                            match request_tick_data_throttled(&mut _lock, &mut last_tick_data_request, tick) {
                                                Ok(_) => {},
                                                Err(_) => {
                                                    //println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                                }
                                            }
                                            drop(_lock);
                                        }
                                    } else {    //We have the Tick and tx_digests hash but not the full tx_digests. Fetch TickData
                                        if tx_digests.len() < 8 {
                                            //println!("We Have Tick But No Digests. Fetching Tick {} Data!", tick);
                                            {
                                                let mut _lock = peer_set.lock().unwrap();
                                                match request_tick_data_throttled(&mut _lock, &mut last_tick_data_request, tick) {
                                                    Ok(_) => {},
                                                    Err(_) => {
                                                        println!("TransactionConfirmer: Failed To Request Tick Data!");
                                                    }
                                                }
                                                drop(_lock);
                                            }
                                        }
                                        else {
                                            let mut tx_included: bool = false;
                                            let transactions = general_purpose::STANDARD_NO_PAD.decode::<&String>(tx_digests).unwrap();
                                            transactions.chunks_exact(32).for_each(|tx| {
                                                let hash = get_identity(&<[u8; 32]>::try_from(tx.to_vec()).unwrap());
                                                //println!("{} vs {}", hash.as_str(), txid.as_str());
                                                if txid.as_str().to_lowercase() ==  hash.to_lowercase() {
                                                    tx_included = true;
                                                    //Included!
                                                    match transfer::set_broadcasted_transfer_as_success(get_db_path().as_str(), txid.as_str()) {
                                                        Ok(_) => {
                                                            println!("Transaction <{}> confirmed.", txid);
                                                            if let Some(source) = transfer.get("source") {
                                                                if !to_contract {
                                                                    refresh_source(&peer_set, source.as_str());
                                                                }
                                                            }
                                                        },
                                                        Err(err) => {
                                                            println!("Failed To Confirm Transaction {} ({})", txid.as_str(), err);
                                                        }
                                                    }
                                                }
                                            });
                                            if !tx_included {
                                                match transfer::set_broadcasted_transfer_as_failure(get_db_path().as_str(), txid.as_str()) {
                                                    Ok(_) => {
                                                        println!("Transaction <{}> Failed.", txid);
                                                    },
                                                    Err(err) => {
                                                        println!("Failed To Set Failed Transaction {} ({})", txid.as_str(), err);
                                                    }
                                                }
                                            }

                                        }
                                    }
                                } else {
                                    //We failed to validate this tick before.
                                    //println!("Requesting Failed Tick To Validate: {}", tick);
                                    std::thread::sleep(std::time::Duration::from_millis(750));
                                    {
                                        let mut _lock = peer_set.lock().unwrap();
                                        match request_tick_data_throttled(&mut _lock, &mut last_tick_data_request, tick) {
                                            Ok(_) => {},
                                            Err(_) => {
                                                //println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                            }
                                        }
                                        drop(_lock);
                                    }
                                }
                            },
                            Err(_) => {
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                //We don't have this tick, fetch it, unless it's too old
                                //println!("Fetching tick {}", tick);
                                {
                                    let mut _lock = peer_set.lock().unwrap();
                                    match request_tick_data_throttled(&mut _lock, &mut last_tick_data_request, tick) {
                                        Ok(_) => {},
                                        Err(_) => {
                                            //println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                        }
                                    }
                                    drop(_lock);
                                }
                            }
                        }
                    }
                    //std::thread::sleep(std::time::Duration::from_millis(500));
                },
                Err(_) => {
                    error!("Db Error Fetching Transfers to Broadcast")
                }
            }

        }
    });
}