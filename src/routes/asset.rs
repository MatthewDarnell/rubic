use std::collections::HashMap;
use rocket::get;
use crypto::qubic_identities::get_identity;
use logger::{debug, error, info};
use store::{get_db_path, sqlite};
use store::sqlite::asset::{fetch_asset_balance, fetch_issued_assets};
use crate::routes::MINPASSWORDLEN;

#[get("/asset/balance/<asset>/<address>")]
pub fn balance(asset: &str, address: &str) -> String {
    match fetch_asset_balance(get_db_path().as_str(), asset, address) {
        Ok(value) => { format!("{:?}", value) },
        Err(error) => format!("{}", error)
    }
}
#[get("/asset/balance/<address>")]
pub fn all_asset_balances(address: &str) -> String {
    match fetch_issued_assets(get_db_path().as_str()) {
        Ok(assets) => {
            let mut balances: Vec<HashMap<String, String>> = Vec::new();
            for asset in assets.iter() {
                match fetch_asset_balance(get_db_path().as_str(), asset, address) {
                    Ok(value) => { 
                        if value.contains_key(&"balance".to_string()) {
                            let balance = value.get(&"balance".to_string()).unwrap();
                            if balance.len() > 0 && *balance != "0".to_string() {
                                balances.push(value);
                            }
                        }
                    },
                    Err(error) => {
                        return format!("{}", error);
                    }
                }
            }
            format!("{:?}", balances)
        },
        Err(error) => format!("{}", error)
    }
}

#[get("/asset/issued")]
pub fn get_assets() -> String {
    match fetch_issued_assets(get_db_path().as_str()) {
        Ok(assets) => {
            format!("{:?}", assets)
        },
        Err(err) => format!("{}", err)
    }
}

#[get("/asset/transfer/<asc>/<limit>/<offset>")]
pub fn fetch_transfers(asc: u8, limit: u32, offset: u32) -> String {
    let order: String = match asc {
        1 => "ASC".to_string(),
        _ => "DESC".to_string()
    };

    let _limit: i32 = match limit {
        0 => -1,
        _ => limit as i32
    };
    match sqlite::asset::asset_transfer::fetch_all_transfers(get_db_path().as_str(), &order, _limit, offset) {
        Ok(txs) => format!("{:?}", txs),
        Err(e) => {
            println!("Error Fetching Asset Transfers: {}", e);
            format!("Error Fetching Asset Transfers.")
        }
    }    
}

#[get("/asset/transfer/<asset_name>/<issuer>/<source>/<dest>/<amount>/<expiration>/<password>")]
pub fn transfer(asset_name: &str, issuer: &str, source: &str, dest: &str, amount: &str, expiration: &str, password: &str) -> String {
    let source_identity: String = source.to_string();
    let dest_identity: String = dest.to_string();

    if asset_name.len() > 8 {
        return "Invalid Asset Name!".to_string();
    }
    
    if source_identity.len() != 60 {
        return "Invalid Source Identity!".to_string();
    }

    if dest_identity.len() != 60 {
        return "Invalid Destination Identity!".to_string();
    }


    let mut source_identity = match sqlite::identity::fetch_identity(get_db_path().as_str(), source) {
        Ok(identity) => identity,
        Err(_) => {
            error!("Failed To Make Transfer, Unknown Identity {}", source);
            return "Unknown Identity".to_string();
        }
    };

    if source_identity.encrypted {
        if password.len() >= MINPASSWORDLEN {
            source_identity = match sqlite::master_password::get_master_password(get_db_path().as_str()) {
                Ok(master_password) => {
                    match crypto::passwords::verify_password(password, master_password[1].as_str()) {
                        Ok(verified) => {
                            if !verified {
                                error!("Failed To Create Transfer; Invalid Password");
                                return "Invalid Password".to_string();
                            } else {
                                match source_identity.decrypt_identity(password) {
                                    Ok(identity) => identity,
                                    Err(_) => {
                                        error!("Failed To Create Transfer; Invalid Password For This Identity");
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
        debug!("Creating Transfer, Wallet Is Not Encrypted!");
    }
    let amt: i64 = amount.parse().unwrap();
    let tck: u32 = expiration.parse().unwrap();

    info!("Creating Asset Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &source_identity.identity.as_str(), amt.to_string().as_str(), dest, tck.to_string().as_str());
    let transfer_tx = api::asset_transfer::AssetTransferTransaction::from_vars(&source_identity, asset_name.to_uppercase().as_str(), issuer, dest, amt, tck);
    let txid = transfer_tx.txid();

    let sig = transfer_tx._signature;
    let sig_str = hex::encode(sig);

    match sqlite::transfer::create_transfer(
        get_db_path().as_str(),
        source_identity.identity.as_str(),
        get_identity(<&[u8; 32]>::try_from(transfer_tx.tx._source_destination_public_key.as_slice()).unwrap()).as_str(),
        transfer_tx.tx._amount,
        transfer_tx.tx._tick,
        sig_str.as_str(),
        txid.as_str()
    ) {
        Ok(_) => {
            match sqlite::asset::asset_transfer::create_asset_transfer(get_db_path().as_str(), 
                                                                       issuer, 
                                                                       dest_identity.as_str(), 
                                                                       amt, 
                                                                       asset_name.to_uppercase().as_str(), 
                                                                       transfer_tx.tx._input_size,
                                                                       transfer_tx.tx._input_type,
                                                                       txid.as_str()) {
                Ok(_) => txid,
                Err(_) => "Error Creating Asset Transfer".to_string()
            }
        },
        Err(err) => {
            println!("Error Inserting Tx into Db: {}", err);
            "Error Creating Asset Transfer".to_string()
        }
    }
}