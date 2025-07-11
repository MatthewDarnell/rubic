use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use sqlite::State;
use logger::{error};
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

pub fn create_peer(path: &str, id: &str, ip: &str, nick: &str, ping_time: u32, whitelisted: bool, last_responded: SystemTime) -> Result<(), String> {
    let prep_query = "INSERT INTO peer (id, ip, nick, whitelisted, ping, last_responded) \
    VALUES (:id, :ip, :nick, :whitelisted, :ping_time, :last_responded)\
     ON CONFLICT(ip) DO NOTHING;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    let whitelisted_string: String = match whitelisted {
                        true => "1".to_string(),
                        false  => "0".to_string()
                    };
                    let last_responded_unix_time: String = last_responded
                        .duration_since(UNIX_EPOCH)
                        .expect("Failed To Get Unix Time For Last Responded!")
                        .as_secs()
                        .to_string();
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id),
                        (":ip", ip),
                        (":nick", nick),
                        (":whitelisted", whitelisted_string.as_str()),
                        (":ping_time", ping_time.to_string().as_str()),
                        (":last_responded", last_responded_unix_time.as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in create_peer! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in create_peer! : {}", &err);
            Err(err)
        }
    }
}

pub fn blacklist(path: &str, id: &str) -> Result<(), String> {
    let prep_query = "UPDATE peer SET whitelisted = -1 WHERE id = :id";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in blacklist! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in blacklist! : {}", &err);
            Err(err)
        }
    }
}

pub fn remove_blacklist(path: &str, id: &str) -> Result<(), String> {
    let prep_query = "UPDATE peer SET whitelisted = 0 WHERE id = :id";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in remove_blacklist! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in remove_blacklist! : {}", &err);
            Err(err)
        }
    }
}


pub fn update_peer_last_responded(path: &str, id: &str, last_responded: SystemTime) -> Result<(), String> {
    let prep_query = "UPDATE peer SET last_responded=:last_responded WHERE id=:id;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    let last_responded_unix_time: String = last_responded
                        .duration_since(UNIX_EPOCH)
                        .expect("Failed To Get Unix Time For Last Responded!")
                        .as_secs()
                        .to_string();
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id),
                        (":last_responded", last_responded_unix_time.as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in update_peer_last_responded! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in update_peer_last_responded! : {}", &err);
            Err(err)
        }
    }
}

pub fn set_peer_connected(path: &str, id: &str) -> Result<(), String> {
    let prep_query = "UPDATE peer SET connected = true WHERE id=:id;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id)
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in set_peer_connected! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in set_peer_connected! : {}", &err);
            Err(err)
        }
    }
}
pub fn set_peer_disconnected(path: &str, id: &str) -> Result<(), String> {
    let prep_query = "UPDATE peer SET connected = false WHERE id=:id;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id)
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => Ok(()),
                                Err(error) => Err(error.to_string()),
                                _ => Err("Weird!".to_string())
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in set_peer_disconnected! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in set_peer_disconnected! : {}", &err);
            Err(err)
        }
    }
}

pub fn set_all_peers_disconnected(path: &str) -> Result<(), String> {
    let prep_query = "UPDATE peer SET connected = false;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match connection.execute(prep_query) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string())
            }
        },
        Err(err) => {
            error!("Error in set_all_peers_disconnected! : {}", &err);
            Err(err)
        }
    }
}


pub fn fetch_peer_by_ip(path: &str, ip: &str) -> Result<HashMap<String, String>, String> {
    let prep_query = "SELECT * FROM peer WHERE ip = :ip LIMIT 1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":ip", ip),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let mut result: HashMap<String, String> = HashMap::new();
                                    result.insert("ip".to_string(), statement.read::<String, _>("ip").unwrap());
                                    result.insert("id".to_string(), statement.read::<String, _>("id").unwrap());
                                    result.insert("nick".to_string(), statement.read::<String, _>("nick").unwrap());
                                    result.insert("whitelisted".to_string(), statement.read::<String, _>("whitelisted").unwrap());
                                    result.insert("ping".to_string(), statement.read::<i64, _>("ping").unwrap().to_string());
                                    result.insert("last_responded".to_string(), statement.read::<i64, _>("last_responded").unwrap().to_string());
                                    result.insert("connected".to_string(), statement.read::<String, _>("connected").unwrap().to_string());
                                    Ok(result)
                                },
                                Ok(State::Done) => {
                                    println!("Finished Reading. Failed To Fetch Peer By Ip.({}).", ip);
                                    Err("Peer Not Found!".to_string())
                                },
                                Err(err) => {
                                    Err(err.to_string())
                                }
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_peer_by_ip! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_peer_by_ip! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_peer_by_id(path: &str, id: &str) -> Result<HashMap<String, String>, String> {
    let prep_query = "SELECT * FROM peer WHERE id = :id LIMIT 1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":id", id),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let mut result: HashMap<String, String> = HashMap::new();
                                    result.insert("ip".to_string(), statement.read::<String, _>("ip").unwrap());
                                    result.insert("id".to_string(), statement.read::<String, _>("id").unwrap());
                                    result.insert("nick".to_string(), statement.read::<String, _>("nick").unwrap());
                                    result.insert("whitelisted".to_string(), statement.read::<String, _>("whitelisted").unwrap());
                                    result.insert("ping".to_string(), statement.read::<i64, _>("ping").unwrap().to_string());
                                    result.insert("last_responded".to_string(), statement.read::<i64, _>("last_responded").unwrap().to_string());
                                    result.insert("connected".to_string(), statement.read::<String, _>("connected").unwrap().to_string());
                                    Ok(result)
                                },
                                Ok(State::Done) => {
                                    println!("Finished Reading. Failed To Fetch Peer.");
                                    Err("Peer Not Found!".to_string())
                                },
                                Err(err) => {
                                    Err(err.to_string())
                                }
                            }
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_peer_by_id! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_peer_by_id! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_non_blacklisted_peers(path: &str) -> Result<Vec<Vec<String>>, String> {
    let prep_query = "SELECT * FROM peer WHERE whitelisted > -1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(_) => {
                    let mut ret_val: Vec<Vec<String>> = Vec::new();
                    connection
                        .iterate(prep_query, |peers| {
                            let mut each_peer: Vec<String> = Vec::new();
                            for &(_, value) in peers.iter() {
                                each_peer.push(value.unwrap().to_string());
                            }
                            ret_val.push(each_peer);
                            true
                        })
                        .unwrap();
                    Ok(ret_val)
                },
                Err(err) => {
                    error!("Error in fetch_non_blacklisted_peers! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_non_blacklisted_peers! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_all_peers(path: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM peer;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    let mut ret_val: Vec<HashMap<String, String>> = Vec::new();
                    loop {
                        match statement.next() {
                            Ok(State::Row) => {
                                let mut peer: HashMap<String, String> = HashMap::new();
                                let id: String = statement.read::<String, _>("id").unwrap();
                                let ip: String = statement.read::<String, _>("ip").unwrap();
                                let nick: String = statement.read::<String, _>("nick").unwrap();
                                let whitelisted: String = statement.read::<String, _>("whitelisted").unwrap();
                                let ping: String = statement.read::<String, _>("ping").unwrap();
                                let last_responded: String = statement.read::<String, _>("last_responded").unwrap();
                                let created: String = statement.read::<String, _>("created").unwrap();
                                let connected: String = statement.read::<String, _>("connected").unwrap();
                                peer.insert("id".to_string(), id);
                                peer.insert("ip".to_string(), ip);
                                peer.insert("nick".to_string(), nick);
                                peer.insert("whitelisted".to_string(), whitelisted);
                                peer.insert("ping".to_string(), ping);
                                peer.insert("last_responded".to_string(), last_responded);
                                peer.insert("created".to_string(), created);
                                peer.insert("connected".to_string(), connected);
                                ret_val.push(peer);                                
                            },
                            Ok(State::Done) => {
                                return Ok(ret_val);
                            },
                            Err(err) => {
                                println!("{}", err);
                                return Err(err.to_string());
                            },
                        }
                    }
                },
                Err(err) => {
                    error!("Error in fetch_all_peers! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_all_peers! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_connected_peers(path: &str) -> Result<Vec<Vec<String>>, String> {
    let prep_query = "SELECT * FROM peer WHERE connected = true;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(_) => {
                    let mut ret_val: Vec<Vec<String>> = Vec::new();
                    connection
                        .iterate(prep_query, |peers| {
                            let mut each_peer: Vec<String> = Vec::new();
                            for &(_, value) in peers.iter() {
                                each_peer.push(value.unwrap().to_string());
                            }
                            ret_val.push(each_peer);
                            true
                        })
                        .unwrap();
                    Ok(ret_val)
                },
                Err(err) => {
                    error!("Error in fetch_all_peers! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_all_peers! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_disconnected_peers(path: &str) -> Result<Vec<Vec<String>>, String> {
    let prep_query = "SELECT * FROM peer WHERE connected = false AND whitelisted > -1 ORDER BY last_responded DESC;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_PEER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(_) => {
                    let mut ret_val: Vec<Vec<String>> = Vec::new();
                    connection
                        .iterate(prep_query, |peers| {
                            let mut each_peer: Vec<String> = Vec::new();
                            for &(_, value) in peers.iter() {
                                each_peer.push(value.unwrap().to_string());
                            }
                            ret_val.push(each_peer);
                            true
                        })
                        .unwrap();
                    Ok(ret_val)
                },
                Err(err) => {
                    error!("Error in fetch_disconnected_peers! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_disconnected_peers! : {}", &err);
            Err(err)
        }
    }
}


pub mod test_peers {
    #![allow(dead_code, unused)]
    use serial_test::serial;
    use std::fs;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use crate::sqlite::peer::{create_peer, fetch_peer_by_id, fetch_peer_by_ip, fetch_all_peers, update_peer_last_responded, blacklist};
    #[test]
    #[serial]
    fn create_peer_and_insert() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_peer_and_insert_and_update_last_responded() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, UNIX_EPOCH).expect("Test Failed To Create Peer");
        update_peer_last_responded("test.sqlite", "id", SystemTime::now()).unwrap();
        match fetch_peer_by_ip("test.sqlite", "ip") {
            Ok(peer) => {
                assert_eq!(peer.keys().len(), 7);
                let time_secs: u64 = peer.get("last_responded").unwrap().parse().unwrap();
                let nineteen_seventy: SystemTime = SystemTime::UNIX_EPOCH;
                assert_eq!(nineteen_seventy.duration_since(UNIX_EPOCH).unwrap().as_secs(), 0);
                assert_ne!(Duration::from_secs(time_secs).as_secs(), nineteen_seventy.duration_since(UNIX_EPOCH).unwrap().as_secs());
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_peer_and_insert_and_fetch_by_ip() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        match fetch_peer_by_ip("test.sqlite", "ip") {
            Ok(peer) => {
                assert_eq!(peer.keys().len(), 7);
                assert_eq!(peer.get("nick").unwrap(), "nickname");
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_peer_and_insert_and_fetch_by_id() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        match fetch_peer_by_id("test.sqlite", "id") {
            Ok(peer) => {
                assert_eq!(peer.keys().len(), 7);
                assert_eq!(peer.get("nick").unwrap(), "nickname");
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_peer_and_insert_and_fetch() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        match fetch_peer_by_id("test.sqlite", "id") {
            Ok(peer) => {
                assert_eq!(peer.keys().len(), 7);
                assert_eq!(peer.get("nick").unwrap(), "nickname");
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }
    #[test]
    #[serial]
    fn create_peers_and_insert_and_fetch_all() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        create_peer("test.sqlite", "id2", "ip2", "nickname2", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        create_peer("test.sqlite", "id3", "ip3", "nickname3", 3000, true, SystemTime::now()).expect("Test Failed To Create Peer");
        match fetch_all_peers("test.sqlite") {
            Ok(peers) => {
                assert_eq!(peers.len(), 3);
                let peer2: &std::collections::HashMap<String, String> = &peers[1];
                assert_eq!(peer2.keys().len(), 8);
                assert_eq!(peer2.get(&"nick".to_string()).unwrap(), "nickname2");
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_peers_and_insert_and_delete_and_fetch_all() {
        create_peer("test.sqlite", "id", "ip", "nickname", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        create_peer("test.sqlite", "id2", "ip2", "nickname2", 3000, false, SystemTime::now()).expect("Test Failed To Create Peer");
        create_peer("test.sqlite", "id3", "ip3", "nickname3", 3000, true, SystemTime::now()).expect("Test Failed To Create Peer");
        blacklist("test.sqlite", "id").expect("Test Failed To Delete Peer");
        match fetch_all_peers("test.sqlite") {
            Ok(peers) => {
                assert_eq!(peers.len(), 3);
                let peer2: &std::collections::HashMap<String, String> = &peers[2];
                assert_eq!(peer2.keys().len(), 8);
                assert_eq!(peer2.get(&"nick".to_string()).unwrap(), "nickname3");
                
                
            },
            Err(err) => {
                println!("Peer Couldn't be Fetched!");
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }
    
    
}