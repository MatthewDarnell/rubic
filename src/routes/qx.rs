use std::str::FromStr;
use rocket::{get, post};
use rocket::serde::{Deserialize, json::Json};
use crypto::qubic_identities::get_identity;
use logger::{debug, error, info};
use store::{get_db_path, sqlite};
use store::sqlite::asset::fetch_asset_balance;
use smart_contract::qx::order;
use store::sqlite::tick::fetch_latest_tick;
use crate::routes::MINPASSWORDLEN;

/// Seconds since each side of an asset's book was last received from a peer.
/// `null` means never (since this start of the wallet).
#[get("/qx/book_age/<asset>")]
pub fn book_age(asset: &str) -> String {
    let (ask, bid) = api::response::book_age_seconds(asset);
    let fmt = |v: Option<u64>| v.map(|s| s.to_string()).unwrap_or_else(|| "null".to_string());
    format!("{{\"asset\": \"{}\", \"ask\": {}, \"bid\": {}}}", asset, fmt(ask), fmt(bid))
}

/// Every resting QX order of the wallet's identities, as reported by the QX
/// contract itself (EntityAskOrders / EntityBidOrders), refreshed every 10 s.
#[get("/qx/open_orders")]
pub fn open_orders() -> String {
    match sqlite::qx::entity_orders::fetch_entity_orders(get_db_path().as_str()) {
        Ok(rows) => format!("{:?}", rows),
        Err(err) => err,
    }
}

// `?refresh=1` marks the book the user is looking at: it is refreshed from peers
// ahead of the background sweep. Background scans (open orders) omit it.
#[get("/qx/orderbook/<asset>/<ask_bid>/<limit>/<offset>?<refresh>")]
pub fn get_orderbook(asset: &str, ask_bid: &str, limit: i32, offset: u32, refresh: Option<bool>) -> String {
    if refresh.unwrap_or(false) {
        crate::peer_loop::qx::request_priority_refresh(asset);
    }
    match sqlite::qx::orderbook::fetch_qx_orderbook(get_db_path().as_str(), asset, ask_bid, limit, offset) {
        Ok(r) => format!("{:?}", r),
        Err(err) => err
    }
}

#[get("/qx/orders/<asc>/<limit>/<offset>")]
pub fn fetch_orders(asc: u8, limit: u32, offset: u32) -> String {
    let order: String = match asc {
        1 => "ASC".to_string(),
        _ => "DESC".to_string()
    };

    let _limit: i32 = match limit {
        0 => -1,
        _ => limit as i32
    };
    match sqlite::qx::order::fetch_all_qx_orders(get_db_path().as_str(), &order, _limit, offset) {
        Ok(txs) => format!("{:?}", txs),
        Err(e) => {
            println!("Error Fetching QX Orders: {}", e);
            format!("Error Fetching  QX Orders.")
        }
    }
}

/// Body of `POST /qx/order`. `side` is ASK, BID, REMOVEASK or REMOVEBID; `tick` 0
/// (or omitted) means the latest known tick; `password` may be omitted when the
/// wallet is unlocked or the identity's seed is stored unencrypted.
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct QxOrderRequest {
    #[serde(default)]
    pub tick: u32,
    pub issuer: String,
    pub asset: String,
    pub side: String,
    pub address: String,
    pub price: u64,
    pub amount: u64,
    #[serde(default)]
    pub password: String,
}

// POST /qx/order  {"tick", "issuer", "asset", "side", "address", "price", "amount", "password"}
#[post("/qx/order", format = "json", data = "<body>")]
pub fn place_order(body: Json<QxOrderRequest>) -> String {
    let QxOrderRequest { tick, issuer, asset, side, address, price, amount, password } = body.into_inner();
    let issuer: &str = issuer.as_str();
    let asset: &str = asset.as_str();
    let ask_bid: &str = side.as_str();
    let address: &str = address.as_str();
    let password: &str = password.as_str();
    let _identity: String = address.to_string();
    if asset.len() > 8 {
        return "Invalid Asset!".to_string();
    }
    if _identity.len() != 60 {
        return "Invalid Identity!".to_string();
    }
    let _procedure: order::QxProcedure = match ask_bid.to_uppercase().as_str() {
        "ASK" => order::QxProcedure::QxAddAskOrder,
        "BID" => order::QxProcedure::QxAddBidOrder,
        "REMOVEASK" => order::QxProcedure::QxRemoveAskOrder,
        "REMOVEBID" => order::QxProcedure::QxRemoveBidOrder,
        _ => { return "Invalid QX Order Type!".to_string(); }
    };
    
    let tick_to_use = match tick {
        0 => u32::from_str(fetch_latest_tick(get_db_path().as_str()).unwrap().as_str()).unwrap(),
        _ => tick
    };
    
    
    let mut identity = match sqlite::identity::fetch_identity(get_db_path().as_str(), _identity.as_str()) {
        Ok(identity) => identity,
        Err(_) => {
            error!("Failed To Make QX Order, Unknown Identity {}", _identity);
            return "Unknown Identity".to_string();
        }
    };

    if identity.encrypted && protocol::wallet_unlock::is_wallet_unlocked().unwrap() {
        identity = identity.decrypt_identity_unlocked_wallet().unwrap();
    }

    if identity.encrypted {
        if password.len() >= MINPASSWORDLEN {
            identity = match sqlite::master_password::get_master_password(get_db_path().as_str()) {
                Ok(master_password) => {
                    match crypto::passwords::verify_password(password, master_password[1].as_str()) {
                        Ok(verified) => {
                            if !verified {
                                error!("Failed To Create QX Order; Invalid Password");
                                return "Invalid Password".to_string();
                            } else {
                                match identity.decrypt_identity(password) {
                                    Ok(identity) => identity,
                                    Err(_) => {
                                        error!("Failed To Create QX Order; Invalid Password For This Identity");
                                        return "Invalid Password For This Identity!".to_string();
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            error!("Failed To Verify Master Password Vs Supplied Password");
                            return "Failed To Verify Master Password Vs Supplied Password!".to_string();
                        }
                    }
                },
                Err(_) => {
                    error!("Identity Is Encrypted, Yet No Master Password Set! Weird");
                    return "Identity Is Encrypted, Yet No Master Password Set! Weird!".to_string();
                }
            };
        } else {
            error!("Failed To Decrypt Password For Transfer; No Password Supplied");
            return "Must Enter A Password!".to_string();
        }
    } else {
        debug!("Creating QX Order, Wallet Is Not Encrypted!");
    }
    
    match fetch_asset_balance(get_db_path().as_str(), asset, address) {
        Ok(_) => {  //todo: enforce sufficient balance
            info!("Creating QX Order: {} .({}) ---> {} (Expires At Tick.<{}>)", &identity.identity.as_str(), amount.to_string().as_str(), price, tick_to_use.to_string().as_str());
            let order_tx = smart_contract::qx::order::QxOrderTransaction::from_vars(_procedure, &identity, asset.to_uppercase().as_str(), issuer, price, amount, tick_to_use);
            let txid = order_tx.txid();

            let sig = order_tx._signature;
            let sig_str = hex::encode(sig);

            match sqlite::transfer::create_transfer(
                get_db_path().as_str(),
                identity.identity.as_str(),
                get_identity(<&[u8; 32]>::try_from(order_tx.tx._source_destination_public_key.as_slice()).unwrap()).as_str(),
                order_tx.tx._amount,
                order_tx.tx._tick,
                sig_str.as_str(),
                txid.as_str()
            ) {
                Ok(_) => {
                    match sqlite::qx::order::create_qx_order(get_db_path().as_str(),
                                                                               issuer,
                                                                               price,
                                                                               amount as i64,
                                                                               asset.to_uppercase().as_str(),
                                                                               order_tx.tx._input_size,
                                                                               order_tx.tx._input_type,
                                                                               txid.as_str()) {
                        Ok(_) => txid,
                        Err(_) => "Error Creating QX Order".to_string()
                    }
                },
                Err(err) => {
                    println!("Error Inserting Tx into Db: {}", err);
                    "Error Creating QX Order".to_string()
                }
            }
        },
        Err(error) => format!("{}", error)
    }
}