use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::error;
use network::peers::PeerSet;
use store::{get_db_path, sqlite};

const DISCONNECT_PEER_TIMEOUT: u64 = 50 * 1000;

pub fn handle_disconnected_peers(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(DISCONNECT_PEER_TIMEOUT));
            let mut ids: Vec<String> = Vec::new();
            {
                ids = peer_set.lock().unwrap().get_peer_ids();
            }
            
            for peer in ids {
                match sqlite::peer::fetch_peer_by_id(get_db_path().as_str(), peer.as_str()) {
                    Ok(_) => { 
                        peer_set.lock().unwrap().delete_peer_by_id(peer.as_str());
                    },
                    Err(err) => {
                        error!("Error Fetching Peer {} By Id! {}", peer.as_str(), err);
                        //panic!("{}", err)
                    }
                }
            }

        }
    });
}