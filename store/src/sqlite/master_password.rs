use sqlite::State;
use logger::{error};
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

pub fn set_master_password(path: &str, ct: &str) -> Result<(), String> {
    let prep_query = "INSERT INTO master_password (ct) \
    VALUES (:ct);";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_MASTER_PASSWORD_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":ct", ct)
                    ][..]) {
                        Ok(_) => {
                            //println!("Master Password Set!");
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
                    error!("Error in set_master_password! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in set_master_password! : {}", &err);
            Err(err)
        }
    }
}
pub fn get_master_password(path: &str) -> Result<Vec<String>, String> {
    let prep_query = "SELECT * FROM master_password LIMIT 1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_MASTER_PASSWORD_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(_) => {
                    let mut ret_val: Vec<String> = Vec::new();
                    match connection
                        .iterate(prep_query, |master_pass| {
                            for &(_, value) in master_pass.iter() {
                                ret_val.push(value.unwrap().to_string());
                            }
                            true
                        }) {
                        Ok(_) => {
                            if ret_val.len() < 1 {
                                Err("No Master Password Set".to_string())
                            } else {
                                Ok(ret_val)
                            }
                        },
                        Err(err) => {
                            error!("Error in get_master_password! : {}", &err);
                            Err(err.to_string())
                        }
                    }
                },
                Err(err) => {
                    error!("Error in get_master_password! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in get_master_password! : {}", &err);
            Err(err)
        }
    }
}


pub mod tests_master_password {
    use serial_test::serial;
    use std::fs;
    use crate::sqlite::master_password::{get_master_password, set_master_password};

    #[test]
    #[serial]
    fn set_a_master_password_and_fetch_it() {
        match set_master_password("test.sqlite", "ciphertext") {
            Ok(_) => {
                match get_master_password("test.sqlite") {
                    Ok(result) => {
                        assert_eq!(result.get(0).unwrap(), &"1".to_string());
                        assert_eq!(result.get(1).unwrap(), &"ciphertext".to_string());
                    },
                    Err(err) => {
                        println!("{}", err);
                        assert_eq!(1, 2);
                    }
                }
            },
            Err(err) => {
                println!("Failed To Insert Master Password: {}", err);
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }

    #[test]
    #[serial]
    fn enforce_max_one_master_password_row() {
        match set_master_password("test.sqlite", "ciphertext") {
            Ok(_) => {
                match set_master_password("test.sqlite", "ciphertext1") {
                    Ok(_) => {
                        assert_eq!(1, 2);
                    },
                    Err(_) => {
                        assert_eq!(1, 1);
                    }
                }
            },
            Err(err) => {
                println!("Failed To Insert Master Password: {}", err);
                assert_eq!(1, 2);
            }
        }
        fs::remove_file("test.sqlite").unwrap();
    }
}