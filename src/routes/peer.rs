use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use rocket::get;
use spmc::Receiver;
use uuid::Uuid;
use logger::error;
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
pub fn add_peer(address: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_peer".to_string());
    map.insert("peer_ip".to_string(), address.to_string());
    let request_id: String = Uuid::new_v4().to_string();
    map.insert("message_id".to_string(), request_id.clone());
    match tx.send(map) {
        Ok(_) => {},
        Err(err) => {
            error(format!("Failed To Send Response From Peers Add Address! : {:?}", err).as_str());
        }
    }
    let mut index = 0;
    loop {
        index = index + 1;
        if index > 5 {
            return format!("Timed Out")
        }
        std::thread::sleep(Duration::from_secs(1));
        match rx.try_recv() {
            Ok(response) => {
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(_) => {
                //println!("got error {:?}", &err);
                // return format!("{}", err.to_string());
            }
        }
    }
}

