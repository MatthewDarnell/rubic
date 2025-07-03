use std::collections::HashMap;
use std::str::FromStr;
use sqlite::State;
use identity::Identity;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;


pub mod order {
    use std::collections::HashMap;
    use sqlite::State;
    use logger::error;
    use crate::sqlite::create::open_database;
    use crate::sqlite::crud::prepare_crud_statement;
    use crate::sqlite::get_db_lock;
    pub fn create_qx_order(path: &str, issuer: &str, price: u64, amount: i64, name: &str, input_size: u16, input_type: u16, txid: &str) -> Result<(), String> {
        let prep_query = "INSERT INTO qx_order (name, issuer, price, num_shares, input_size, input_type, txid) VALUES (
    :name, :issuer, :price, :num_shares, :size, :type, :txid
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
                            (":price", price.to_string().as_str()),
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
    

    pub fn fetch_qx_order_by_txid(path: &str, txid: &str) -> Result<Option<HashMap<String, String>>, String> {
        let _prep_query = "SELECT * FROM qx_order WHERE txid = :txid".to_string();
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
                                    transfer.insert("price".to_string(), statement.read::<String, _>("price").unwrap());
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
                        error!("Error in qx.order.fetch_qx_order_by_txid! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                error!("Error in qx.order.fetch_qx_order_by_txid! : {}", &err);
                Err(err)
            }
        }
    }

    pub fn fetch_all_qx_orders(path: &str, asc: &String, limit: i32, offset: u32) -> Result<Vec<HashMap<String, String>>, String> {
        let _prep_query = format!("SELECT * FROM qx_order at INNER JOIN transfer t on at.txid=t.txid ORDER BY t.tick {} LIMIT {} OFFSET {};", asc, limit, offset);
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
                                    transfer.insert("price".to_string(), statement.read::<String, _>("price").unwrap());
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
                        error!("Error in qx.order.fetch_all_qx_orders! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                error!("Error in qx.order.fetch_all_qx_orders! : {}", &err);
                Err(err)
            }
        }
    }
    
}