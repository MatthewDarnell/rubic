use lazy_static::lazy_static;

use base64::{engine::general_purpose, Engine as _};
use crate::sqlite::create::open_database;
use sqlite::State;
use logger::{error};
use std::sync::Mutex;

lazy_static! {
    static ref SQLITE_MUTEX: Mutex<i32> = Mutex::new(0i32); //Unlocks when goes out of scope due to fancy RAII
}

pub(crate) fn prepare_crud_statement<'a>(connection: &'a sqlite::Connection, prep_query: &'a str) -> Result<sqlite::Statement<'a>, String> {
        match connection.prepare(prep_query) {
            Ok(stmt) => Ok(stmt),
            Err(err) => Err(err.to_string())
        }
}


pub fn create_peer_response(path: &str, peer: &str, data: &Vec<u8>) -> Result<(), String> {
    let prep_query = "INSERT INTO response (peer, header, type, data) VALUES (:peer, :header, :response_type, :data);";
    //let _lock =SQLITE_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    let header = &data[0..8];
                    let real_data = &data[8..];
                    let response_type = &data[7];
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":peer", peer),
                        (":header", general_purpose::STANDARD.encode(&header).as_str()),
                        (":response_type", response_type.to_string().as_str()),
                        (":data", general_purpose::STANDARD.encode(&real_data).as_str()),
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
                    error!("Error in create_peer_response! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in create_peer_response! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_peer_response_by_type(path: &str, response_type: u8) -> Result<Vec<Vec<u8>>, String> {
    let prep_query = "SELECT * FROM response WHERE type = :response_type ORDER BY created DESC;";
    //let _lock =SQLITE_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":response_type", response_type.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<Vec<u8>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                //let peer_ip = statement.read::<String, _>("peer").unwrap();
                                let mut header_bytes: Vec<u8> = general_purpose::STANDARD.decode(statement.read::<String, _>("header").unwrap()).unwrap();
                                let mut data_bytes: Vec<u8> = general_purpose::STANDARD.decode(statement.read::<String, _>("data").unwrap()).unwrap();
                                header_bytes.append(&mut data_bytes);
                                response.push(header_bytes);
                                }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_peer_response_by_type! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_peer_response_by_type! : {}", &err);
            Err(err)
        }
    }
}

#[cfg(test)]
mod store_crud_tests {

    pub mod response {
        use crate::sqlite::response_entity::{create_response_entity, fetch_latest_response_entity_by_identity_group_peers, fetch_response_entity_by_identity};
        use serial_test::serial;
        use std::fs;

        #[test]
        #[serial]
        fn create_response_entity_and_insert_and_fetch() {
            create_response_entity("test.sqlite", "127.0.0.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 100000, 1).unwrap();
            create_response_entity("test.sqlite", "127.0.0.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 100000, 1).unwrap();
                match fetch_response_entity_by_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                    Ok(response_vec) => {
                    assert_eq!(response_vec.len(), 2);
                        assert_eq!(response_vec[0].get("identity").unwrap().as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                    },
                    Err(err) => {
                        println!("ResponseEntity Couldn't be Fetched!");
                        assert_eq!(1, 2);
                    }
                }
            fs::remove_file("test.sqlite").unwrap();
        }

        #[test]
        #[serial]
        fn create_response_entities_and_insert_and_fetch_grouping_by_peer_and_sorting_latest_time() {
            create_response_entity("test.sqlite", "127.0.0.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 9000, 1).unwrap();
            create_response_entity("test.sqlite", "0.0.0.0", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 8500, 1).unwrap();
            create_response_entity("test.sqlite", "127.0.0.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 9001, 1).unwrap();
            create_response_entity("test.sqlite", "0.0.0.0", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 9000, 1).unwrap();
            create_response_entity("test.sqlite", "10.1.1.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 70, 1).unwrap();
            create_response_entity("test.sqlite", "127.0.0.1", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", 1000, 0, 1, 0, 100, 100, 1000, 8999, 1).unwrap();
            match fetch_latest_response_entity_by_identity_group_peers("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                Ok(response_vec) => {
                    assert_eq!(response_vec.len(), 3);  //num peers = 3
                    assert_eq!(response_vec[0].get("identity").unwrap().as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                    for peer in &response_vec {
                        match peer.get("peer_ip").unwrap().as_str() {
                            "127.0.0.1" => {
                                assert_eq!(peer.get("incoming").unwrap(), "1000");
                                assert_eq!(peer.get("tick").unwrap(), "9001");
                            },
                            "0.0.0.0" => {
                                assert_eq!(peer.get("incoming").unwrap(), "1000");
                                assert_eq!(peer.get("tick").unwrap(), "9000");
                            },
                            "10.1.1.1" => {
                                assert_eq!(peer.get("incoming").unwrap(), "1000");
                                assert_eq!(peer.get("tick").unwrap(), "70");

                            },
                            _ => { assert_eq!(1, 2) }
                        }
                    }
                },
                Err(err) => {
                    println!("ResponseEntity Couldn't be Fetched!");
                    assert_eq!(1, 2);
                }
            }
            fs::remove_file("test.sqlite").unwrap();
        }
        #[test]
        #[serial]
        fn delete_db() {
            fs::remove_file("test.sqlite"); //Don't care about the result
        }
    }
}
