
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;
use spmc::Receiver;
use rocket::get;
use uuid::Uuid;
use store;
use identity;
use crypto;

#[get("/tick")]
pub fn latest_tick() -> String {
    match store::sqlite::crud::fetch_latest_tick(store::get_db_path().as_str()) {
        Ok(tick) => format!("{}", tick),
        Err(err) => format!("Error! : {}", err.to_string())
    }
}

#[get("/info")]
pub fn info() -> String {
    match store::sqlite::crud::peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{}", value.len())
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}
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
               //println!("got error {:?}", &err);
               // return format!("{}", err.to_string());
            }
        }
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

#[get("/identity/add/<seed>")]
pub fn add_identity(seed: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
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
                println!("{:?}", &response);
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(err) => {
                //println!("got error {:?}", &err);
                // return format!("{}", err.to_string());
            }
        }
    }
}

#[get("/identity/add/<seed>/<password>")]
pub fn add_identity_with_password(seed: &str, password: &str, mtx: &rocket::State<Mutex<Sender<HashMap<String, String>>>>, responses: &rocket::State<Mutex<Receiver<HashMap<String, String>>>>) -> String {
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
                println!("{:?}", &response);
                let id = response.get(&"message_id".to_string()).unwrap();
                if id == &request_id {
                    return format!("{}", response.get(&"status".to_string()).unwrap());
                } else {
                    continue;
                }
            },
            Err(err) => {
                // println!("got error {:?}", &err);
                // return format!("{}", err.to_string());
            }
        }
    }
}


#[get("/wallet/is_encrypted")]
pub fn is_wallet_encrypted() -> String {
    match store::sqlite::crud::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(pass) => {
            if pass.len() > 0 {
                format!("true")
            } else {
                format!("false")
            }
        },
        Err(err) => format!("{:?}", err)
    }
}

#[get("/wallet/set_master_password/<password>")]
pub fn set_master_password(password: &str) -> String {
    if password.len() < 4 {
        return format!("Password Too Short!");
    }
    match store::sqlite::crud::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(pass) => {
            if pass.len() > 0 {
                return format!("Wallet Password Already Set!");
            } else {
                match crypto::passwords::hash_password(password) {
                    Ok(hashed) => {
                        match store::sqlite::crud::master_password::set_master_password(store::get_db_path().as_str(), hashed.as_str()) {
                            Ok(_) => {
                                match store::sqlite::crud::fetch_all_identities_full(store::get_db_path().as_str()) {
                                    Ok(identities) => {
                                        for mut id in identities {
                                            if !(&id.encrypted) {
                                                match id.encrypt_identity(password) {
                                                    Ok(encrypted) => {
                                                        match store::sqlite::crud::update_identity_encrypted(store::get_db_path().as_str(), &encrypted) {
                                                            Ok(_) => println!("Updating Database, Identity.({}) Encrypted.", &encrypted.identity),
                                                            Err(err) => println!("Failed To Encrypt Identity.({}) : <{}>", &encrypted.identity, err)
                                                        }
                                                    },
                                                    Err(err) => {
                                                        return format!("{}", err);
                                                    }
                                                }
                                            }
                                        }
                                        return format!("ok");
                                    },
                                    Err(err) => {return format!("{}", err);}
                                }
                            },
                            Err(err) => {
                                return format!("{}", err);
                            }
                        }
                    },
                    Err(err) => {
                        return format!("{}", err);
                    }
                }
            }
        },
        Err(err) => {
            return format!("{:?}", err);
        }
    }
    format!("ok")
}

#[get("/wallet/encrypt/<password>")]
pub fn encrypt_wallet(password: &str) -> String {
    match store::sqlite::crud::master_password::get_master_password(store::get_db_path().as_str()) {
        Ok(pass) => {
            if pass.len() == 0 {
                return format!("You Must Set A Master Password First!");
            } else {
                match crypto::passwords::verify_password(password, pass[1].as_str()) {
                    Ok(verified) => {
                        if !verified {
                            return format!("Invalid Password!");
                        }
                        match store::sqlite::crud::fetch_all_identities_full(store::get_db_path().as_str()) {
                            Ok(identities) => {
                                for mut id in identities {
                                    if !(&id.encrypted) {
                                        match id.encrypt_identity(password) {
                                            Ok(encrypted) => {
                                                match store::sqlite::crud::update_identity_encrypted(store::get_db_path().as_str(), &encrypted) {
                                                    Ok(_) => println!("Updating Database, Identity.({}) Encrypted.", &encrypted.identity),
                                                    Err(err) => println!("Failed To Encrypt Identity.({}) : <{}>", &encrypted.identity, err)
                                                }
                                            },
                                            Err(err) => {
                                                return format!("{}", err);
                                            }
                                        }
                                    }
                                }
                                return format!("ok");
                            },
                            Err(err) => {return format!("{}", err);}
                        }
                    },
                    Err(err) => {
                        return format!("Invalid Password!");
                    }
                }
            }
        },
        Err(err) => {
            return format!("{:?}", err);
        }
    }
    format!("ok")
}

#[get("/wallet/download/<password>")]
pub fn download_wallet(password: &str) -> String {
    let mut ret_val: String = String::from("");
    match store::sqlite::crud::fetch_all_identities_full(store::get_db_path().as_str()) {
        Ok(mut identities) => {
            if password.len() < 4 {
                //invalid master password, don't decrypt wallet
                println!("Leaving Encrypted");
            }
            let mut isValid = false;

            for identity in &mut identities {
                let id: String = identity.identity.clone();
                ret_val += &id.clone();
                ret_val += ",";
                let encrypted: bool = identity.encrypted;
                //ret_val +=
                if password.len() < 4 {
                    isValid = true;
                    ret_val += &identity.seed.clone();
                    ret_val += ",";

                    ret_val += ",";
                    ret_val += "\n";
                } else {
                    if encrypted {
                        println!("Decrypting {}", &id);
                        match identity.decrypt_identity(password) {
                            Ok(decrypted) => {
                                isValid = true;
                                ret_val += &decrypted.seed.clone();
                                ret_val += ",";

                                ret_val += &decrypted.salt.clone();
                                ret_val += ",";

                                ret_val += &decrypted.hash.clone();
                                ret_val += "\n";
                            },
                            Err(_) => {}
                        }
                    } else {
                        isValid = true;
                        ret_val += &identity.seed.clone();
                        ret_val += ",";

                        ret_val += &identity.salt.clone();
                        ret_val += ",";

                        ret_val += &identity.hash.clone();
                        ret_val += "\n";
                    }
                }
            }
            if isValid {
                format!("{}", ret_val)
            } else {
                format!("Invalid Password!")
            }
        },
        Err(err) => format!("{}", err)
    }
}