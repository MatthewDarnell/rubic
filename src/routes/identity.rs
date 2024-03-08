use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use rocket::get;
use spmc::Receiver;
use uuid::Uuid;
use store;

#[get("/balance/<address>")]
pub fn balance(address: &str) -> String {
    match store::sqlite::crud::fetch_balance_by_identity(store::get_db_path().as_str(), address) {
        Ok(value) => { format!("{:?}", value) },
        Err(error) => format!("{}", error)
    }
}

#[get("/identities")]
pub fn get_identities() -> String {
    match store::sqlite::crud::fetch_all_identities_full(store::get_db_path().as_str()) {
        Ok(v) => {
            let mut response: Vec<String> = vec![];
            for identity in &v {
                let encrypted: String = match identity.encrypted {
                    true => "true".to_string(),
                    _ => "false".to_string()
                };
                response.push(identity.identity.clone());
                response.push(encrypted);
            }
            format!("{:?}", response)
        },
        Err(err) => format!("{}", err)
    }
}

#[get("/identity/from_seed/<seed>")]
pub fn get_identity_from_seed(seed: &str) -> String {
    let i: identity::Identity = identity::Identity::new(seed);
    format!("{}", i.identity.as_str())
}

#[get("/identity/new/<password>")]
pub fn create_random_identity(password: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_identity".to_string());



    let mut seed_string: String = String::from("");
    while seed_string.len() < 55 {
        let temp_seed: Vec<u8> = crypto::random::random_bytes(32);
        for val in temp_seed {
            if val >= 97 && val <= 122 {
                seed_string += char::from(val).to_string().as_str();
                if seed_string.len() >= 55 {
                    break;
                }
            }
        }

    }
    if password.len() > 4 {
        map.insert("password".to_string(), password.to_string());
    }
    map.insert("seed".to_string(), seed_string);

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


#[get("/identity/add/<seed>")]
pub fn add_identity(seed: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_identity".to_string());
    map.insert("seed".to_string(), seed.to_string());
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

#[get("/identity/add/<seed>/<password>")]
pub fn add_identity_with_password(seed: &str, password: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
    let lock = mtx.lock().unwrap();
    let tx = lock.clone();
    drop(lock);

    let lock2 = responses.lock().unwrap();
    let rx = lock2.clone();
    drop(lock2);


    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("method".to_string(), "add_identity".to_string());
    map.insert("seed".to_string(), seed.to_string());
    if password.len() > 1 {
        map.insert("password".to_string(), password.to_string());
    }
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
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(_) => {
            }
        }
    }
}
