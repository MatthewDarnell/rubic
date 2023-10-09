#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;
use spmc::Receiver;
use rocket::{get, routes};
use uuid::Uuid;
use store;
use identity;
#[get("/info")]
pub fn info() -> String {
    match store::sqlite::crud::Peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{}", value.len())
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}
#[get("/peers")]
pub fn peers() -> String {
    match store::sqlite::crud::Peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{:?}", value)
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}

#[get("/balance/<address>")]
pub fn balance(address: &str) -> String {
    match store::sqlite::crud::fetch_balance_by_identity(store::get_db_path().as_str(), address) {
        Ok(value) => { format!("{:?}", value) },
        Err(error) => format!("{}", error)
    }
}


#[get("/peers/add/<address>")]
pub fn add_peer(address: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    println!("Locking Mutex");
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    println!("Dropped Mutex Lock");
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_peer".to_string());
    map.insert("peer_ip".to_string(), address.to_string());
    let request_id: String = Uuid::new_v4().to_string();
    map.insert("message_id".to_string(), request_id.clone());
    tx.send(map).unwrap();
    let mut index = 0;
    loop {
        index = index + 1;
        if index > 5 {
            return format!("Timed Out")
        }
        std::thread::sleep(Duration::from_secs(1));
        match rx.try_recv() {
            Ok(response) => {
                println!("{:?}", &response);
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(err) => {
                println!("got error {:?}", &err);
               // return format!("{}", err.to_string());
            }
        }
    }




    //format!("Timed Out")
}

#[get("/identities")]
pub fn get_identities() -> String {
    match store::sqlite::crud::fetch_all_identities(store::get_db_path().as_str()) {
        Ok(v) => format!("{:?}", v),
        Err(err) => format!("{}", err)
    }
}

#[get("/identity/from_seed/<seed>")]
pub fn get_identity_from_seed(seed: &str) -> String {
    let i: identity::Identity = identity::Identity::new(seed, "");
    format!("{}", i.identity.as_str())
}

#[get("/identity/add/<identity>")]
pub fn add_identity(identity: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    println!("Locking Mutex");
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    println!("Dropped Mutex Lock");
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_identity".to_string());
    map.insert("identity".to_string(), identity.to_string());
    let request_id: String = Uuid::new_v4().to_string();
    map.insert("message_id".to_string(), request_id.clone());
    tx.send(map).unwrap();
    let mut index = 0;
    loop {
        index = index + 1;
        if index > 5 {
            return format!("Timed Out")
        }
        std::thread::sleep(Duration::from_secs(1));
        match rx.try_recv() {
            Ok(response) => {
                println!("{:?}", &response);
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(err) => {
                println!("got error {:?}", &err);
                // return format!("{}", err.to_string());
            }
        }
    }




    //format!("Timed Out")
}