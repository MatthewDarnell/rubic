use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;
use api::RequestedEntity;
use logger::{debug, error, info, trace};
use network::peers::PeerSet;
use store::get_db_path;
use store::sqlite::crud;
use crate::env;

pub fn start_peer_set_thread(tx: &mpsc::Sender<std::collections::HashMap<String, String>>, rx: mpsc::Receiver<std::collections::HashMap<String, String>>) {
    {
        let tx = tx.clone();
        let rx = rx;
        std::thread::spawn(move || {
            println!("PEER SET THREAD SPAWNED");
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


            let mut latest_tick: u32 = match crud::fetch_latest_tick(get_db_path().as_str()) {
                Ok(tick) => {
                    tick.parse::<u32>().unwrap()
                },
                Err(_) => {
                    0 as u32
                }
            };

            let mut tick_updated: bool;

            //Main Thread Loop
            loop {
                
                /*
                *
                *   SECTION <Update Latest Tick And Update Balances>
                *
                */
                
                
                let request = api::QubicApiPacket::get_latest_tick();
                match peer_set.make_request(request) {
                    Ok(_) => {},
                    //Ok(_) => println!("{:?}", request.response_data),
                    Err(err) => error!("{}", err)
                }
                tick_updated = false;
                let temp_latest_tick: u32 = match crud::fetch_latest_tick(get_db_path().as_str()) {
                    Ok(tick) => {
                        tick.parse::<u32>().unwrap()
                    },
                    Err(_) => {
                        0 as u32
                    }
                };
                if temp_latest_tick > latest_tick {
                    debug!("Tick Updated! {} -> {}", latest_tick, temp_latest_tick);
                    latest_tick = temp_latest_tick;
                    tick_updated = true;
                }

                //Update Balances For All Stored Identities
                if tick_updated == true {
                    trace!("Updating Balances!");
                    match crud::fetch_all_identities(get_db_path().as_str()) {
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
                
                for peer in peer_set.get_peer_ids() {
                    match crud::peer::fetch_peer_by_id(get_db_path().as_str(), peer.as_str()) {
                        Ok(temp_peer) => {
                            if let Some(connected) = temp_peer.get(&"connected".to_string()) {
                                if connected.as_str() != "1" {
                                    //println!("{:?}", &temp_peer);
                                    error!("Is Peer {} connected? {}", peer.as_str(), &connected);
                                    peer_set.delete_peer_by_id(peer.as_str());
                                }
                            }
                        },
                        Err(err) => {
                            error!("Error Fetching Peer {} By Id! {}", peer.as_str(), err);
                            //panic!("{}", err)
                        }
                    }
                }



                /*
                *
                *   SECTION <Connect To New Peers As Needed>
                *
                */
                
                //Try To Spin up New Peers Until We Reach The Min Number
                let min_peers: usize = env::get_min_peers();
                let num_peers: usize = peer_set.get_peers().len();
                if num_peers < min_peers {
                    debug!("Number Of Peers.({}) Less Than Min Peers.({}). Adding More...", num_peers, min_peers);
                    match crud::peer::fetch_disconnected_peers(get_db_path().as_str()) {
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
                *   SECTION <Look For Pending Transfers>
                *
                */
                
                match crud::fetch_transfers_to_broadcast(get_db_path().as_str()) {
                    Ok(transfers_to_broadcast) => {
                        println!("Found {} Transfers To Broadcast", transfers_to_broadcast.len());
                    },
                    Err(_) => {}
                }
            }
        });   
    }
    
}