use std::collections::HashMap;
use rocket::get;
use store::get_db_path;
use store::sqlite::asset::{fetch_asset_balance, fetch_issued_assets};

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
