use rocket::get;
use store;

#[get("/tick")]
pub fn latest_tick() -> String {
    match store::sqlite::tick::fetch_latest_tick(store::get_db_path().as_str()) {
        Ok(tick) => format!("{}", tick),
        Err(err) => format!("{}", err.to_string())
    }
}

/// How the wallet is keeping up with the network: request queue depth (total and
/// tick polls), peers in the working set, and the latest known tick.
#[get("/health")]
pub fn health() -> String {
    let (backlog, tick_backlog, peers) = network::peers::queue_stats();
    let latest_tick = store::sqlite::tick::fetch_latest_tick(store::get_db_path().as_str()).unwrap_or_default();
    let tick_json = if latest_tick.chars().all(|c| c.is_ascii_digit()) && !latest_tick.is_empty() { latest_tick } else { "null".to_string() };
    format!(
        "{{\"backlog\": {}, \"tick_backlog\": {}, \"routine_backlog\": {}, \"routine_limit\": {}, \"peers\": {}, \"latest_tick\": {}}}",
        backlog,
        tick_backlog,
        backlog.saturating_sub(tick_backlog),
        network::peers::MAX_REQUEST_BACKLOG,
        peers,
        tick_json
    )
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

