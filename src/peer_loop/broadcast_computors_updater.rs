use std::sync::{Arc, Mutex};
use std::time::Duration;
use network::peers::PeerSet;

pub fn update_broadcast_computors(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            /*
            *
            *   SECTION Fetch BroadcastComputors For This Epoch
            *
            */
            let request = api::QubicApiPacket::get_computors();
            {
                match peer_set.lock().unwrap().make_request(request) {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Failed To Request Computors - {}", err);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(10000));
        }
    });
}