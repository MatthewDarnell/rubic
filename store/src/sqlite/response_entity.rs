use std::collections::HashMap;
use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

pub fn create_response_entity(path: &str, peer: &str, identity: &str, incoming: u64, outgoing: u64, balance: u64, num_in_txs: u32, num_out_txs: u32, latest_in_tick: u32, latest_out_tick: u32, tick: u32, spectrum_index: i32) -> Result<(), String> {
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "INSERT INTO response_entity (peer, identity, incoming, outgoing, balance, num_in_txs, num_out_txs, latest_in_tick, latest_out_tick, tick, spectrum_index) VALUES (
    :peer, :identity, :incoming, :outgoing, :balance, :num_in_txs, :num_out_txs, :latest_in_tick, :latest_out_tick, :tick, :spectrum_index
    );";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":peer", peer),
                        (":identity", identity),
                        (":incoming", incoming.to_string().as_str()),
                        (":outgoing", outgoing.to_string().as_str()),
                        (":balance", balance.to_string().as_str()),
                        (":num_in_txs", num_in_txs.to_string().as_str()),
                        (":num_out_txs", num_out_txs.to_string().as_str()),
                        (":latest_in_tick", latest_in_tick.to_string().as_str()),
                        (":latest_out_tick", latest_out_tick.to_string().as_str()),
                        (":tick", tick.to_string().as_str()),
                        (":spectrum_index", spectrum_index.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => { Ok(()) },
                                Err(error) => { Err(error.to_string()) },
                                _ => { Err("Weird!".to_string()) }
                            }
                        },
                        Err(err) => { Err(err.to_string()) }
                    }
                },
                Err(err) => {
                    error(format!("Failed To Prepare Statement! {}", err.to_string()).as_str());
                    Err(err.to_string())
                }
            }
        },
        Err(err) => {
            error(format!("Failed To Open Database! {}", err.to_string()).as_str());
            Err(err.to_string())
        }
    }
}
pub fn fetch_response_entity_by_identity(path: &str, identity: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM response_entity WHERE identity = :identity ORDER BY created DESC;";
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut response_entity: HashMap<String, String> = HashMap::new();
                                response_entity.insert("peer_ip".to_string(), statement.read::<String, _>("peer").unwrap());
                                response_entity.insert("identity".to_string(), statement.read::<String, _>("identity").unwrap());
                                response_entity.insert("incoming".to_string(), statement.read::<String, _>("incoming").unwrap());
                                response_entity.insert("outgoing".to_string(), statement.read::<String, _>("outgoing").unwrap());
                                response_entity.insert("num_in_txs".to_string(), statement.read::<String, _>("num_in_txs").unwrap());
                                response_entity.insert("num_out_txs".to_string(), statement.read::<String, _>("num_out_txs").unwrap());
                                response_entity.insert("latest_in_tick".to_string(), statement.read::<String, _>("latest_in_tick").unwrap());
                                response_entity.insert("latest_out_tick".to_string(), statement.read::<String, _>("latest_out_tick").unwrap());
                                response_entity.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                response_entity.insert("spectrum_index".to_string(), statement.read::<String, _>("spectrum_index").unwrap());
                                response.push(response_entity);
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
pub fn fetch_latest_response_entity_by_identity_group_peers(path: &str, identity: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM (SELECT * FROM response_entity WHERE identity = :identity ORDER BY tick DESC) GROUP BY peer;";
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut response_entity: HashMap<String, String> = HashMap::new();
                                response_entity.insert("peer_ip".to_string(), statement.read::<String, _>("peer").unwrap());
                                response_entity.insert("identity".to_string(), statement.read::<String, _>("identity").unwrap());
                                response_entity.insert("incoming".to_string(), statement.read::<String, _>("incoming").unwrap());
                                response_entity.insert("outgoing".to_string(), statement.read::<String, _>("outgoing").unwrap());
                                response_entity.insert("num_in_txs".to_string(), statement.read::<String, _>("num_in_txs").unwrap());
                                response_entity.insert("num_out_txs".to_string(), statement.read::<String, _>("num_out_txs").unwrap());
                                response_entity.insert("latest_in_tick".to_string(), statement.read::<String, _>("latest_in_tick").unwrap());
                                response_entity.insert("latest_out_tick".to_string(), statement.read::<String, _>("latest_out_tick").unwrap());
                                response_entity.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                response_entity.insert("spectrum_index".to_string(), statement.read::<String, _>("spectrum_index").unwrap());
                                response.push(response_entity);
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


pub mod test_response_entity {
    use crate::sqlite::crud::{create_peer_response, fetch_peer_response_by_type};
    use serial_test::serial;
    use std::fs;

    #[test]
    #[serial]
    fn create_response_and_insert_and_fetch() {
        let data: Vec<u8> = vec![80, 3, 0, 167, 8, 105, 98, 32, 80, 3, 0, 167, 8, 105, 98, 32, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 225, 245, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 250, 100, 124, 0, 0, 0, 0, 0, 124, 141, 129, 0, 1, 0, 0, 0, 42, 90, 241, 198, 106, 243, 239, 74, 41, 78, 9, 242, 122, 237, 3, 13, 63, 174, 255, 185, 161, 145, 0, 18, 70, 139, 156, 127, 62, 70, 189, 159, 18, 49, 252, 149, 121, 228, 161, 156, 5, 104, 234, 45, 220, 245, 230, 19, 51, 151, 17, 31, 147, 163, 145, 11, 96, 54, 223, 194, 153, 88, 7, 153, 236, 214, 251, 45, 205, 47, 16, 11, 86, 100, 214, 84, 204, 245, 113, 6, 108, 13, 172, 151, 88, 42, 241, 66, 109, 41, 52, 62, 12, 163, 125, 174, 57, 33, 123, 231, 45, 173, 64, 110, 153, 145, 12, 112, 192, 130, 163, 44, 89, 9, 43, 129, 141, 112, 192, 170, 171, 155, 11, 204, 121, 169, 79, 92, 65, 156, 144, 198, 90, 88, 74, 154, 40, 181, 191, 15, 219, 29, 67, 231, 230, 43, 230, 5, 19, 23, 124, 204, 180, 165, 144, 161, 73, 135, 50, 77, 141, 111, 78, 247, 107, 163, 45, 75, 151, 228, 192, 122, 239, 28, 53, 101];;
        create_peer_response("test.sqlite", "127.0.0.1", &data).unwrap();
        create_peer_response("test.sqlite", "127.0.0.1", &data).unwrap();
        match fetch_peer_response_by_type("test.sqlite", 32) {
            Ok(response_vec) => {
                assert_eq!(response_vec.len(), 2);
                //assert_eq!(response_vec[0].peer.as_ref().unwrap().as_str(), "127.0.0.1");
            },
            Err(err) => {
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