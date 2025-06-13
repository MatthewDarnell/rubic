use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::{debug, error};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};
use store::sqlite::tick;

const OLD_ENTITIES_DELETE_TICK: u32 = 100;


pub fn update_balances(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut latest_tick: u32 = 0;
        let mut last_deleted_tick: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(2500));
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
            if temp_latest_tick > latest_tick {
                debug("Updating Balances!");
                match sqlite::identity::fetch_all_identities(get_db_path().as_str()) {
                    Ok(identities) => {
                        for identity in identities {
                            let request = api::QubicApiPacket::get_identity_balance(identity.as_str());
                            {
                                match peer_set.lock().unwrap().make_request(request) {
                                    Ok(_) => {},
                                    Err(err) => error!("{}", err)
                                }
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
                        Ok(_) => {},
                        Err(_err) => {
                            println!("Failed To Delete Old Entities {}", _err);
                        }
                    }   
                    last_deleted_tick = latest_tick;
                }
                latest_tick = temp_latest_tick;
            }
        }
    });
}