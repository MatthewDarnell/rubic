use std::str::FromStr;
use rocket::get;
use network::peer::Peer;
use store;

#[get("/peers")]
pub fn peers() -> String {
    match store::sqlite::peer::fetch_connected_peers(store::get_db_path().as_str()) {
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

