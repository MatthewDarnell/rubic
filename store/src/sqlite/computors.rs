use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose};
use std::sync::Mutex;
use lazy_static::lazy_static;
use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;

lazy_static! {
    static ref SQLITE_COMPUTORS_MUTEX: Mutex<i32> = Mutex::new(0i32); //Unlocks when goes out of scope due to fancy RAII
}


pub fn insert_computors_from_bytes(path: &str, peer: &str, bytes: &Vec<u8>) -> Result<(), String> {
    let _lock = get_db_lock().lock().unwrap();
    let epoch: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
    let mut pub_key_bytes: [u8; 676*32] = [0; 676*32];
    for (index, el) in bytes[2..2 + 676*32].chunks_exact(32).enumerate() {
        pub_key_bytes[index*32..index*32 + 32].copy_from_slice(el);
    }
    let mut signature: [u8; 64] = [0u8; 64];
    signature.copy_from_slice(&bytes[2 + 32*676..]);

    let mut keys: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(&pub_key_bytes);
    let mut sig: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(&signature);


    let prep_query = "INSERT INTO computors (peer, epoch, pub_keys, signature) VALUES(:peer, :epoch, :keys, :sig) ON CONFLICT(epoch) DO NOTHING;";
    //let _lock =SQLITE_COMPUTORS_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":peer", peer),
                        (":epoch", epoch.to_string().as_str()),
                        (":keys", keys.as_str()),
                        (":sig", sig.as_str())
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => {
                                    Ok(())
                                },
                                Err(error) => {
                                    Err(error.to_string())
                                },
                                _ => {
                                    Err("Weird!".to_string())
                                }
                            }
                        },
                        Err(err) => {
                            Err(err.to_string())
                        }
                    }
                },
                Err(err) => {
                    error!("Error in insert_computors! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in insert_computors! : {}", &err);
            Err(err)
        }
    }
}

pub fn insert_computors_from_parts(path: &str, epoch: u16, pub_keys: &Vec<Vec<u8>>, signature: &Vec<u8>) -> Result<(), String> {
    let _lock = get_db_lock().lock().unwrap();
    let mut pub_key_bytes: [u8; 676*32] = [0; 676*32];
    for (index, el) in pub_keys.iter().enumerate() {
        pub_key_bytes[index*32..index*32 + 32].copy_from_slice(el);
    }

    let mut keys: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(&pub_key_bytes);
    let mut sig: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(&signature);


    let prep_query = "INSERT INTO computors (epoch, pub_keys, signature) VALUES(:epoch, :keys, :sig) ON CONFLICT(epoch) DO NOTHING;";
    //let _lock =SQLITE_COMPUTORS_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":epoch", epoch.to_string().as_str()),
                        (":keys", keys.as_str()),
                        (":sig", sig.as_str())
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => {
                                    Ok(())
                                },
                                Err(error) => {
                                    Err(error.to_string())
                                },
                                _ => {
                                    Err("Weird!".to_string())
                                }
                            }
                        },
                        Err(err) => {
                            Err(err.to_string())
                        }
                    }
                },
                Err(err) => {
                    error!("Error in insert_computors! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in insert_computors! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_computors_by_epoch(path: &str, epoch: u16) -> Result<[u8; 2 + 676*32 + 64], String> {
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "SELECT epoch, pub_keys, signature FROM computors WHERE epoch = :epoch ORDER BY epoch DESC LIMIT 1;";
    //let _lock =SQLITE_COMPUTORS_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":epoch", epoch.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let mut result: [u8; 2 + 676*32 + 64] = [0u8; 2 + 676*32 + 64];
                                    let epoch_string: String = statement.read::<String, _>("epoch").unwrap();
                                    let pub_keys_string: String = statement.read::<String, _>("pub_keys").unwrap();
                                    let signature_string: String = statement.read::<String, _>("signature").unwrap();

                                    let epoch: u16 = u16::from_str(epoch_string.as_str()).unwrap();
                                    let pub_keys = general_purpose::STANDARD_NO_PAD.decode::<&String>(&pub_keys_string).unwrap();
                                    let signature = general_purpose::STANDARD_NO_PAD.decode::<&String>(&signature_string).unwrap();
                                    result[0..2].copy_from_slice(&epoch.to_le_bytes());
                                    result[2..2 + 676*32].copy_from_slice(&pub_keys.as_slice());
                                    result[2 + 676*32..].copy_from_slice(&signature.as_slice());
                                    Ok(result)
                                },
                                Ok(State::Done) => {
                                    //println!("Finished Reading. Failed To Fetch Latest Tick.");
                                    Err("No Tick Reported".to_string())
                                },
                                Err(err) => {
                                    Err(err.to_string())
                                }
                            }
                        },
                        Err(_) => {
                            Err("Failed".to_string())
                        }
                    }
                },
                Err(err) => Err(err)
            }
        },
        Err(err) => Err(err)
    }
}

pub fn fetch_latest_computors(path: &str) -> Result<[u8; 2 + 676*32 + 64], String> {
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "SELECT epoch, pub_keys, signature FROM computors ORDER BY epoch DESC LIMIT 1;";
    //let _lock =SQLITE_COMPUTORS_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.next() {
                        Ok(State::Row) => {
                            let mut result: [u8; 2 + 676*32 + 64] = [0u8; 2 + 676*32 + 64];
                            let epoch_string: String = statement.read::<String, _>("epoch").unwrap();
                            let pub_keys_string: String = statement.read::<String, _>("pub_keys").unwrap();
                            let signature_string: String = statement.read::<String, _>("signature").unwrap();

                            let epoch: u16 = u16::from_str(epoch_string.as_str()).unwrap();
                            let pub_keys = general_purpose::STANDARD_NO_PAD.decode::<&String>(&pub_keys_string).unwrap();
                            let signature = general_purpose::STANDARD_NO_PAD.decode::<&String>(&signature_string).unwrap();
                            result[0..2].copy_from_slice(&epoch.to_le_bytes());
                            result[2..676*32].copy_from_slice(&pub_keys.as_slice());
                            result[676*32..].copy_from_slice(&signature.as_slice());
                            Ok(result)
                        },
                        Ok(State::Done) => {
                            //println!("Finished Reading. Failed To Fetch Latest Tick.");
                            Err("No Tick Reported".to_string())
                        },
                        Err(err) => {
                            Err(err.to_string())
                        }
                    }
                },
                Err(err) => {
                    error!("Error in fetch_latest_computors! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_latest_computors! : {}", &err);
            Err(err)
        }
    }
}
