use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::error;
use network::peers::PeerSet;

pub fn monitor_latest_tick(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            // Once a second to every peer; the highest answer keeps the wallet's
            // tick within a tick or two of the network.
            std::thread::sleep(Duration::from_millis(1000));
            /*
            *
            *   SECTION <Update Latest Tick And Update Balances>
            *
            */

            let request = api::QubicApiPacket::get_latest_tick();
            {
                match peer_set.lock().unwrap().make_request(request) {
                    Ok(_) => {},
                    Err(err) => error!("{}", err)
                }
            }
            
        }
    });
}