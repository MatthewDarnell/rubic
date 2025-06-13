use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use base64::Engine;
use base64::engine::general_purpose;
use crypto::qubic_identities::get_identity;
use logger::error;
use network::peers::PeerSet;
use store::get_db_path;
use store::sqlite::{tick, transfer};

pub fn confirm_transactions(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(1000));
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
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        let _tick = transfer.get("tick").unwrap();
                        let txid = transfer.get("txid").unwrap();
                        //println!("looking for tx {} at tick {}", txid, _tick);
                        let tick = u32::from_str(_tick.as_str()).unwrap();
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
                                            match _lock.make_request(api::QubicApiPacket::request_quorum_tick(tick)) {
                                                Ok(_) => {},
                                                Err(_) => {
                                                    println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                                }
                                            }
                                            drop(_lock);
                                        }
                                    } else {    //We have the Tick and tx_digests hash but not the full tx_digests. Fetch TickData
                                        if tx_digests.len() < 8 {
                                            //println!("We Have Tick But No Digests. Fetching Tick {} Data!", tick);
                                            {
                                                let mut _lock = peer_set.lock().unwrap();
                                                match _lock.make_request(api::QubicApiPacket::request_tick_data(tick)) {
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
                                    //println!("Requesting Tick To Validate: {}", tick);
                                    std::thread::sleep(std::time::Duration::from_millis(750));
                                    {
                                        let mut _lock = peer_set.lock().unwrap();
                                        match _lock.make_request(api::QubicApiPacket::request_quorum_tick(tick)) {
                                            Ok(_) => {},
                                            Err(_) => {
                                                println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                            }
                                        }
                                        drop(_lock);
                                    }
                                }
                            },
                            Err(_) => {
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                //We don't have this tick, fetch it, unless it's too old
                                if latest_tick - tick > 350000 {
                                    match transfer::set_broadcasted_transfer_as_failure(get_db_path().as_str(), txid.as_str()) {
                                        Ok(_) => {
                                            println!("Transaction <{}> Too Old. Marking Failed.", txid);
                                        },
                                        Err(err) => {
                                            println!("Failed To Set Old Transaction As Failed {} ({})", txid.as_str(), err);
                                        }
                                    }
                                } else {
                                    //println!("Fetching tick {}", tick);
                                    {
                                        let mut _lock = peer_set.lock().unwrap();
                                        match _lock.make_request(api::QubicApiPacket::request_quorum_tick(tick)) {
                                            Ok(_) => {},
                                            Err(_) => {
                                                println!("TransactionConfirmer: Failed To Request Quorum Tick!");
                                            }
                                        }
                                        drop(_lock);
                                    }
                                }
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                },
                Err(_) => {
                    error!("Db Error Fetching Transfers to Broadcast")
                }
            }

        }
    });
}