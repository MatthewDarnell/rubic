#![feature(proc_macro_hygiene, decl_macro)]
use rocket::{get, routes};

#[macro_use]
extern crate dotenv_codegen;


use std::sync::mpsc::{Sender, Receiver};
use std::sync::*;
use std::time::Duration;

static mut SENDER: Option<Mutex<Sender<String>>> = None;


mod environment {
    use dotenv::dotenv;
    pub fn get_host() -> String {
        dotenv().ok();
        return match std::env::var("RUBIC_HOST") {
            Ok(v) => v,
            Err(err) => {
                println!("RUBIC_HOST not found in env vars! Defaulting...");
                let mut default_host: String = "127.0.0.1".to_string();
                println!("Using RUBIC_HOST: <{}>", default_host.as_str());
                return default_host;
            }
        }
    }
    pub fn get_port() -> String {
        dotenv().ok();
        return match std::env::var("RUBIC_PORT") {
            Ok(v) => v,
            Err(err) => {
                println!("RUBIC_PORT not found in env vars! Defaulting...");
                let mut default_port: String = "8080".to_string();
                println!("Using RUBIC_PORT: <{}>", default_port.as_str());
                return default_port;
            }
        }
    }
}




#[get("/info")]
fn info() -> String {
    format!("Hello, year old named!")
}
pub fn start_server(tx: Sender<String>) -> Result<(), rocket::Error> {
    println!("In Start Server");
    unsafe {
        SENDER = Some(Mutex::new(tx));
    }
    let host = environment::get_host();
    let port: u32 = match environment::get_port().parse() {
        Ok(v) => v,
        Err(err) => panic!("Invalid Server Port! {}", err.to_string())
    };
    println!("Starting Rubic Server at.({}:{})", &host, port);
    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", host.as_str()));
    let rock = rocket::custom(figment)
        .mount("/", routes![info]);
        //.launch().await?;
    Ok(())
}