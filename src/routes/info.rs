
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;
use spmc::Receiver;
use rocket::get;
use uuid::Uuid;
use store;
use identity;
use crypto;
use logger::{debug, error};

#[get("/tick")]
pub fn latest_tick() -> String {
    match store::sqlite::crud::fetch_latest_tick(store::get_db_path().as_str()) {
        Ok(tick) => format!("{}", tick),
        Err(err) => format!("{}", err.to_string())
    }
}

#[get("/info")]
pub fn info() -> String {
    match store::sqlite::crud::peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{}", value.len())
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}

