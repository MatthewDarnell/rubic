use std::collections::LinkedList;
use api::qubic_api_t;
use identity::Identity;
use base64::{engine::general_purpose, Engine as _};
use crate::sqlite::create::open_database;
use sqlite::State;
use api::response::response_entity::ResponseEntity;

fn prepare_crud_statement<'a>(path: &'a str, connection: &'a sqlite::Connection, prep_query: &'a str) -> Result<sqlite::Statement<'a>, String> {
        match connection.prepare(prep_query) {
            Ok(stmt) => Ok(stmt),
            Err(err) => Err(err.to_string())
        }
}

pub fn create_account(path: &str, identity: &Identity) -> Result<(), String> {
    let prep_query = "INSERT INTO account (name, seed, salt, hash, is_encrypted) VALUES (:name, :seed, :salt, :hash, :is_encrypted);";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { account, hash, salt, identity, index, seed, encrypted } => {
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
                                        0,
                                        statement.read::<String, _>("is_encrypted").unwrap().as_str() == "true",
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
    let prep_query = "INSERT INTO identities (account, identity_index, identity) VALUES (:account, :index, :identity)";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match identity {
                        Identity { account, hash, salt, identity, index, seed, encrypted } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":account", account.as_str()),
                                (":index", index.to_string().as_str()),
                                (":identity", identity.as_str()),
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
                                    let index: u32 = statement.read::<String, _>("identity_index").unwrap().parse().unwrap();
                                    let id: Identity = Identity::from_vars(
                                        statement.read::<String, _>("name").unwrap().as_str(),
                                        statement.read::<String, _>("seed").unwrap().as_str(),
                                        statement.read::<String, _>("hash").unwrap().as_str(),
                                        statement.read::<String, _>("salt").unwrap().as_str(),
                                        statement.read::<String, _>("identity").unwrap().as_str(),
                                        index,
                                        statement.read::<String, _>("is_encrypted").unwrap().as_str() == "true",
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
pub fn fetch_peer_response_by_type(path: &str, response_type: u8) -> Result<Vec<qubic_api_t>, String> {
    let prep_query = "SELECT * FROM response WHERE type = :response_type ORDER BY created DESC;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":response_type", response_type.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<qubic_api_t> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let peer_ip = statement.read::<String, _>("peer").unwrap();
                                let mut header_bytes: Vec<u8> = general_purpose::STANDARD.decode(statement.read::<String, _>("header").unwrap()).unwrap();
                                let mut data_bytes: Vec<u8> = general_purpose::STANDARD.decode(statement.read::<String, _>("data").unwrap()).unwrap();
                                header_bytes.append(&mut data_bytes);
                                match qubic_api_t::format_response_from_bytes(&peer_ip, header_bytes) {
                                    Some(t) => response.push(t),
                                    None => {}
                                }
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
//        identity TEXT NOT NULL,
//         incoming INTEGER NOT NULL,
//         outgoing INTEGER NOT NULL,
//         balance INTEGER NOT NULL,
//         num_in_txs INTEGER NOT NULL,
//         num_out_txs INTEGER NOT NULL,
//         latest_in_tick INTEGER NOT NULL,
//         latest_out_tick INTEGER NOT NULL,
//         tick INTEGER NOT NULL,
//         spectrum_index INTEGER NOT NULL,
//         created DATETIME DEFAULT CURRENT_TIMESTAMP,
//         FOREIGN KEY(identity) REFERENCES identities(identity)

pub fn create_response_entity(path: &str, response_entity: &ResponseEntity) -> Result<(), String> {
    let prep_query = "INSERT INTO response_entity (peer, identity, incoming, outgoing, balance, num_in_txs, num_out_txs, latest_in_tick, latest_out_tick, tick, spectrum_index) VALUES (
    :peer, :identity, :incoming, :outgoing, :balance, :num_in_txs, :num_out_txs, :latest_in_tick, :latest_out_tick, :tick, :spectrum_index
    );";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match response_entity {
                        ResponseEntity { peer, identity, incoming, outgoing, final_balance, number_outgoing_transactions, number_incoming_transactions, latest_outgoing_transfer_tick, latest_incoming_transfer_tick, tick, spectrum_index } => {
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":peer", peer.as_str()),
                                (":identity", identity.as_str()),
                                (":incoming", incoming.to_string().as_str()),
                                (":outgoing", outgoing.to_string().as_str()),
                                (":balance", final_balance.to_string().as_str()),
                                (":num_in_txs", number_incoming_transactions.to_string().as_str()),
                                (":num_out_txs", number_outgoing_transactions.to_string().as_str()),
                                (":latest_in_tick", latest_incoming_transfer_tick.to_string().as_str()),
                                (":latest_out_tick", latest_outgoing_transfer_tick.to_string().as_str()),
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
                    }
                },
                Err(err) => { return Err(err.to_string()); }
            }
        },
        Err(err) => { return Err(err.to_string()); }
    }
}




pub fn fetch_response_entity_by_identity(path: &str, identity: &str) -> Result<Vec<ResponseEntity>, String> {
    let prep_query = "SELECT * FROM response_entity WHERE identity = :identity ORDER BY created DESC;";
    match open_database(path, true) {
        Ok(connection) => {
            match prepare_crud_statement(path, &connection, prep_query) {
                Ok(mut statement) => {
                    match statement.bind::<&[(&str, &str)]>(&[
                        (":identity", identity.to_string().as_str()),
                    ][..]) {
                        Ok(_) => {
                            let mut response: Vec<ResponseEntity> = vec![];
                            while let Ok(State::Row) = statement.next() {
                                let peer_ip = statement.read::<String, _>("peer").unwrap();
                                let identity = statement.read::<String, _>("identity").unwrap();
                                let incoming = statement.read::<String, _>("incoming").unwrap();
                                let outgoing = statement.read::<String, _>("outgoing").unwrap();
                                let num_in_txs = statement.read::<String, _>("num_in_txs").unwrap();
                                let num_out_txs = statement.read::<String, _>("num_out_txs").unwrap();
                                let latest_in_tick = statement.read::<String, _>("latest_in_tick").unwrap();
                                let latest_out_tick = statement.read::<String, _>("latest_out_tick").unwrap();
                                let tick = statement.read::<String, _>("tick").unwrap();
                                let spectrum_index = statement.read::<String, _>("spectrum_index").unwrap();

                                                                response.push(ResponseEntity::new(
                                    identity.as_str(),
                                    peer_ip.as_str(),
                                                                    incoming.parse().unwrap(),
                                                                    outgoing.parse().unwrap(),
                                    num_in_txs.parse().unwrap(),
                                    num_out_txs.parse().unwrap(),
                                    latest_in_tick.parse().unwrap(),
                                    latest_out_tick.parse().unwrap(),
                                    tick.parse().unwrap(),
                                    spectrum_index.parse().unwrap()
                                                                ));
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
    pub mod accounts {

        use identity::Identity;
        use crate::sqlite::crud::{create_account, insert_new_identity, fetch_identity, delete_identity, fetch_all_accounts};
        use serial_test::serial;
        use std::fs;
        use crate::sqlite::crud::fetch_account;

        #[test]
        #[serial]
        fn create_account_and_insert() {

            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            create_account("test.sqlite", &id).unwrap();
            fs::remove_file("test.sqlite").unwrap();
        }

        #[test]
        #[serial]
        fn create_account_and_insert_and_fetch() {
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
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
            let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
            let id2: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount2", 0);
            let id3: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount3", 0);
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
        use crate::sqlite::crud::{create_response_entity, fetch_response_entity_by_identity};
        use serial_test::serial;
        use std::fs;
        use api::response::response_entity::ResponseEntity;

        #[test]
        #[serial]
        fn create_response_entity_and_insert_and_fetch() {
                let response: ResponseEntity = ResponseEntity::new("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON", "127.0.0.1", 1000, 0, 1, 0, 100, 100, 1000, 1);
            create_response_entity("test.sqlite", &response).unwrap();
            create_response_entity("test.sqlite", &response).unwrap();
                match fetch_response_entity_by_identity("test.sqlite", "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON") {
                    Ok(response_vec) => {
                    assert_eq!(response_vec.len(), 2);
                        assert_eq!(response_vec[0].identity.as_str(), "EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                    },
                    Err(err) => {
                        println!("ResponeEntity Couldn't be Fetched!");
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
                    assert_eq!(response_vec[0].peer.as_ref().unwrap().as_str(), "127.0.0.1");
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
        use crate::sqlite::crud::{create_account, insert_new_identity, fetch_identity, delete_identity, fetch_all_identities_by_account};
        use serial_test::serial;
        use std::fs;
        #[test]
        #[serial]
        fn create_identity_and_insert() {
            {
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
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
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
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
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
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
                let id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
                let id2: Identity = Identity::new("twohvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 1);
                let id3: Identity = Identity::new("threebvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 2);
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
                let mut id: Identity = Identity::new("lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf", "testAccount", 0);
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
