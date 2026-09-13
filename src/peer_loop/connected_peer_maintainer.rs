use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use logger::{debug, error};
use network::peers::{connect, PeerSet, DEFAULT_CONNECT_TIMEOUT};
use store::{get_db_path, sqlite};
use crate::*;

const SEED_PEERS: [&str; 3] = [
    "31.204.159.155:21841",
    "138.68.105.178:21841",
    "164.90.210.6:21841",
];
// Don't retry an address that just refused us on every pass.
const FAILED_PEER_BACKOFF: Duration = Duration::from_secs(60);
// How many addresses to dial concurrently per pass.
const DIAL_BATCH: usize = 8;

/// Dials every address in parallel (outside any lock) and returns the ones that answered.
fn dial_all(ips: &[String]) -> Vec<(String, std::net::TcpStream)> {
    let handles: Vec<_> = ips.iter().cloned().map(|ip| {
        std::thread::spawn(move || (ip.clone(), connect(ip.as_str(), DEFAULT_CONNECT_TIMEOUT)))
    }).collect();
    let mut connected = Vec::new();
    for handle in handles {
        if let Ok((ip, result)) = handle.join() {
            match result {
                Ok(stream) => connected.push((ip, stream)),
                Err(err) => debug!("Peer {} did not answer: {}", ip, err),
            }
        }
    }
    connected
}

/// Registers dialed sockets until the set holds `wanted` peers; surplus sockets are dropped.
fn register(peer_set: &Arc<Mutex<PeerSet>>, dialed: Vec<(String, std::net::TcpStream)>, wanted: usize) -> usize {
    let mut added = 0;
    let mut lock = peer_set.lock().unwrap();
    for (ip, stream) in dialed {
        if lock.num_peers() >= wanted {
            break; // dropping `stream` closes the surplus socket
        }
        match lock.add_connected_peer(ip.as_str(), stream) {
            Ok(_) => {
                added += 1;
                debug!("Peer {} added ({} in set)", ip, lock.num_peers());
            },
            Err(err) => debug!("Peer {} not added: {}", ip, err),
        }
    }
    added
}

pub fn maintain_peers(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut recently_failed: HashMap<String, Instant> = HashMap::new();

        // Seed peers: dial concurrently, register whatever answered.
        let seeds: Vec<String> = SEED_PEERS.iter().map(|s| s.to_string()).collect();
        register(&peer_set, dial_all(&seeds), env::get_max_peers());

        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let min_peers: usize = env::get_min_peers();
            let max_peers: usize = env::get_max_peers();

            let (num_peers, current_ips) = {
                let lock = peer_set.lock().unwrap();
                (lock.num_peers(), lock.get_peer_ips())
            };

            if num_peers < min_peers {
                debug!("Number Of Peers.({}) Less Than Min Peers.({}). Adding More... (Max of {})", num_peers, min_peers, max_peers);
                let disconnected = match sqlite::peer::fetch_disconnected_peers(get_db_path().as_str()) {
                    Ok(list) => list,
                    Err(_) => {
                        error!("Db Error Fetching Disconnected Peers");
                        continue;
                    }
                };

                let now = Instant::now();
                recently_failed.retain(|_, at| now.duration_since(*at) < FAILED_PEER_BACKOFF);

                // Candidates are ordered by last_responded DESC by the query; skip ones we
                // already hold or that refused us moments ago.
                let candidates: Vec<String> = disconnected.iter()
                    .map(|p| p[1].clone())
                    .filter(|ip| !current_ips.contains(ip) && !recently_failed.contains_key(ip))
                    .take(DIAL_BATCH)
                    .collect();
                if candidates.is_empty() {
                    continue;
                }

                let dialed = dial_all(&candidates);
                let answered: Vec<String> = dialed.iter().map(|(ip, _)| ip.clone()).collect();
                for ip in &candidates {
                    if !answered.contains(ip) {
                        recently_failed.insert(ip.clone(), now);
                    }
                }
                let target = max_peers.max(min_peers);
                let added = register(&peer_set, dialed, target);
                debug!("Dialed {} peers, {} answered, {} added", candidates.len(), answered.len(), added);
            } else if num_peers > max_peers {
                let num_to_disconnect = num_peers - max_peers;
                let ids_to_delete: Vec<String> = {
                    let lock = peer_set.lock().unwrap();
                    lock.get_peer_ids().into_iter().take(num_to_disconnect).collect()
                };
                for id in ids_to_delete {
                    peer_set.lock().unwrap().delete_peer_by_id(id.as_str());
                }
            }
        }
    });
}
