use std::collections::HashMap;
use crate::api_request_handler::request;

pub fn get_routes() -> Vec<request> {
    vec![
        get_info_request()
    ]
}


fn get_info_request() -> request {
    request::new(
        "/info",
        None,
        Some(info)
    )
}

pub fn info() -> String {
    match store::sqlite::crud::peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{}", value.len())
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}