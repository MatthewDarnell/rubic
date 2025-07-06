use std::collections::HashMap;
use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

pub fn create_transfer(path: &str, source: &str, destination: &str, amount: u64, tick: u32, signature: &str, txid: &str) -> Result<(), String> {
    let prep_query = "INSERT INTO transfer (source_identity, destination_identity, amount, tick, signature, txid) VALUES (
    :source, :destination, :amount, :tick, :signature, :txid
    );";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":source", source),
                        (":destination", destination),
                        (":amount", amount.to_string().as_str()),
                        (":tick", tick.to_string().as_str()),
                        (":signature", signature.to_string().as_str()),
                        (":txid", txid),
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

pub fn fetch_all_transfers(path: &str, asc: &String, limit: i32, offset: u32) -> Result<Vec<HashMap<String, String>>, String> {
    let _prep_query = format!("SELECT * FROM transfer WHERE txid NOT IN (SELECT txid FROM asset_transfer) ORDER BY tick {} LIMIT {} OFFSET {};", asc, limit, offset);
    let prep_query = _prep_query.as_str();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut transfer: HashMap<String, String> = HashMap::new();
                                transfer.insert("source".to_string(), statement.read::<String, _>("source_identity").unwrap());
                                transfer.insert("destination".to_string(), statement.read::<String, _>("destination_identity").unwrap());
                                transfer.insert("amount".to_string(), statement.read::<String, _>("amount").unwrap());
                                transfer.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                transfer.insert("signature".to_string(), statement.read::<String, _>("signature").unwrap());
                                transfer.insert("txid".to_string(), statement.read::<String, _>("txid").unwrap());
                                transfer.insert("broadcast".to_string(), statement.read::<String, _>("broadcast").unwrap());
                                transfer.insert("status".to_string(), statement.read::<String, _>("status").unwrap().to_string());
                                transfer.insert("created".to_string(), statement.read::<String, _>("created").unwrap());
                                response.push(transfer);
                            }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_all_transfers! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_all_transfers! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_transfer_by_txid(path: &str, txid: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM transfer WHERE txid = :txid ORDER BY created DESC;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":txid", txid.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut transfer: HashMap<String, String> = HashMap::new();
                                transfer.insert("source".to_string(), statement.read::<String, _>("source_identity").unwrap());
                                transfer.insert("destination".to_string(), statement.read::<String, _>("destination_identity").unwrap());
                                transfer.insert("amount".to_string(), statement.read::<String, _>("amount").unwrap());
                                transfer.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                transfer.insert("signature".to_string(), statement.read::<String, _>("signature").unwrap());
                                transfer.insert("txid".to_string(), statement.read::<String, _>("txid").unwrap());
                                transfer.insert("broadcast".to_string(), statement.read::<String, _>("broadcast").unwrap());
                                transfer.insert("status".to_string(), statement.read::<String, _>("status").unwrap());
                                transfer.insert("created".to_string(), statement.read::<String, _>("created").unwrap());
                                response.push(transfer);
                            }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_transfer_by_txid! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_transfer_by_txid! : {}", &err);
            Err(err)
        }
    }
}

pub fn delete_transfers_by_source_identity(path: &str, source_identity: &str) -> Result<(), String> {
    let prep_query = "DELETE FROM transfer WHERE source_identity = :source_identity;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":source_identity", source_identity),
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


pub fn fetch_transfers_to_broadcast(path: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM transfer WHERE broadcast = false ORDER BY tick ASC;";
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut transfer: HashMap<String, String> = HashMap::new();
                                transfer.insert("source".to_string(), statement.read::<String, _>("source_identity").unwrap());
                                transfer.insert("destination".to_string(), statement.read::<String, _>("destination_identity").unwrap());
                                transfer.insert("amount".to_string(), statement.read::<String, _>("amount").unwrap());
                                transfer.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                transfer.insert("signature".to_string(), statement.read::<String, _>("signature").unwrap());
                                transfer.insert("txid".to_string(), statement.read::<String, _>("txid").unwrap());
                                transfer.insert("broadcast".to_string(), statement.read::<String, _>("broadcast").unwrap());
                                transfer.insert("status".to_string(), statement.read::<String, _>("status").unwrap().to_string());
                                transfer.insert("created".to_string(), statement.read::<String, _>("created").unwrap());
                                response.push(transfer);
                            }
                            drop(_lock);
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_transfer_by_txid! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_transfer_by_txid! : {}", &err);
            Err(err)
        }
    }
}
pub fn set_transfer_as_broadcast(path: &str, txid: &str) -> Result<(), String> {
    let prep_query = "UPDATE transfer SET broadcast = true WHERE txid = :txid;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":txid", txid),
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

pub fn fetch_expired_and_broadcasted_transfers_with_unknown_status(path: &str, latest_tick: u32) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM transfer WHERE broadcast = true AND tick <= :latest_tick AND status = -1 ORDER BY tick ASC;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":latest_tick", latest_tick.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut transfer: HashMap<String, String> = HashMap::new();
                                transfer.insert("source".to_string(), statement.read::<String, _>("source_identity").unwrap());
                                transfer.insert("destination".to_string(), statement.read::<String, _>("destination_identity").unwrap());
                                transfer.insert("amount".to_string(), statement.read::<String, _>("amount").unwrap());
                                transfer.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                transfer.insert("signature".to_string(), statement.read::<String, _>("signature").unwrap());
                                transfer.insert("txid".to_string(), statement.read::<String, _>("txid").unwrap());
                                transfer.insert("broadcast".to_string(), statement.read::<String, _>("broadcast").unwrap());
                                transfer.insert("status".to_string(), statement.read::<String, _>("status").unwrap());
                                transfer.insert("created".to_string(), statement.read::<String, _>("created").unwrap());
                                response.push(transfer);
                            }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_transfer_by_txid! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_transfer_by_txid! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_expired_and_broadcasted_transfers_with_unknown_status_and_specific_tick(path: &str, tick: u32) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM transfer WHERE broadcast = true AND tick = :tick AND status = -1 ORDER BY tick ASC;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":tick", tick.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<HashMap<String, String>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let mut transfer: HashMap<String, String> = HashMap::new();
                                transfer.insert("source".to_string(), statement.read::<String, _>("source_identity").unwrap());
                                transfer.insert("destination".to_string(), statement.read::<String, _>("destination_identity").unwrap());
                                transfer.insert("amount".to_string(), statement.read::<String, _>("amount").unwrap());
                                transfer.insert("tick".to_string(), statement.read::<String, _>("tick").unwrap());
                                transfer.insert("signature".to_string(), statement.read::<String, _>("signature").unwrap());
                                transfer.insert("txid".to_string(), statement.read::<String, _>("txid").unwrap());
                                transfer.insert("broadcast".to_string(), statement.read::<String, _>("broadcast").unwrap());
                                transfer.insert("status".to_string(), statement.read::<String, _>("status").unwrap());
                                transfer.insert("created".to_string(), statement.read::<String, _>("created").unwrap());
                                response.push(transfer);
                            }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_transfer_by_txid! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_transfer_by_txid! : {}", &err);
            Err(err)
        }
    }
}

pub fn set_broadcasted_transfer_as_success(path: &str, txid: &str) -> Result<(), String> {
    let prep_query = "UPDATE transfer SET status = 0 WHERE txid = :txid AND broadcast = true;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":txid", txid),
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

pub fn set_broadcasted_transfer_as_failure(path: &str, txid: &str) -> Result<(), String> {
    let prep_query = "UPDATE transfer SET status = 1 WHERE txid = :txid AND broadcast = true;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":txid", txid),
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


pub mod test_transfer {
    use crate::sqlite::identity::insert_new_identity;
    use crate::sqlite::transfer::{create_transfer, fetch_transfer_by_txid, set_transfer_as_broadcast};
    use serial_test::serial;
    use std::fs;
    use protocol::identity::Identity;

    #[test]
    #[serial]
    fn create_transfer_and_insert_and_fetch_and_set_broadcast() {
        let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
        insert_new_identity("test.sqlite", &id);
        match create_transfer("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON",
                              "PBPMLQVFUQKBSCZSJLRMNCYEJXSBQOEKECAXARVEIDKDQZPNVSGVSLZFDQMD",
                              100, 4000, "signature", "txid") {
            Ok(_) => {
                match fetch_transfer_by_txid("test.sqlite", "txid") {
                    Ok(response_vec) => {
                        assert_eq!(response_vec.len(), 1);
                        let mut tx = response_vec.first().unwrap();
                        assert_eq!(tx.get(&"broadcast".to_string()).unwrap(), &"0".to_string());
                        match set_transfer_as_broadcast("test.sqlite", "txid") {
                            Ok(_) => {
                                match fetch_transfer_by_txid("test.sqlite", "txid") {
                                    Ok(response_vec) => {
                                        assert_eq!(response_vec.len(), 1);
                                        let mut tx = response_vec.first().unwrap();
                                        assert_eq!(tx.get(&"broadcast".to_string()).unwrap(), &"1".to_string());
                                    },
                                    Err(err) => {
                                        println!("Failed To Fetch Transfer! : {}", err.as_str());
                                        assert_eq!(1, 2);
                                    }
                                }
                            },
                            Err(err) => {
                                println!("{}", err);
                                assert_eq!(1, 2);
                            }
                        }
                    },
                    Err(err) => {
                        assert_eq!(1, 2);
                    }
                }
            },
            Err(err) => {
                println!("{}", err);
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