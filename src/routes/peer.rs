use std::str::FromStr;
use rocket::get;
use network::peer::Peer;
use store;
use crate::env::{get_max_peers, get_min_peers};

#[get("/peers")]
pub fn peers() -> String {
    match store::sqlite::peer::fetch_all_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{:?}", value)
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}

#[get("/peers/add/<address>")]
pub fn add_peer(address: &str) -> String {
    match std::net::SocketAddrV4::from_str(address) {
        Ok(_) => {
            Peer::new(address, None, "");
            match store::sqlite::peer::fetch_peer_by_ip(store::get_db_path().as_str(), address) {
                Ok(peer_map) => { peer_map.get(&"id".to_string()).unwrap().clone() },
                Err(_) => { "Failed To Add Peer".to_string() }
            }
        },
        Err(_) => {
            "Failed To Add Peer".to_string()
        }
    }
}

#[get("/peers/delete/<peer_id>")]
pub fn delete_peer(peer_id: &str) -> String {
    match store::sqlite::peer::blacklist(store::get_db_path().as_str(), peer_id) {
        Ok(_) => { "Peer Deleted".to_string() },
        Err(err) => { 
            println!("Error! : {}", err.to_string());
            "Failed To Deleted Peer".to_string()
        }
    }
}

#[get("/peers/limit/<min_max>/<limit>")]
pub fn set_peer_limit(min_max: &str, limit: u8) -> String {
    
    let current_min = get_min_peers();
    let current_max = get_max_peers();
    
    
    
    if limit < 1 {
        return "Invalid Min/Max Limit".to_string();
    }
    match min_max.to_lowercase().as_str() {
        "min" => {
            if (limit as usize) > current_max {
                return "Can't Set Min More Than Max!".to_string();
            } else {
                std::env::set_var("RUBIC_MIN_PEERS", limit.to_string().as_str());
            }
        },
        "max" => {
            if (limit as usize) < current_min {
                return "Can't Set Max Less Than Min!".to_string();
            } else {
                std::env::set_var("RUBIC_MAX_PEERS", limit.to_string().as_str());
            }
        },
        _ => {
            return "Invalid Min/Max Option".to_string();
        }
    }
    "Ok".to_string()
}