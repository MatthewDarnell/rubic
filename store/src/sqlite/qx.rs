use std::collections::HashMap;
use std::str::FromStr;
use sqlite::State;
use protocol::identity::Identity;
use logger::error;
use crate::sqlite::create::open_database;
use crate::sqlite::crud::prepare_crud_statement;
use crate::sqlite::get_db_lock;


pub mod orderbook {
    use std::collections::HashMap;
    use sqlite::State;
    use logger::error;
    use crypto::qubic_identities::{get_identity, get_public_key_from_identity};
    use smart_contract::qx::orderbook::OrderBook;
    use crate::sqlite::create::open_database;
    use crate::sqlite::crud::prepare_crud_statement;
    use crate::sqlite::get_db_lock;

    
    pub fn create_qx_orderbook(path: &str, asset: &str, side: &str, orders: &OrderBook) -> Result<(), String> {
        let prep_query: &str = "INSERT INTO qx_orderbook (asset, side, entity, price, num_shares, stale, offset_at_price) VALUES (:asset, :side, :entity, :price, :num_shares, 0, :offset_at_price) \
        ON CONFLICT DO UPDATE SET stale = 0 WHERE asset=:asset AND side=:side AND entity=:entity AND price=:price AND num_shares=:num_shares;";
        let _lock = get_db_lock().lock().unwrap();
        match open_database(path, false) {
            Ok(connection) => {
                connection.execute("BEGIN TRANSACTION;").unwrap();
                match prepare_crud_statement(&connection, "UPDATE qx_orderbook SET stale = 1 WHERE side=:side AND asset = :asset;") {
                    Ok(mut stmt) => {
                        match stmt.bind::<&[(&str, &str)]>(&[
                            (":side", side),
                            (":asset", asset),
                        ][..]) {
                            Ok(_) => {
                                match stmt.next() {
                                    Ok(State::Done) => {},
                                    _ => {}
                                }
                            },
                            Err(err) => {
                                println!("error: {}", err.to_string());
                                //return Err(err.to_string());
                            }
                        }
                    },
                    Err(_e) => {}
                }
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        let mut offset_at_price: i32 = -1;  //Keep track of FIFO for orders
                        let mut curr_price: i64 = 0;        // at same price
                        for order in orders.full_order_list.iter() {
                            if order.price != curr_price {  //New Price Point, Reset Order Offsets
                                offset_at_price = 0;
                                curr_price = order.price;
                            } else {    //Same Price as Before, Increment Order Offset
                                offset_at_price += offset_at_price + 1;
                            }
                            if asset=="CFB" && side == "A" {
                                println!("Inserting: {} - {} - price.{} - offset.{}", asset, get_identity(&order.entity).as_str(), order.price, offset_at_price);
                            }
                            match statement.bind::<&[(&str, &str)]>(&[
                                (":asset", asset),
                                (":side", side),
                                (":entity", get_identity(&order.entity).as_str()),
                                (":price", order.price.to_string().as_str()),
                                (":num_shares", order.num_shares.to_string().as_str()),
                                (":offset_at_price", offset_at_price.to_string().as_str()),
                            ][..]) {
                                Ok(_) => {
                                    match statement.next() {
                                        Ok(State::Done) => {},
                                        Err(error) => {
                                            println!("Failed Inserting Order From QX OrderBook: {}", error.to_string());
                                        },
                                        _ => {}
                                    }
                                },
                                Err(err) => { 
                                    println!("error: {}", err.to_string());
                                    //return Err(err.to_string());
                                }
                            }   
                            statement.reset().unwrap();
                        }
                        match prepare_crud_statement(&connection, "DELETE FROM qx_orderbook WHERE stale=1 AND side=:side AND asset = :asset;") {
                            Ok(mut stmt) => {
                                match stmt.bind::<&[(&str, &str)]>(&[
                                    (":side", side),
                                    (":asset", asset),
                                ][..]) {
                                    Ok(_) => {
                                        match stmt.next() {
                                            Ok(State::Done) => {
                                                connection.execute("COMMIT;").unwrap();
                                            },
                                            _ => {}
                                        }
                                    },
                                    Err(err) => {
                                        println!("error: {}", err.to_string());
                                        //return Err(err.to_string());
                                    }
                                }
                            },
                            Err(_e) => {}
                        }
                        Ok(())
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

    pub fn fetch_qx_orderbook(path: &str, asset: &str, side: &str, limit: i32, offset: u32) -> Result<Vec<HashMap<String, String>>, String> {
        let s = match side {
            "ASK" => "A",
            _ => "B"
        };
        let asc = match s {
            "A" => "ASC",
            _ => "DESC"
        };
        
        let _prep_query = format!("SELECT * FROM qx_orderbook WHERE asset=:asset AND side=:side ORDER BY price {} LIMIT {} OFFSET {};", asc, limit, offset);
        let prep_query = _prep_query.as_str();
        //let _lock =SQLITE_TRANSFER_MUTEX.lock().unwrap();
        let _lock = get_db_lock().lock().unwrap();
        match open_database(path, false) {
            Ok(connection) => {
                match prepare_crud_statement(&connection, prep_query) {
                    Ok(mut statement) => {
                        match statement.bind::<&[(&str, &str)]>(&[
                            (":asset", asset),
                            (":side", s),
                        ][..]) {
                            Ok(_) => {
                                let mut response: Vec<HashMap<String, String>> = vec![];
                                while let Ok(State::Row) = statement.next() {
                                    let mut order: HashMap<String, String> = HashMap::new();
                                    order.insert("asset".to_string(), statement.read::<String, _>("asset").unwrap());
                                    order.insert("entity".to_string(), statement.read::<String, _>("entity").unwrap());
                                    order.insert("price".to_string(), statement.read::<String, _>("price").unwrap());
                                    order.insert("num_shares".to_string(), statement.read::<String, _>("num_shares").unwrap());
                                    order.insert("side".to_string(), statement.read::<String, _>("side").unwrap());
                                    response.push(order);
                                }
                                Ok(response)
                            },
                            Err(err) => Err(err.to_string())
                        }
                    },
                    Err(err) => {
                        error!("Error in qx.orderbook.fetch_qx_orderbook! : {}", &err);
                        Err(err)
                    }
                }
            },
            Err(err) => {
                error!("Error in qx.orderbook.fetch_qx_orderbook! : {}", &err);
                Err(err)
            }
        }
    }
}

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
        match open_database(path, false) {
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