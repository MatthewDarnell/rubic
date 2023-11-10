use dotenv::dotenv;
pub fn get_host() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_HOST") {
        Ok(v) => v,
        Err(_) => {
            println!("RUBIC_HOST not found in env vars! Defaulting...");
            let default_host: String = "127.0.0.1".to_string();
            println!("Using RUBIC_HOST: <{}>", default_host.as_str());
            return default_host;
        }
    }
}
pub fn get_port() -> String {
    dotenv().ok();
    return match std::env::var("RUBIC_PORT") {
        Ok(v) => v,
        Err(_) => {
            println!("RUBIC_PORT not found in env vars! Defaulting...");
            let default_port: String = "8080".to_string();
            println!("Using RUBIC_PORT: <{}>", default_port.as_str());
            return default_port;
        }
    }
}