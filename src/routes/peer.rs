use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use rocket::get;
use spmc::Receiver;
use uuid::Uuid;
use logger::error;
use network::peer::Peer;
use store;

#[get("/peers")]
pub fn peers() -> String {
    match store::sqlite::crud::peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{:?}", value)
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}

#[get("/peers/add/<address>")]
pub fn add_peer(address: &str) -> String {
    match Ipv4Addr::from_str(address) {
        Ok(_) => {
            let new_peer = Peer::new(address, None, "");
            let id = new_peer.get_id().to_owned();
            match store::sqlite::crud::peer::fetch_peer_by_ip(store::get_db_path().as_str(), address) {
                Ok(_) => { "Peer Added".to_string() },
                Err(_) => { "Failed To Add Peer".to_string() }
            }
        },
        Err(_) => {
            "Failed To Add Peer".to_string()
        }
    }
    
}

