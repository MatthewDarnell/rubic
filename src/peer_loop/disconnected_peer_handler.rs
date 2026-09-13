use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::{debug, error};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};

const DISCONNECT_PEER_TIMEOUT: u64 = 50 * 1000;

/// Reconciles the in-memory peer set with the database: anything a worker thread
/// flagged dead, or that another component marked disconnected in the DB (e.g. a
/// malformed response, or the user removing it), is dropped so it can be redialed.
pub fn handle_disconnected_peers(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(DISCONNECT_PEER_TIMEOUT));

            let pruned = peer_set.lock().unwrap().prune_disconnected();
            if pruned > 0 {
                debug!("Pruned {} peers whose sockets had closed", pruned);
            }

            let ids: Vec<String> = peer_set.lock().unwrap().get_peer_ids();
            for id in ids {
                match sqlite::peer::fetch_peer_by_id(get_db_path().as_str(), id.as_str()) {
                    Ok(row) => {
                        let connected = row.get("connected").map(|v| v == "1").unwrap_or(false);
                        let removed = row.get("whitelisted").map(|v| v == "-1").unwrap_or(false);
                        if !connected || removed {
                            debug!("Dropping peer {} (connected={}, removed={})", id, connected, removed);
                            peer_set.lock().unwrap().delete_peer_by_id(id.as_str());
                        }
                    },
                    Err(err) => {
                        error!("Error Fetching Peer {} By Id! {}", id.as_str(), err);
                    }
                }
            }
        }
    });
}
