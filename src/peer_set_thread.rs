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
                let request = api::QubicApiPacket::get_latest_tick();
                match peer_set.make_request(request) {
                    Ok(_) => {},
                    //Ok(_) => println!("{:?}", request.response_data),
                    Err(err) => error!("{}", err)
                }
                //std::thread::sleep(delay);
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
                                    //Ok(_) => println!("{:?}", request.response_data),
                                    Err(err) => error!("{}", err)
                                }
                                //std::thread::sleep(delay);
                            }
                        },
                        Err(err) => {
                            error!("Error: {:?}", err);
                        }
                    }
                    //println!("Finished Updating Balances");
                } else {
                    //println!("Not Updating Balances!");
                }

                //Try To Receive Messages From Server Api
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(map) => {
                        println!("[PeerSetThread] Received API Route Request");
                        if let Some(method) = map.get(&"method".to_string()) {
                            debug!("Api got request method=[{}]", method.as_str());
                            if method == &"transfer".to_string() {
                                let message_id = map.get(&"message_id".to_string()).unwrap();
                                let mut response: HashMap<String, String> = HashMap::new();

                                let source = map.get(&"source".to_string()).unwrap();
                                let dest = map.get(&"dest".to_string()).unwrap();
                                let amount = map.get(&"amount".to_string()).unwrap();
                                let expiration = map.get(&"expiration".to_string()).unwrap();

                                let mut id: identity::Identity = match store::sqlite::crud::fetch_identity(get_db_path().as_str(), source.as_str()) {
                                    Ok(identity) => identity,
                                    Err(_) => {
                                        response.insert("message_id".to_string(), message_id.to_string());
                                        response.insert("status".to_string(), "Unknown Source Identity!".to_string());
                                        error!("Failed To Make Transfer, Unknown Identity {}", source.as_str());
                                        tx.send(response).unwrap();
                                        continue;
                                    }
                                };

                                if id.encrypted {
                                    if let Some(pass) = map.get(&"password".to_string()) {
                                        id = match crud::master_password::get_master_password(get_db_path().as_str()) {
                                            Ok(master_password) => {
                                                //println!("{} : {:?}", pass.as_str(), &master_password);
                                                match crypto::passwords::verify_password(pass.as_str(), master_password[1].as_str()) {
                                                    Ok(verified) => {
                                                        if !verified {
                                                            response.insert("message_id".to_string(), message_id.to_string());
                                                            response.insert("status".to_string(), "Invalid Password!".to_string());
                                                            error!("Failed To Create Transfer; Invalid Password");
                                                            tx.send(response).unwrap();
                                                            continue;
                                                        } else {
                                                            match id.decrypt_identity(pass.as_str()) {
                                                                Ok(identity) => identity,
                                                                Err(_) => {
                                                                    response.insert("message_id".to_string(), message_id.to_string());
                                                                    response.insert("status".to_string(), "Invalid Password For This Identity!".to_string());
                                                                    error!("Failed To Create Transfer; Invalid Password For This Identity");
                                                                    tx.send(response).unwrap();
                                                                    continue;
                                                                }
                                                            }
                                                        }
                                                    },
                                                    Err(_) => {
                                                        response.insert("message_id".to_string(), message_id.to_string());
                                                        response.insert("status".to_string(), "Failed To Verify Master Password Vs Supplied Password!".to_string());
                                                        error!("Failed To Verify Master Password Vs Supplied Password");
                                                        tx.send(response).unwrap();
                                                        continue;
                                                    }
                                                }
                                            },
                                            Err(_) => {
                                                response.insert("message_id".to_string(), message_id.to_string());
                                                response.insert("status".to_string(), "Identity Is Encrypted, Yet No Master Password Set! Weird!".to_string());
                                                error!("Identity Is Encrypted, Yet No Master Password Set! Weird");
                                                tx.send(response).unwrap();
                                                continue;
                                            }
                                        };
                                    } else {
                                        response.insert("message_id".to_string(), message_id.to_string());
                                        response.insert("status".to_string(), "Must Enter A Password!".to_string());
                                        error!("Failed To Decrypt Password For Transfer; No Password Supplied");
                                        tx.send(response).unwrap();
                                        continue;
                                    }
                                } else {
                                    debug!("Creating Transfer, Wallet Is Not Encrypted!");
                                }
                                let amt: u64 = amount.parse().unwrap();
                                let tck: u32 = expiration.parse().unwrap();

                                //info(format!("Creating Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &id.identity.as_str(), amt.to_string().as_str(), dest.as_str(), tck.to_string().as_str()).as_str());
                                info!("Creating Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &id.identity.as_str(), amt.to_string().as_str(), dest.as_str(), tck.to_string().as_str());
                                let transfer_tx = api::transfer::TransferTransaction::from_vars(&id, &dest, amt, tck);
                                response.insert("message_id".to_string(), message_id.to_string());
                                response.insert("status".to_string(), "Transfer Sent!".to_string());

                                let request = api::QubicApiPacket::broadcast_transaction(&transfer_tx);
                                match peer_set.make_request(request) {
                                    Ok(_) => { info!("Transaction Sent!"); },
                                    //Ok(_) => println!("{:?}", request.response_data),
                                    Err(err) => error!("{}", err)
                                }
                                tx.send(response).unwrap();
                                continue;
                            }
                        }
                    },
                    Err(_) => {
                        //No Error, just timed out due to no web requests
                        //println!("Read TimeOut Error: {}", err.to_string());
                    }
                }


                //Check if Any Peers Have Been Set As 'Disconnected'. This means The TcpStream Connection Terminated. Delete them.
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
            }
        });   
    }
    
}