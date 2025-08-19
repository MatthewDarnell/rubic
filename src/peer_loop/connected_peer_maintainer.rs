use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::{debug, error};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};
use crate::*;

pub fn maintain_peers(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let peer_ips = vec![
            "31.204.159.155:21841",
            "138.68.105.178:21841",
            "164.90.210.6:21841"
        ];
        for ip in peer_ips {
            debug!("Adding Peer {}", ip);
            peer_set.lock().unwrap().add_peer(ip).ok();
            debug!("Peer Added");
        }

        loop {
            std::thread::sleep(Duration::from_millis(1000));
            /*
            *
            *   SECTION <Connect To New Peers As Needed>
            *
            */
            //Try To Spin up New Peers Until We Reach The Min Number
            let min_peers: usize = env::get_min_peers();
            let max_peers: usize = env::get_max_peers();

            let _lock = peer_set.lock().unwrap();
            let num_peers: usize = _lock.get_peers().len();
            std::mem::drop(_lock);
            
            if num_peers < min_peers {
                debug!("Number Of Peers.({}) Less Than Min Peers.({}). Adding More... (Max of {})", num_peers, min_peers, max_peers);
                match sqlite::peer::fetch_disconnected_peers(get_db_path().as_str()) {
                    Ok(disconnected_peers) => {
                        let num_to_add = max_peers - min_peers;
                        let mut count = 0;
                        debug!("Fetched {} Disconnected Peers", disconnected_peers.len());
                        for p in disconnected_peers {
                            let peer_id = &p[0];
                            let peer_ip = &p[1];
                            {
                                match peer_set.lock().unwrap().add_peer(peer_ip.as_str()) {
                                    Ok(_) => {
                                        debug!("Peer.({}) Added {} ({} left)", peer_ip.as_str(), peer_id.as_str(), num_to_add - count);
                                        count = count + 1;
                                        if count > num_to_add {
                                            break;
                                        }
                                    },
                                    Err(err) => {
                                        debug!("Failed To Add Peer.({}) : ({:?})", peer_ip.as_str(), err);
                                    }
                                }
                            }
                        }
                    },
                    Err(_) => error!("Db Error Fetching Disconnected Peers")
                }
            } else if num_peers > max_peers {
                let num_to_disconnect = num_peers - max_peers;
                let mut ids_to_delete: Vec<String>  = Vec::with_capacity(num_to_disconnect);

                {
                    for (index, peer) in peer_set.lock().unwrap().get_peers().iter().enumerate() {
                        let id = peer.get_id().clone();
                        if index < num_to_disconnect {
                            ids_to_delete.push(id);
                        }
                    }
                }
                
                for id in ids_to_delete {
                    {
                        peer_set.lock().unwrap().delete_peer_by_id(id.as_str());
                    }
                }
            }
        }
    });
}