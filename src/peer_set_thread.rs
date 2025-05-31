use std::str::FromStr;
use std::sync::mpsc;
use api::transfer::TransferTransaction;
use crypto::qubic_identities::{get_identity, get_public_key_from_identity};
use logger::{debug, error, info, trace};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};
use store::sqlite::{tick, transfer};
use store::sqlite::transfer::set_transfer_as_broadcast;
use crate::env;
use base64::{Engine as _, engine::general_purpose};

pub fn start_peer_set_thread(_: &mpsc::Sender<std::collections::HashMap<String, String>>, _: mpsc::Receiver<std::collections::HashMap<String, String>>) {
    {
        std::thread::spawn(move || {

                /*
                *
                *   SECTION <Add Initial Seeded Peers, Figure Out Our Latest Known Tick>
                *
                */

            let peer_ips = vec![
                "62.2.98.75:21841",
                "185.117.0.116:21841",
                "144.2.106.163:21841"
            ];
            debug!("Creating Peer Set");

            let mut peer_set = PeerSet::new();
            for ip in peer_ips {
                debug!("Adding Peer {}", ip);
                peer_set.add_peer(ip).ok();
                debug!("Peer Added");
            }


            let mut latest_tick: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
                Ok(tick) => {
                    tick.parse::<u32>().unwrap()
                },
                Err(_) => {
                    0 as u32
                }
            };

            let mut time = std::time::SystemTime::now();

            const NUM_SECONDS_FETCH_COMPUTORS: u64 = 86400;   //Once Per Day
            let mut fetched_computor_time = std::time::SystemTime::now();
            let mut have_fetched_computors: bool = false;


            let mut update_balances: bool = false;


            let mut delete_all_peers: u32 = 0;
            const NUM_LOOPS_DELETE_ALL_PEERS: u32 = 60;


            const NUM_TICKS_UPDATED_CHECK_BALANCES: u32 = 15;

            //Main Thread Loop
            loop {


                /*
                *
                *   SECTION <Connect To New Peers As Needed>
                *
                */

                //Try To Spin up New Peers Until We Reach The Min Number
                let min_peers: usize = env::get_min_peers();
                let num_peers: usize = peer_set.get_peers().len();
                if num_peers < min_peers {
                    //println!("Have {} Peers. Connecting To More.", num_peers);
                    debug!("Number Of Peers.({}) Less Than Min Peers.({}). Adding More...", num_peers, min_peers);
                    match sqlite::peer::fetch_disconnected_peers(get_db_path().as_str()) {
                        Ok(disconnected_peers) => {
                            debug!("Fetched {} Disconnected Peers", disconnected_peers.len());
                            for p in disconnected_peers {
                                let peer_id = &p[0];
                                let peer_ip = &p[1];
                                match peer_set.add_peer(peer_ip.as_str()) {
                                    Ok(_) => {
                                        debug!("Peer.({}) Added {}", peer_ip.as_str(), peer_id.as_str());
                                    },
                                    Err(err) => {
                                        debug!("Failed To Add Peer.({}) : ({:?})", peer_ip.as_str(), err);
                                    }
                                }
                            }
                        },
                        Err(_) => error!("Db Error Fetching Disconnected Peers")
                    }
                }


                /*
                *
                *   SECTION <Update Latest Tick And Update Balances>
                *
                */

                //We have to sleep to avoid being rate limited
                                                                                 // by our peers

                let curr_time = std::time::SystemTime::now();
                if curr_time >= time + std::time::Duration::from_millis(900) {
                    time = curr_time;
                    let request = api::QubicApiPacket::get_latest_tick();
                    match peer_set.make_request(request) {
                        Ok(_) => {},
                        Err(err) => error!("{}", err)
                    }
                }

                let temp_latest_tick: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
                    Ok(tick) => {
                        tick.parse::<u32>().unwrap()
                    },
                    Err(_) => {
                        0 as u32
                    }
                };


                if temp_latest_tick > latest_tick { //quick updates
                    delete_all_peers = delete_all_peers + 1;
                }
                
                if temp_latest_tick > (latest_tick) + NUM_TICKS_UPDATED_CHECK_BALANCES {  //lagging updates
                    debug!("Tick Updated! {} -> {}", latest_tick, temp_latest_tick);
                    latest_tick = temp_latest_tick;
                    update_balances = true;
                }

                //Update Balances For All Stored Identities
                if update_balances == true {
                    //println!("Updating Balances");
                    update_balances = false;
                    trace!("Updating Balances!");
                    match sqlite::identity::fetch_all_identities(get_db_path().as_str()) {
                        Ok(identities) => {
                            for identity in identities {
                                let request = api::QubicApiPacket::get_identity_balance(identity.as_str());
                                match peer_set.make_request(request) {
                                    Ok(_) => {},
                                    Err(err) => error!("{}", err)
                                }
                            }
                        },
                        Err(err) => {
                            error!("Error: {:?}", err);
                        }
                    }
                } else {
                }


                /*
                *
                *   SECTION <Handle Disconnected Peers>
                *
                */

                if delete_all_peers > NUM_LOOPS_DELETE_ALL_PEERS {
                    //println!("Dis/Re-connecting From All Peers");
                }
                for peer in peer_set.get_peer_ids() {
                    match sqlite::peer::fetch_peer_by_id(get_db_path().as_str(), peer.as_str()) {
                        Ok(temp_peer) => {
                            if delete_all_peers > NUM_LOOPS_DELETE_ALL_PEERS {
                                peer_set.delete_peer_by_id(peer.as_str());
                            } else {
                                if let Some(connected) = temp_peer.get(&"connected".to_string()) {
                                    if connected.as_str() != "1" {
                                        //println!("{:?}", &temp_peer);
                                        error!("Is Peer {} connected? {}", peer.as_str(), &connected);
                                        peer_set.delete_peer_by_id(peer.as_str());
                                    }
                                }   
                            }
                        },
                        Err(err) => {
                            error!("Error Fetching Peer {} By Id! {}", peer.as_str(), err);
                            //panic!("{}", err)
                        }
                    }
                }
                
                if delete_all_peers > 25 {
                    delete_all_peers = 0;
                }



                /*
                *
                *   SECTION <Look For Pending Transfers To Broadcast>
                *
                */

                match transfer::fetch_transfers_to_broadcast(get_db_path().as_str()) {
                    Ok(transfers_to_broadcast) => {
                        for transfer_map in transfers_to_broadcast {
                            let source_id = transfer_map.get("source").unwrap();
                            let dest_id = transfer_map.get("destination").unwrap();

                            let amount = transfer_map.get("amount").unwrap();
                            let tick = transfer_map.get("tick").unwrap();

                            let amt: u64 = amount.parse().unwrap();
                            let tck: u32 = tick.parse().unwrap();

                            let source_pub_key = get_public_key_from_identity(source_id).unwrap();
                            let des_pub_key = get_public_key_from_identity(dest_id).unwrap();

                            let signature = transfer_map.get("signature").unwrap();
                            let sig_arr = hex::decode(signature).unwrap();
                            let txid = transfer_map.get("txid").unwrap();

                            let tx = TransferTransaction::from_signed_data(
                                &source_pub_key,
                                &des_pub_key,
                                amt,
                                tck,
                                0,
                                0,
                                sig_arr.as_slice()
                            );

                            let broadcast = api::QubicApiPacket::broadcast_transaction(&tx);

                            match peer_set.make_request(broadcast) {
                                Ok(_) => {
                                    match set_transfer_as_broadcast(get_db_path().as_str(), txid.as_str()) {
                                        Ok(_) => {
                                            //println!("Transaction {} Broadcast", txid);
                                            info!("Transaction {} Broadcast", txid);
                                        },
                                        Err(err) => {
                                            error!("Failed To Set Transaction <{}> as Broadcast! ({})", txid, err);
                                        }
                                    }
                                },
                                Err(err) => error!("{}", err)
                            }

                        }
                    },
                    Err(_) => {}
                }



                /*
                *
                *   SECTION <Look For Broadcasted Transfers That Are Executed To Confirm>
                *
                */

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
                                            peer_set.make_request(api::QubicApiPacket::request_quorum_tick(tick)).expect("Failed To Request Tick Data!");
                                        } else {    //We have the Tick and tx_digests hash but not the full tx_digests. Fetch TickData
                                            if tx_digests.len() < 8 {
                                                //println!("We Have Tick But No Digests. Fetching Tick {} Data!", tick);
                                                peer_set.make_request(api::QubicApiPacket::request_tick_data(tick)).expect("Failed To Request Tick Data!");
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
                                        peer_set.make_request(api::QubicApiPacket::request_quorum_tick(tick)).expect("Failed To Request Tick!");
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
                                        peer_set.make_request(api::QubicApiPacket::request_quorum_tick(tick)).expect("Failed To Request Tick!");
                                    }
                                }
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    },
                    Err(_) => error!("Db Error Fetching Transfers to Broadcast")
                }

                /*
                *
                *   SECTION TODO: <Fetch BroadcastComputors For This Epoch>
                *
                */


                if !have_fetched_computors || curr_time >= fetched_computor_time + std::time::Duration::from_secs(NUM_SECONDS_FETCH_COMPUTORS) {
                    let request = api::QubicApiPacket::get_computors();
                    match peer_set.make_request(request) {
                        Ok(_) => {
                            if have_fetched_computors {
                                std::thread::sleep(std::time::Duration::from_millis(5000));
                            }
                            have_fetched_computors = true;
                            fetched_computor_time = curr_time;
                            //println!("Request Computors");
                        },
                        Err(err) => {
                            println!("Failed To Request Computors - {}", err);
                        }
                    }
                }

            }
        });
    }
    
}