use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use logger::{debug, error};
use network::peers::PeerSet;
use smart_contract::qx::entity_orders::EntityOrdersRequest;
use smart_contract::qx::QxFunctions;
use store::{get_db_path, sqlite};
use crate::peer_loop::rotation::Rotation;

/// How often each identity's resting QX orders are asked for. Two small
/// requests per identity; the answer is the contract's own list, so "my open
/// orders" no longer depends on how fresh the asset books are. The identities
/// are spread over the interval, a few per pass, rather than queued in a burst.
const POLL_INTERVAL: Duration = Duration::from_secs(10);
const PASS: Duration = Duration::from_secs(1);

pub fn monitor_qx_entity_orders(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut rotation = Rotation::new();
        let mut last_pass = Instant::now();
        loop {
            std::thread::sleep(PASS);
            let identities: Vec<String> = match sqlite::identity::fetch_all_identities(get_db_path().as_str()) {
                Ok(list) => list.into_iter().collect(),
                Err(err) => {
                    error!("Failed to load identities for the QX open-orders monitor: {:?}", err);
                    continue;
                }
            };
            let elapsed = last_pass.elapsed();
            last_pass = Instant::now();
            for identity in rotation.take(&identities, POLL_INTERVAL, elapsed) {
                for function in [QxFunctions::QxGetEntityAskOrder, QxFunctions::QxGetEntityBidOrder] {
                    let request = match EntityOrdersRequest::new(function, identity.as_str(), 0) {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Bad identity {} for QX entity orders: {}", identity, err);
                            break;
                        }
                    };
                    match peer_set.lock().unwrap().make_request(api::QubicApiPacket::get_entity_qx_orders(&request)) {
                        Ok(_) => {},
                        Err(err) => debug!("{}", err),
                    }
                }
            }
        }
    });
}
