use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crypto::qubic_identities::get_public_key_from_identity;
use logger::{debug, error};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};
use store::sqlite::tick;
use crate::peer_loop::rotation::Rotation;

const OLD_ENTITIES_DELETE_TICK: u32 = 100;
// Each identity's balance is asked for once per interval, and its holdings
// far less often since they change rarely. The identities are spread evenly
// over the interval (a few per one-second pass, round robin) rather than all
// queued at once: a burst of every identity x every peer overflows the request
// backlog cap, and always for the same identities at the end of the list.
const BALANCE_INTERVAL: Duration = Duration::from_secs(10);
const HOLDINGS_INTERVAL: Duration = Duration::from_secs(60);


pub fn update_balances(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut latest_tick: u32 = 0;
        let mut last_deleted_tick: u32 = 0;
        let mut balances = Rotation::new();
        let mut holdings = Rotation::new();
        let mut last_pass = Instant::now();
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            /*
            *
            *   SECTION <Update Latest Tick And Update Balances>
            *
            */
            let temp_latest_tick: u32 = match tick::fetch_latest_tick(get_db_path().as_str()) {
                Ok(tick) => {
                    tick.parse::<u32>().unwrap()
                },
                Err(_) => {
                    0 as u32
                }
            };
            // Nothing new to ask for until the network has moved on.
            if temp_latest_tick <= latest_tick {
                continue;
            }
            let elapsed = last_pass.elapsed();
            last_pass = Instant::now();
            debug("Updating Balances!");
            match sqlite::identity::fetch_all_identities(get_db_path().as_str()) {
                Ok(identities) => {
                    let identities: Vec<String> = identities.into_iter().collect();
                    for identity in balances.take(&identities, BALANCE_INTERVAL, elapsed) {
                        let request = api::QubicApiPacket::get_identity_balance(identity.as_str());
                        match peer_set.lock().unwrap().make_request(request) {
                            Ok(_) => {},
                            Err(err) => error!("{}", err)
                        }
                    }
                    for identity in holdings.take(&identities, HOLDINGS_INTERVAL, elapsed) {
                        let Ok(public_key) = get_public_key_from_identity(&identity) else { continue };
                        let possessed_asset_request = api::QubicApiPacket::request_possessed_assets(&public_key);
                        match peer_set.lock().unwrap().make_request(possessed_asset_request) {
                            Ok(_) => {},
                            Err(err) => error!("{}", err)
                        }
                    }
                },
                Err(err) => {
                    error(format!("Error: {:?}", err).as_str());
                }
            }

            if latest_tick - last_deleted_tick > OLD_ENTITIES_DELETE_TICK {
                debug!("Deleting Before Tick {}", latest_tick - OLD_ENTITIES_DELETE_TICK);
                match sqlite::identity::delete_all_response_entities_before_tick(get_db_path().as_str(), latest_tick - OLD_ENTITIES_DELETE_TICK) {
                    Ok(_) => {
                        match sqlite::asset::delete_all_assets_before_tick(get_db_path().as_str(), latest_tick - OLD_ENTITIES_DELETE_TICK) {
                            Ok(_) => {
                            },
                            Err(err) => {
                                eprintln!("Error Deleting Assets! {}", err);
                            }
                        }
                    },
                    Err(_err) => {
                        println!("Failed To Delete Old Entities {}", _err);
                    }
                }
                last_deleted_tick = latest_tick;
            }
            latest_tick = temp_latest_tick;
        }
    });
}
