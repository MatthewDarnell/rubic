use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::error;
use network::peers::PeerSet;

pub fn monitor_issued_assets(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        loop {
            /*
            *
            *   SECTION <Query for Known Issued Assets>
            *
            */

            //get my pub keys
            
            let request = api::QubicApiPacket::request_issued_assets(None, None);
            {
                match peer_set.lock().unwrap().make_request(request) {
                    Ok(_) => {
                        //println!("Requested Issued Assets");

                    },
                    Err(err) => error!("{}", err)
                }
            }
            
            
            std::thread::sleep(Duration::from_millis(5 * 1000));

        }
    });
}