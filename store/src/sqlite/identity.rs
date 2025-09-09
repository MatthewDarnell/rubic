use std::collections::LinkedList;
use sqlite::State;
use protocol::identity::Identity;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;


pub fn insert_new_identity(path: &str, identity: &Identity) -> Result<(), String> {
    //TODO: get master password
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "INSERT INTO identities (seed, salt, hash, is_encrypted, identity) VALUES (:seed, :salt, :hash, :is_encrypted, :identity)";
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { hash, salt, identity, seed, encrypted } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":seed", seed.as_str()),
                                (":salt", salt.as_str()),
                                (":hash", hash.as_str()),
                                (":is_encrypted", encrypted.to_string().as_str()),
                                (":identity", identity.as_str())
                            ][..]) {
                                Ok(_) => {
                                    match statement.next() {
                                        Ok(State::Done) => {
                                            Ok(())
                                        },
                                        Err(error) => Err(error.to_string()),
                                        _ => Err("Weird!".to_string())
                                    }
                                },
                                Err(err) => Err(err.to_string())
                            }
                        }
                    }
                },
                Err(err) => {
                    error!("Error in insert_new_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in insert_new_identity! : {}", &err);
            Err(err)
        }
    }
}
pub fn update_identity_encrypted(path: &str, identity: &Identity) -> Result<(), String> {
    //TODO: get master password
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "UPDATE identities SET seed = :seed, salt = :salt, hash = :hash, is_encrypted = :is_encrypted WHERE identity = :identity";
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { hash, salt, identity, seed, encrypted } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":seed", seed.as_str()),
                                (":salt", salt.as_str()),
                                (":hash", hash.as_str()),
                                (":is_encrypted", encrypted.to_string().as_str()),
                                (":identity", identity.as_str())
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
                        }
                    }
                },
                Err(err) => {
                    error!("Error in update_identity_encrypted! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in update_identity_encrypted! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_all_identities(path: &str) -> Result<LinkedList<String>, String> {
    let prep_query = "SELECT identity FROM identities;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                    ][..]) {
                        Ok(_) => {
                            let mut ret_val: LinkedList<String> = LinkedList::new();
                            while let Ok(State::Row) = statement.next() {
                                ret_val.push_back(
                                    statement.read::<String, _>("identity").unwrap()
                                );
                            }
                            Ok(ret_val)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_all_identities! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_all_identities! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_all_identities_full(path: &str) -> Result<LinkedList<Identity>, String> {
    let prep_query = "SELECT * FROM identities;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                    ][..]) {
                        Ok(_) => {
                            let mut ret_val: LinkedList<Identity> = LinkedList::new();
                            while let Ok(State::Row) = statement.next() {
                                let temp_identity: String = statement.read::<String, _>("identity").unwrap();
                                let temp_seed: String = statement.read::<String, _>("seed").unwrap();
                                let temp_salt: String = statement.read::<String, _>("salt").unwrap();
                                let temp_hash: String = statement.read::<String, _>("hash").unwrap();
                                let temp_is_encrypted: String = statement.read::<String, _>("is_encrypted").unwrap();
                                ret_val.push_back(Identity::from_vars(
                                    temp_seed.as_str(),
                                    temp_hash.as_str(),
                                    temp_salt.as_str(),
                                    temp_identity.as_str(),
                                    temp_is_encrypted == "true"
                                ));
                            }
                            Ok(ret_val)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_all_identities_full! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_all_identities_full! : {}", &err);
            Err(err)
        }
    }
}
pub fn delete_all_response_entities_before_tick(path: &str, tick: u32) -> Result<(), String> {
    let prep_query = "DELETE FROM response_entity WHERE tick < :tick;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":tick", tick.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    println!("Read a Row While Trying To Delete Response Entities?");
                                    Ok(())
                                },
                                Ok(State::Done) => {
                                    Ok(())
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
                    error!("Error in delete_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in delete_identity! : {}", &err);
            Err(err)
        }
    }
}


pub fn fetch_balance_by_identity(path: &str, identity: &str) -> Result<Vec<String>, String> {
    //let prep_query = "SELECT * FROM (SELECT * FROM response_entity WHERE identity = :identity ORDER BY tick DESC) GROUP BY peer LIMIT 3;";
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "
    SELECT a.tick, b.identity, b.balance, c.ip as peer
        FROM (
            SELECT tick
                FROM response_entity
                WHERE identity = :identity
                GROUP by tick
                HAVING COUNT (DISTINCT peer) >= 2
                ORDER BY tick DESC
                LIMIT 1
        ) a
        INNER JOIN response_entity b
            ON a.tick = b.tick
        INNER JOIN peer c
                ON b.peer = c.id
        WHERE b.identity = :identity;
    ";
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    let mut response: Vec<String> = Vec::new();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            while let Ok(State::Row) = statement.next() {
                                response.push(
                                    statement.read::<String, _>("tick").unwrap()
                                );
                                response.push(
                                    statement.read::<String, _>("peer").unwrap()
                                );
                                response.push(
                                    statement.read::<String, _>("balance").unwrap()
                                );
                            }
                            Ok(response)
                        },
                        Err(err) => Err(err.to_string())
                    }
                },
                Err(err) => {
                    error!("Error in fetch_balance_by_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_balance_by_identity! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_identity(path: &str, identity: &str) -> Result<Identity, String> {
    let prep_query = "SELECT * FROM identities WHERE identity = :identity LIMIT 1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let id: Identity = Identity::from_vars(
                                        statement.read::<String, _>("seed").unwrap().as_str(),
                                        statement.read::<String, _>("hash").unwrap().as_str(),
                                        statement.read::<String, _>("salt").unwrap().as_str(),
                                        statement.read::<String, _>("identity").unwrap().as_str(),
                                        statement.read::<String, _>("is_encrypted").unwrap().as_str() == "true"
                                    );
                                    Ok(id)
                                },
                                Ok(State::Done) => {
                                    //println!("Finished Reading. Failed To Fetch Identity.");
                                    Err("Identity Not Found!".to_string())
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
                    error!("Error in fetch_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_identity! : {}", &err);
            Err(err)
        }
    }
}
pub fn delete_identity(path: &str, identity: &str) -> Result<(), String> {
    crate::sqlite::transfer::delete_transfers_by_source_identity(path, identity)?;
    let prep_query = "DELETE FROM identities WHERE identity = :identity;";
    //let prep_query = "DELETE FROM transfer WHERE source_identity = :identity; DELETE FROM identities WHERE identity = :identity;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, false) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    println!("Read a Row While Trying To Delete Identity?");
                                    Err("Identity Not Found".to_string())
                                },
                                Ok(State::Done) => {
                                    Ok(())
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
                    error!("Error in delete_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in delete_identity! : {}", &err);
            Err(err)
        }
    }
}


pub mod test_identities {
    use protocol::identity::Identity;
    use crate::sqlite::identity::{insert_new_identity, fetch_identity, delete_identity};
    use serial_test::serial;
    use std::fs;
    #[test]
    #[serial]
    fn create_identity_and_insert() {
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    //println!("Identity Inserted Ok!");
                },
                Err(err) => {
                    println!("{}", err);
                    assert_eq!(1, 2);
                }
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_identity_and_delete() {
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    match delete_identity("test.sqlite", &id.identity.as_str()) {
                        Ok(_) => {
                            match fetch_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                                Ok(identity) => {
                                    assert_eq!(1, 2);
                                },
                                Err(_err) => {
                                    //println!("Identity Deleted Ok!");
                                }
                            }
                        },
                        Err(err) => {
                            println!("Failed To Delete Identity! : {}", err.as_str());
                            assert_eq!(1, 2);
                        }
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    assert_eq!(1, 2);
                }
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }


    #[test]
    #[serial]
    fn create_identity_and_insert_and_fetch() {
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf");
            //println!("{:?}", &id);
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    //println!("Identity Inserted Ok!");
                    match fetch_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                        Ok(identity) => {
                            assert_eq!(identity.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                        },
                        Err(err) => {
                            println!("Failed To Fetch Identity! : {}", err.as_str());
                            assert_eq!(1, 2);
                        }
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    assert_eq!(1, 2);
                }
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn create_identity_encrypt_and_insert_and_fetch() {
        {
            let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf",);
            id =  id.encrypt_identity("password").unwrap();
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    //println!("Identity Inserted Ok!");
                    match fetch_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                        Ok(identity) => {
                            assert_eq!(identity.identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                        },
                        Err(err) => {
                            println!("Failed To Fetch Identity! : {}", err.as_str());
                            assert_eq!(1, 2);
                        }
                    }
                },
                Err(err) => {
                    println!("{}", err);
                    assert_eq!(1, 2);
                }
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }
}