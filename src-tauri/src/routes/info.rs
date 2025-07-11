use rocket::get;
use store;

#[get("/tick")]
pub fn latest_tick() -> String {
    match store::sqlite::tick::fetch_latest_tick(store::get_db_path().as_str()) {
        Ok(tick) => format!("{}", tick),
        Err(err) => format!("{}", err.to_string())
    }
}

#[get("/info")]
pub fn info() -> String {
    match store::sqlite::peer::fetch_connected_peers(store::get_db_path().as_str()) {
        Ok(value) => {
            format!("{}", value.len())
        }, Err(err) => {
            format!("Error! : {}", err.to_string())
        }
    }
}

