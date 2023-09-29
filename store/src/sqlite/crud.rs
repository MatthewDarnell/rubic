use std::collections::{HashMap, LinkedList};
use identity::Identity;
use base64::{engine::general_purpose, Engine as _};
use crate::sqlite::create::open_database;
use sqlite::State;

fn prepare_crud_statement<'a>(path: &'a str, connection: &'a sqlite::Connection, prep_query: &'a str) -> Result<sqlite::Statement<'a>, String> {
        match connection.prepare(prep_query) {
            Ok(stmt) => Ok(stmt),
            Err(err) => Err(err.to_string())
        }
}

//    stream: Option<TcpStream>,
//     ping_time: u32,
//     ip_addr: String,
//     nick: String,
//     whitelisted: bool,
//     last_responded: SystemTime,
//     id: String,
//     thread_handle: Option<thread::JoinHandle<()>>

//      id TEXT UNIQUE NOT NULL PRIMARY KEY,
//       ip TEXT UNIQUE NOT NULL,
//       nick TEXT,
//       whitelisted INTEGER,
//       ping UNSIGNED INTEGER,
//       last_responded UNSIGNED INTEGER,
pub mod Peer {
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use sqlite::State;
    use crate::sqlite::crud::{open_database, prepare_crud_statement};
    pub fn create_peer(path: &str, id: &str, ip: &str, nick: &str, ping_time: u32, whitelisted: bool, last_responded: SystemTime) -> Result<(), String> {
        let prep_query = "INSERT INTO peer (id, ip, nick, whitelisted, ping, last_responded) \
        VALUES (:id, :ip, :nick, :whitelisted, :ping_time, :last_responded)\
         ON CONFLICT(ip) DO NOTHING;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
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
                                println!("Peer Created!");
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
                        println!("Error in create_account! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in create_account! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn update_peer_last_responded(path: &str, id: &str, last_responded: SystemTime) -> Result<(), String> {
        let prep_query = "UPDATE peer SET last_responded=:last_responded WHERE id=:id;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
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
                        println!("Error in update_peer_last_responded! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in update_peer_last_responded! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn set_peer_connected(path: &str, id: &str) -> Result<(), String> {
        let prep_query = "UPDATE peer SET connected = true WHERE id=:id;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
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
                        println!("Error in set_peer_connected! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in set_peer_connected! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn set_all_peers_disconnected(path: &str) -> Result<(), String> {
        let prep_query = "UPDATE peer SET connected = false;";
        match open_database(path, true) {
            Ok(connection) => {
                match connection.execute(prep_query) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err.to_string())
                }
            },
            Err(err) => {
                println!("Error in set_all_peers_disconnected! : {}", &err);
                Err(err)
            }
        }
    }


    pub fn fetch_peer_by_ip(path: &str, ip: &str) -> Result<HashMap<String, String>, String> {
        let prep_query = "SELECT * FROM peer WHERE ip = :ip LIMIT 1;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
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
                        println!("Error in fetch_peer_by_ip! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in fetch_peer_by_ip! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn fetch_peer_by_id(path: &str, id: &str) -> Result<HashMap<String, String>, String> {
        let prep_query = "SELECT * FROM peer WHERE id = :id LIMIT 1;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
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
                        println!("Error in fetch_peer_by_id! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in fetch_peer_by_id! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn fetch_all_peers(path: &str) -> Result<Vec<Vec<String>>, String> {
        let prep_query = "SELECT * FROM peer;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
                    Ok(_) => {
                        let mut ret_val: Vec<Vec<String>> = Vec::new();
                        connection
                            .iterate(prep_query, |peers| {
                                let mut each_peer: Vec<String> = Vec::new();
                                for &(name, value) in peers.iter() {
                                    println!("Iterating fetch_all_peers: {}", name);
                                    each_peer.push(value.unwrap().to_string());
                                }
                                ret_val.push(each_peer);
                                true
                            })
                            .unwrap();
                        Ok(ret_val)
                    },
                    Err(err) => {
                        println!("Error in fetch_all_peers! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in fetch_all_peers! : {}", &err);
                Err(err)
            }
        }
    }
    pub fn fetch_connected_peers(path: &str) -> Result<Vec<Vec<String>>, String> {
        let prep_query = "SELECT * FROM peer WHERE connected = true;";
        match open_database(path, true) {
            Ok(connection) => {
                match prepare_crud_statement(path, &connection, prep_query) {
                    Ok(_) => {
                        let mut ret_val: Vec<Vec<String>> = Vec::new();
                        connection
                            .iterate(prep_query, |peers| {
                                let mut each_peer: Vec<String> = Vec::new();
                                for &(name, value) in peers.iter() {
                                    println!("Iterating fetch_all_peers: {}", name);
                                    each_peer.push(value.unwrap().to_string());
                                }
                                ret_val.push(each_peer);
                                true
                            })
                            .unwrap();
                        Ok(ret_val)
                    },
                    Err(err) => {
                        println!("Error in fetch_all_peers! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                println!("Error in fetch_all_peers! : {}", &err);
                Err(err)
            }
        }
    }

}

pub fn create_account(path: &str, identity: &Identity) -> Result<(), String> {
    let prep_query = "INSERT INTO account (name, seed, salt, hash, is_encrypted) VALUES (:name, :seed, :salt, :hash, :is_encrypted);";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { account, hash, salt, identity, seed, encrypted } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":name", account.as_str()),
                                (":seed", seed.as_str()),
                                (":salt", salt.as_str()),
                                (":hash", hash.as_str()),
                                (":is_encrypted", encrypted.to_string().as_str()),
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
                    println!("Error in create_account! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in create_account! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_all_accounts(path: &str) -> Result<LinkedList<String>, String> {
    let prep_query = "SELECT name FROM account;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                        Ok(_) => {
                            let mut ret_val: LinkedList<String> = LinkedList::new();
                            connection
                                .iterate(prep_query, |accounts| {
                                    for &(name, value) in accounts.iter() {
                                        ret_val.push_back(name.to_string());
                                    }
                                    true
                                })
                                .unwrap();
                            Ok(ret_val)
                        },
                Err(err) => {
                    println!("Error in fetch_account! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_account! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_account(path: &str, name: &str) -> Result<Identity, String> {
    let prep_query = "SELECT * FROM account WHERE name = :name LIMIT 1;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":name", name),
                    ][..]) {
                        Ok(_) => {
                            match statement.next() {
                                Ok(State::Row) => {
                                    let id: Identity = Identity::from_vars(
                                        statement.read::<String, _>("name").unwrap().as_str(),
                                        statement.read::<String, _>("seed").unwrap().as_str(),
                                        statement.read::<String, _>("hash").unwrap().as_str(),
                                        statement.read::<String, _>("salt").unwrap().as_str(),
                                        "",
                                        statement.read::<String, _>("is_encrypted").unwrap().as_str() == "true"
                                    );
                                    Ok(id)
                                },
                                Ok(State::Done) => {
                                    println!("Finished Reading. Failed To Fetch Account.");
                                    Err("Account Not Found!".to_string())
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
                    println!("Error in fetch_account! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_account! : {}", &err);
            Err(err)
        }
    }
}


pub fn insert_new_identity(path: &str, identity: &Identity) -> Result<(), String> {

    {   //First time creating an identity for this account
        create_account(path, identity).ok();
    }
    let prep_query = "INSERT INTO identities (account, identity) VALUES (:account, :identity)";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { account, hash, salt, identity, seed, encrypted } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":account", account.as_str()),
                                (":identity", identity.as_str())
                            ][..]) {
                                Ok(val) => {
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
pub fn fetch_all_identities_by_account(path: &str, account: &str) -> Result<LinkedList<String>, String> {
    let prep_query = "SELECT identity FROM identities WHERE account=:account;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":account", account),
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
                    println!("Error in fetch_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_account! : {}", &err);
            Err(err)
        }
    }
}

pub fn fetch_balance_by_identity(path: &str, identity: &str) -> Result<Vec<String>, String> {
    let prep_query = "SELECT * FROM (SELECT * FROM response_entity WHERE identity = :identity ORDER BY tick DESC) GROUP BY peer LIMIT 3;";
    let mut response: Vec<String> = Vec::new();
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity),
                    ][..]) {
                        Ok(_) => {
                            while let Ok(State::Row) = statement.next() {
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
                    println!("Error in fetch_balance_by_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_balance_by_identity! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_identity(path: &str, identity: &str) -> Result<Identity, String> {
    let prep_query = "SELECT * FROM identities i INNER JOIN account a ON a.name=i.account WHERE i.identity = :identity  LIMIT 1;";
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
                                    let id: Identity = Identity::from_vars(
                                        statement.read::<String, _>("name").unwrap().as_str(),
                                        statement.read::<String, _>("seed").unwrap().as_str(),
                                        statement.read::<String, _>("hash").unwrap().as_str(),
                                        statement.read::<String, _>("salt").unwrap().as_str(),
                                        statement.read::<String, _>("identity").unwrap().as_str(),
                                        statement.read::<String, _>("is_encrypted").unwrap().as_str() == "true"
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
                    println!("Error in fetch_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_identity! : {}", &err);
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
                    println!("Error in delete_identity! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in delete_identity! : {}", &err);
            Err(err)
        }
    }
}


pub fn create_peer_response(path: &str, peer: &str, data: &Vec<u8>) -> Result<(), String> {
    let prep_query = "INSERT INTO response (peer, header, type, data) VALUES (:peer, :header, :response_type, :data);";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
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
                    println!("Error in create_account! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in create_account! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_peer_response_by_type(path: &str, response_type: u8) -> Result<Vec<Vec<u8>>, String> {
    let prep_query = "SELECT * FROM response WHERE type = :response_type ORDER BY created DESC;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":response_type", response_type.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<Vec<u8>> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let peer_ip = statement.read::<String, _>("peer").unwrap();
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
                    println!("Error in fetch_peer_response_by_type! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_peer_response_by_type! : {}", &err);
            Err(err)
        }
    }
}

pub fn create_response_entity(path: &str, peer: &str, identity: &str, incoming: u64, outgoing: u64, balance: u64, num_in_txs: u32, num_out_txs: u32, latest_in_tick: u32, latest_out_tick: u32, tick: u32, spectrum_index: i32) -> Result<(), String> {
    let prep_query = "INSERT INTO response_entity (peer, identity, incoming, outgoing, balance, num_in_txs, num_out_txs, latest_in_tick, latest_out_tick, tick, spectrum_index) VALUES (
    :peer, :identity, :incoming, :outgoing, :balance, :num_in_txs, :num_out_txs, :latest_in_tick, :latest_out_tick, :tick, :spectrum_index
    );";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
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
                                        Ok(State::Done) => { return Ok(()); },
                                        Err(error) => { return Err(error.to_string()); },
                                        _ => { return Err("Weird!".to_string()); }
                                    }
                                },
                                Err(err) => { return Err(err.to_string()); }
                            }

                },
                Err(err) => { return Err(err.to_string()); }
            }
        },
        Err(err) => { return Err(err.to_string()); }
    }
}
pub fn fetch_response_entity_by_identity(path: &str, identity: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM response_entity WHERE identity = :identity ORDER BY created DESC;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
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
                    println!("Error in fetch_peer_response_by_type! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_peer_response_by_type! : {}", &err);
            Err(err)
        }
    }
}
pub fn fetch_latest_response_entity_by_identity_group_peers(path: &str, identity: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let prep_query = "SELECT * FROM (SELECT * FROM response_entity WHERE identity = :identity ORDER BY tick DESC) GROUP BY peer;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
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
                    println!("Error in fetch_peer_response_by_type! : {}", &err);
                    Err(err)
                }
            }
        },
        Err(err) => {
            println!("Error in fetch_peer_response_by_type! : {}", &err);
            Err(err)
        }
    }
}


#[cfg(test)]
mod store_crud_tests {
    pub mod peers {
        use std::alloc::System;
        use serial_test::serial;
        use std::fs;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};
        use crate::sqlite::crud::Peer::{create_peer, fetch_peer_by_id, fetch_peer_by_ip, fetch_all_peers, update_peer_last_responded};
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
                    assert_eq!(peer.keys().len(), 6);
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
                    assert_eq!(peer.keys().len(), 6);
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
                    assert_eq!(peer.keys().len(), 6);
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
                    assert_eq!(peer.keys().len(), 6);
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
                    let peer2: &Vec<String> = &peers[1];
                    assert_eq!(peer2.len(), 7);
                    assert_eq!(peer2[2], "nickname2");
                },
                Err(err) => {
                    println!("Peer Couldn't be Fetched!");
                    assert_eq!(1, 2);
                }
            }
            fs::remove_file("test.sqlite").unwrap();
        }
    }
    pub mod accounts {

        use identity::Identity;
        use crate::sqlite::crud::{create_account, insert_new_identity, fetch_identity, delete_identity, fetch_all_accounts};
        use serial_test::serial;
        use std::fs;
        use crate::sqlite::crud::fetch_account;

        #[test]
        #[serial]
        fn create_account_and_insert() {

            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
            create_account("test.sqlite", &id).unwrap();
            fs::remove_file("test.sqlite").unwrap();
        }

        #[test]
        #[serial]
        fn create_account_and_insert_and_fetch() {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
            create_account("test.sqlite", &id).unwrap();
            match fetch_account("test.sqlite", "testAccount") {
                Ok(id) => {
                    assert_eq!(id.account.as_str(), "testAccount");
                },
                Err(err) => {
                    println!("Account Couldn't be Fetched!");
                    assert_eq!(1, 2);
                }
            }
            fs::remove_file("test.sqlite").unwrap();
        }

        #[test]
        #[serial]
        fn create_multiple_accounts_and_insert_and_fetch() {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
            let id2: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount2");
            let id3: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount3");
            create_account("test.sqlite", &id).unwrap();
            create_account("test.sqlite", &id2).unwrap();
            create_account("test.sqlite", &id3).unwrap();
            match fetch_all_accounts("test.sqlite",) {
                Ok(list) => {
                    assert_eq!(list.len(), 3);
                },
                Err(err) => {
                    println!("Account Couldn't be Fetched!");
                    assert_eq!(1, 2);
                }
            }
            fs::remove_file("test.sqlite").unwrap();
        }
    }

    pub mod response {
        use crate::sqlite::crud::{create_response_entity, fetch_latest_response_entity_by_identity_group_peers, fetch_response_entity_by_identity};
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

    pub mod response_entity {
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
                    println!("Account Couldn't be Fetched!");
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

    pub mod identities {
        use identity::Identity;
        use crate::sqlite::crud::{insert_new_identity, fetch_identity, delete_identity, fetch_all_identities_by_account};
        use serial_test::serial;
        use std::fs;
        #[test]
        #[serial]
        fn create_identity_and_insert() {
            {
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
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
            {
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
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
            {
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
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


        #[test]
        #[serial]
        fn create_identities_and_insert_and_fetch_all_by_account() {
            {
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
                let id2: Identity = Identity::new("twohvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
                let id3: Identity = Identity::new("threebvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount");
                println!("{:?}", &id);
                insert_new_identity("test.sqlite", &id).unwrap();
                insert_new_identity("test.sqlite", &id2).unwrap();
                match insert_new_identity("test.sqlite", &id3) {
                    Ok(_) => {
                        match fetch_all_identities_by_account("test.sqlite", "testAccount") {
                            Ok(identities) => {
                                println!("{:?}", &identities);
                                assert_eq!(identities.len(), 3);
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
                let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount",);
                id =  id.encrypt_identity("password").unwrap();
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
}
