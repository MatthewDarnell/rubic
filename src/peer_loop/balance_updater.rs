use std::sync::{Arc, Mutex};
use std::time::Duration;
use logger::{error, trace};
use network::peers::PeerSet;
use store::{get_db_path, sqlite};
use store::sqlite::tick;

const OLD_ENTITIES_DELETE_TICK: u32 = 100;


pub fn update_balances(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let latest_tick: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(10000));
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
                trace!("Updating Balances!");
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
                        error!("Error: {:?}", err);
                    }
                }
                
                if latest_tick > OLD_ENTITIES_DELETE_TICK {
                    match sqlite::identity::delete_all_response_entities_before_tick(get_db_path().as_str(), latest_tick - OLD_ENTITIES_DELETE_TICK) {
                        Ok(_) => {},
                        Err(_err) => {
                            println!("Failed To Delete Old Entities {}", _err);
                        }
                    }   
                }
            }
        }
    });
}