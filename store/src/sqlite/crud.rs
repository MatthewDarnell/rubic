use std::collections::HashMap;
use identity::Identity;
use crate::sqlite::create::open_database;
use sqlite::State;

fn prepare_crud_statement<'a>(path: &'a str, connection: &'a sqlite::Connection, prep_query: &'a str) -> Result<sqlite::Statement<'a>, String> {
        match connection.prepare(prep_query) {
            Ok(stmt) => Ok(stmt),
            Err(err) => Err(err.to_string())
        }
}



pub fn insert_new_identity(path: &str, identity: &Identity) -> Result<(), String> {
    let prep_query = "INSERT INTO identities (identity_index, seed, seed_ct, salt, hash, identity) VALUES (:index, :seed, :seed_ct, :salt, :hash, :identity)";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { seed_ct, hash, salt, identity, index, seed } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":index", index.to_string().as_str()),
                                (":seed", seed.as_str()),
                                (":seed_ct", seed_ct.as_str()),
                                (":salt", salt.as_str()),
                                (":hash", hash.as_str()),
                                (":identity", identity.as_str()),
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
                    println!("Error in insert_new_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in insert_new_identity! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_identity(path: &str, identity: &str) -> Result<Identity, String> {
    let prep_query = "SELECT * FROM identities WHERE identity = :identity LIMIT 1;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let index: u32 = statement.read::<String, _>("identity_index").unwrap().parse().unwrap();
                                    let id: Identity = Identity::from_vars(
                                        statement.read::<String, _>("seed").unwrap().as_str(),
                                        statement.read::<String, _>("seed_ct").unwrap().as_str(),
                                        statement.read::<String, _>("hash").unwrap().as_str(),
                                        statement.read::<String, _>("salt").unwrap().as_str(),
                                        statement.read::<String, _>("identity").unwrap().as_str(),
                                        index
                                    );
                                    Ok(id)
                                },
                                Ok(State::Done) => {
                                  println!("Finished Reading. Failed To Fetch Identity.");
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
                    println!("Error in insert_new_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in insert_new_identity! : {}", &err);
            Err(err)
        }
    }
}

pub fn delete_identity(path: &str, identity: &str) -> Result<(), String> {
    let prep_query = "DELETE FROM identities WHERE identity = :identity;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
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
                    println!("Error in insert_new_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in insert_new_identity! : {}", &err);
            Err(err)
        }
    }
}




#[cfg(test)]
mod store_crud_tests {
    use identity::Identity;
    use crate::sqlite::crud::{insert_new_identity, fetch_identity, delete_identity};
    use serial_test::serial;

    #[test]
    #[serial]
    fn create_identity_and_insert() {
        use std::fs;
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", 0);
            println!("{:?}", &id);
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    println!("Identity Inserted Ok!");
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
        use std::fs;
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", 0);
            println!("{:?}", &id);
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    match delete_identity("test.sqlite", &id.identity.as_str()) {
                        Ok(_) => {
                            match fetch_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                                Ok(identity) => {
                                    assert_eq!(1, 2);
                                },
                                Err(err) => {
                                    println!("Identity Deleted Ok!");
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
        use std::fs;
        {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", 0);
            println!("{:?}", &id);
            match insert_new_identity("test.sqlite", &id) {
                Ok(_) => {
                    println!("Identity Inserted Ok!");
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
