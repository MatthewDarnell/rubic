use sqlite::State;
use logger::error;
use crate::sqlite::create::open_database;
use base64::Engine;
use base64::engine::general_purpose;
use crate::sqlite::get_db_lock;

pub fn insert_tick(path: &str, peer_id: &str, tick: u32) -> Result<(), String> {
    let prep_query = "INSERT INTO tick (tick, peer) VALUES(:tick, :peer)";
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match crate::sqlite::crud::prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":tick", tick.to_string().as_str()),
                        (":peer", peer_id)
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
                    error!("Error in insert_Tick! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in insert_Tick! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_latest_tick(path: &str) -> Result<String, String> {
    let prep_query = "SELECT tick FROM tick ORDER BY tick DESC LIMIT 1;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TICK_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match crate::sqlite::crud::prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.next() {
                        Ok(State::Row) => {
                            let result: String = statement.read::<String, _>("tick").unwrap();
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
                    error!("Error in fetch_latest_tick! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_latest_tick! : {}", &err);
            Err(err)
        }
    }
}
pub fn set_tick_validated(path: &str, tick: u32) -> Result<(), String> {
    let prep_query = "UPDATE tick SET valid = true WHERE tick = :tick;";
    //let _lock =SQLITE_TICK_MUTEX.lock().unwrap();
    let _lock = get_db_lock().lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match crate::sqlite::crud::prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":tick", tick.to_string().as_str()),
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
                    error!("Error in set_tick_validated! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in set_tick_validated! : {}", &err);
            Err(err)
        }
    }
}

pub fn set_tick_transaction_digests(path: &str, tick: u32, tx_digests: &[u8]) -> Result<(), String> {
    let prep_query = "UPDATE tick SET transaction_digests = :digests WHERE tick = :tick;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_TICK_MUTEX.lock().unwrap();
    let digests: String = general_purpose::STANDARD_NO_PAD.encode::<&[u8]>(tx_digests);
    match open_database(path, true) {
        Ok(connection) => {
            match crate::sqlite::crud::prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":digests", digests.as_str()),
                        (":tick", tick.to_string().as_str()),
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
                    error!("Error in set_tick_transaction_digests! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in set_tick_transaction_digests! : {}", &err);
            Err(err)
        }
    }
}