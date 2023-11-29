use std::convert::TryInto;
use dotenv::dotenv;
use logger::debug;


pub fn get_host() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_HOST") {
        Ok(v) => v,
        Err(_) => {
            debug("RUBIC_HOST not found in env vars! Defaulting...");
            let default_host: String = "127.0.0.1".to_string();
            debug("Using RUBIC_HOST: <127.0.0.1>");
            return default_host;
        }
    }
}
pub fn get_port() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_PORT") {
        Ok(v) => v,
        Err(_) => {
            debug("RUBIC_PORT not found in env vars! Defaulting...");
            let default_port: String = "8080".to_string();
            debug("Using RUBIC_PORT: <8080>");
            return default_port;
        }
    }
}

pub fn get_min_peers() -> usize {
    dotenv().ok();
    return match std::env::var("RUBIC_MIN_PEERS") {
        Ok(v) => {
            match v.parse::<usize>() {
                Ok(value) => value,
                Err(err) => {
                    println!("Invalid RUBIC_MIN_PEERS in env vars. ({}) -> {:?} Defaulting...", v.as_str(), err);
                    return 3;
                }
            }
        },
        Err(_) => {
            debug("RUBIC_MIN_PEERS not found in env vars! Defaulting...");
            let default_min_peers: usize = 3;
            debug("Using RUBIC_MIN_PEERS: <3>");
            return default_min_peers;
        }
    }
}