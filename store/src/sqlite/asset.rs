use std::collections::HashMap;
use std::str::FromStr;
use sqlite::State;
use identity::Identity;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;


/*
    CREATE TABLE IF NOT EXISTS asset (
        identity TEXT NOT NULL,
        asset TEXT NOT NULL,
        issuance TEXT,
        ownership TEXT,
        tick INTEGER NOT NULL,
        universe_index INTEGER NOT NULL,
        siblings TEXT NOT NULL,
        created DATETIME DEFAULT CURRENT_TIMESTAMP,
        peer TEXT,
        FOREIGN KEY(source_identity) REFERENCES identities(identity) ON DELETE CASCADE
    );
    */
pub fn create_asset(path: &str, peer: &str, identity: &str, tick: u32, universe_index: u32, siblings: &str) -> Result<u64, String> {
    let _lock = get_db_lock().lock().unwrap();
    let prep_query = "INSERT INTO asset (peer, identity, tick, universe_index, siblings) VALUES (
    :peer, :identity, :tick, :universe_index, :siblings);";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":peer", peer),
                        (":identity", identity),
                        (":tick", tick.to_string().as_str()),
                        (":universe_index", universe_index.to_string().as_str()),
                        (":siblings", siblings)
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Done) => {
                                    match prepare_crud_statement(&connection, "SELECT last_insert_rowid() as last_id") {
                                        Ok(mut statement) => {
                                            match statement.next() {
                                                Ok(State::Row) => {
                                                    let _id = statement.read::<String, _>("last_id").unwrap();
                                                    match u64::from_str(_id.as_str()) {
                                                        Ok(r) => Ok(r),
                                                        Err(_) => {
                                                            Err("Failed To Retrieve Inserted Asset Id!".to_string())
                                                        }
                                                    }
                                                },
                                                _ => {
                                                    Err("Failed To Parse Inserted Asset Id!".to_string())
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            Err(e)
                                        }
                                    }
                                },
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

pub fn fetch_issued_assets(path: &str) -> Result<Vec<String>, String> {
    let prep_query = "SELECT DISTINCT(name) FROM asset_record WHERE name IS NOT NULL;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    let mut ret_val: Vec<String> = Vec::new();
                    loop {
                        match statement.next() {
                            Ok(State::Row) => {
                                let name: String = statement.read::<String, _>("name").unwrap();
                                ret_val.push(name);
                            },
                            Ok(State::Done) => {
                                //println!("Finished Reading. Failed To Fetch Identity.");
                                return Ok(ret_val);
                            },
                            Err(err) => {
                                return Err(err.to_string());
                            }
                        }   
                    }
                },
                Err(err) => {
                    error!("Error in fetch_issued_assets! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_issued_assets! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_asset_balance(path: &str, asset_name: &str, identity: &str) -> Result<HashMap<String, String>, String> {
    let prep_query = "SELECT asset.tick, a.peer, issuance.name,
                               issuance.pub_key as issuer,
                               issuance.num_decimal,
                               possession.num_shares as balance
                                   FROM (
                                          SELECT tick
                                          FROM asset
                                          WHERE identity = :identity
                                          GROUP by tick
                                          HAVING COUNT (DISTINCT peer) >= 1
                                          ORDER BY tick DESC
                                          limit 1
                                   ) asset
                               left outer join asset a
                                        on asset.tick = a.tick
                               inner join asset_record issuance
                                         ON issuance.asset_id = a.id
                               inner join asset_record possession
                                         ON possession.asset_id = a.id
                               where a.identity=:identity
                               and issuance.name = :asset_name
                               and issuance.record_type = 'I'
                               AND possession.record_type = 'P'
                               GROUP BY issuance.name, a.peer
                               HAVING COUNT (DISTINCT balance) <= 1
                               order by asset.tick desc
                               LIMIT 1
                        ";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":asset_name", asset_name),
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            let mut ret_val: HashMap<String, String> = HashMap::new();
                            match statement.next() {
                                Ok(State::Row) => {
                                    let tick: String = statement.read::<String, _>("tick").unwrap();
                                    let name: String = statement.read::<String, _>("name").unwrap();
                                    let peer: String = statement.read::<String, _>("peer").unwrap();
                                    let issuer: String = statement.read::<String, _>("issuer").unwrap();
                                    let num_decimal: String = statement.read::<String, _>("num_decimal").unwrap();
                                    let balance: String = statement.read::<String, _>("balance").unwrap();
                                    ret_val.insert("tick".to_string(), tick);
                                    ret_val.insert("name".to_string(), name);
                                    ret_val.insert("peer".to_string(), peer);
                                    ret_val.insert("issuer".to_string(), issuer);
                                    ret_val.insert("num_decimal".to_string(), num_decimal);
                                    ret_val.insert("balance".to_string(), balance);
                                    Ok(ret_val)
                                },
                                Ok(State::Done) => {
                                    //println!("Finished Reading. Failed To Fetch Identity.");
                                    Ok(ret_val) //none
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
                    error!("Error in fetch_issued_assets! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in fetch_issued_assets! : {}", &err);
            Err(err)
        }
    }
}

pub fn delete_all_assets_before_tick(path: &str, tick: u32) -> Result<(), String> {
    let prep_query = "DELETE FROM asset WHERE tick < :tick;";
    let _lock = get_db_lock().lock().unwrap();
    //let _lock =SQLITE_IDENTITY_MUTEX.lock().unwrap();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(&connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":tick", tick.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    println!("Read a Row While Trying To Delete Assets?");
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
                    error!("Error in delete_all_assets_before_tick! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            error!("Error in delete_all_assets_before_tick! : {}", &err);
            Err(err)
        }
    }
}


pub mod asset_record {
    use sqlite::State;
    use logger::error;
    use crate::sqlite::create::open_database;
    use crate::sqlite::crud::prepare_crud_statement;
    use crate::sqlite::get_db_lock;
    pub fn create_asset_issuance(path: &str, asset_id: u64, pub_key: &str, _type: u8, name: &str, num_decimal_places: u8, unit_of_measurement: &str) -> Result<(), String> {
        let _lock = get_db_lock().lock().unwrap();
        let prep_query = "INSERT INTO asset_record (asset_id, record_type, pub_key, type, name, num_decimal, unit_measure) VALUES (
    :asset_id, 'I', :pub_key, :type, :name, :num_decimal, :unit_measure);";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":asset_id", asset_id.to_string().as_str()),
                            (":pub_key", pub_key),
                            (":type", _type.to_string().as_str()),
                            (":name", name),
                            (":num_decimal", num_decimal_places.to_string().as_str()),
                            (":unit_measure", unit_of_measurement),
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
    
    pub fn create_asset_ownership(path: &str, asset_id: u64, pub_key: &str, _type: u8, padding: u8, managing_contract_index: u16, issuance_index: u32, num_shares: u64) -> Result<(), String> {
        let _lock = get_db_lock().lock().unwrap();
        let prep_query = "INSERT INTO asset_record (asset_id, record_type, pub_key, type, padding, managing_contract, issuance_index, num_shares) VALUES (
    :asset_id, 'O', :pub_key, :type, :padding, :managing_contract, :issuance_index, :num_shares);";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":asset_id", asset_id.to_string().as_str()),
                            (":pub_key", pub_key),
                            (":type", _type.to_string().as_str()),
                            (":padding", padding.to_string().as_str()),
                            (":managing_contract", managing_contract_index.to_string().as_str()),
                            (":issuance_index", issuance_index.to_string().as_str()),
                            (":num_shares", num_shares.to_string().as_str()),
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

    pub fn create_asset_possession(path: &str, asset_id: u64, pub_key: &str, _type: u8, padding: u8, managing_contract_index: u16, issuance_index: u32, num_shares: u64) -> Result<(), String> {
        let _lock = get_db_lock().lock().unwrap();
        let prep_query = "INSERT INTO asset_record (asset_id, record_type, pub_key, type, padding, managing_contract, issuance_index, num_shares) VALUES (
    :asset_id, 'P', :pub_key, :type, :padding, :managing_contract, :issuance_index, :num_shares);";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":asset_id", asset_id.to_string().as_str()),
                            (":pub_key", pub_key),
                            (":type", _type.to_string().as_str()),
                            (":padding", padding.to_string().as_str()),
                            (":managing_contract", managing_contract_index.to_string().as_str()),
                            (":issuance_index", issuance_index.to_string().as_str()),
                            (":num_shares", num_shares.to_string().as_str()),
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

}

pub mod asset_transfer {
    use std::collections::HashMap;
    use sqlite::State;
    use logger::error;
    use crate::sqlite::create::open_database;
    use crate::sqlite::crud::prepare_crud_statement;
    use crate::sqlite::get_db_lock;
    pub fn create_asset_transfer(path: &str, issuer: &str, new_owner_and_possessor: &str, amount: i64, name: &str, input_size: u16, input_type: u16, txid: &str) -> Result<(), String> {
        let prep_query = "INSERT INTO asset_transfer (name, issuer, new_owner_and_possessor, num_shares, input_size, input_type, txid) VALUES (
    :name, :issuer, :new_owner_and_possessor, :num_shares, :size, :type, :txid
    );";
        let _lock = get_db_lock().lock().unwrap();
        //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":name", name),
                            (":issuer", issuer),
                            (":new_owner_and_possessor", new_owner_and_possessor),
                            (":num_shares", amount.to_string().as_str()),
                            (":size", input_size.to_string().as_str()),
                            (":type", input_type.to_string().as_str()),
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

    pub fn fetch_transfer_by_txid(path: &str, txid: &str) -> Result<Option<HashMap<String, String>>, String> {
        let _prep_query = "SELECT * FROM asset_transfer WHERE txid = :txid".to_string();
        let prep_query = _prep_query.as_str();
        //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
        let _lock = get_db_lock().lock().unwrap();
        match open_database(path, false) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":txid", txid),
                        ][..]) {
                            Ok(_) => {
                                if let Ok(State::Row) = statement.next() {
                                    let mut transfer: HashMap<String, String> = HashMap::new();
                                    //Asset
                                    transfer.insert("name".to_string(), statement.read::<String, _>("name").unwrap());
                                    transfer.insert("issuer".to_string(), statement.read::<String, _>("issuer").unwrap());
                                    transfer.insert("new_owner_and_possessor".to_string(), statement.read::<String, _>("new_owner_and_possessor").unwrap());
                                    transfer.insert("num_shares".to_string(), statement.read::<String, _>("num_shares").unwrap());
                                    transfer.insert("input_size".to_string(), statement.read::<String, _>("input_size").unwrap());
                                    transfer.insert("input_type".to_string(), statement.read::<String, _>("input_type").unwrap());
                                    Ok(Some(transfer))
                                } else {
                                    Ok(None)
                                }
                            },
                            Err(err) => Err(err.to_string())
                        }
                    },
                    Err(err) => {
                        error!("Error in asset_transfer.fetch_all_transfers! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                error!("Error in asset_transfer.fetch_all_transfers! : {}", &err);
                Err(err)
            }
        }
    }

    pub fn fetch_all_transfers(path: &str, asc: &String, limit: i32, offset: u32) -> Result<Vec<HashMap<String, String>>, String> {
        let _prep_query = format!("SELECT * FROM asset_transfer at INNER JOIN transfer t on at.txid=t.txid ORDER BY t.tick {} LIMIT {} OFFSET {};", asc, limit, offset);
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
                                    
                                    //Asset
                                    transfer.insert("name".to_string(), statement.read::<String, _>("name").unwrap());
                                    transfer.insert("issuer".to_string(), statement.read::<String, _>("issuer").unwrap());
                                    transfer.insert("new_owner_and_possessor".to_string(), statement.read::<String, _>("new_owner_and_possessor").unwrap());
                                    transfer.insert("num_shares".to_string(), statement.read::<String, _>("num_shares").unwrap());
                                    transfer.insert("input_size".to_string(), statement.read::<String, _>("input_size").unwrap());
                                    transfer.insert("input_type".to_string(), statement.read::<String, _>("input_type").unwrap());
                                    
                                    //Tx
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
                        error!("Error in asset_transfer.fetch_all_transfers! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                error!("Error in asset_transfer.fetch_all_transfers! : {}", &err);
                Err(err)
            }
        }
    }
}