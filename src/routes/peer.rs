use std::str::FromStr;
use rocket::get;
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
    match std::net::SocketAddrV4::from_str(address) {
        Ok(_) => {
            Peer::new(address, None, "");
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

