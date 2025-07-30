use std::collections::HashMap;
use rocket::routes;

extern crate dotenv_codegen;
use logger::{setup_logger, info};
use std::sync::mpsc;
mod env;
mod routes;
mod peer_loop;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use store::sqlite;
use crate::peer_loop::start_peer_set_thread;

#[rocket::main]
async fn main() {

    let qubic_ascii_art_logo: &str = "
   ....................
   .....          .....
   ...    || |||    ...
   ..     || |||     ..
   ..        |||    ...
   ...       |||   ....
   ......        ......
   ....................
     ";

  let version: String = env!("CARGO_PKG_VERSION").to_string();
    
  match setup_logger() {
      Ok(_) => {},
      Err(error) => {
          eprintln!("Failed To Set Up Logging!: {}", error);
      }
  }
  logger::info(format!("Starting Rubic v{} - A Qubic Wallet", version).as_str());
  println!("{}", qubic_ascii_art_logo);
  println!("Starting Rubic v{} - A Qubic Wallet", version);
  println!("Warning! This software comes with no warranty, real or implied. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.");
  info("Warning! This software comes with no warranty, real or implied. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.");
  
  crypto::initialize();  
    
  let path = store::get_db_path();
  match sqlite::create::open_database(path.as_str(), true) {
      Ok(_) => info!("Database successfully opened"),
      Err(error) => {
          logger::error(format!("Database Failed To Open/Create: {}", error).as_str());
          panic!("Failed To Open Database!");
      }
  }  
    
    
    
  sqlite::peer::set_all_peers_disconnected(path.as_str()).unwrap();

    
  let (tx, rx) = mpsc::channel::<std::collections::HashMap<String, String>>();
  let (_, rx_server_route_responses_from_thread) = spmc::channel::<std::collections::HashMap<String, String>>();

  let (tx_incoming_api_request, rx_incoming_api_request) = mpsc::channel::<HashMap<String, String>>();
    start_peer_set_thread(&tx, rx_incoming_api_request);

  {
    let tx = tx_incoming_api_request;
    let rx = rx;
    std::thread::spawn(move || {
      loop {
        match rx.recv() {
        //match rx.recv_timeout(Duration::from_secs(5)) {
          Ok(map) => {
            match tx.send(map) {
                Ok(_) => {
                    println!("Received API Route Request, Forwarding to Peer Set Thread.");
                },
                Err(_) => {
                    println!("Error Passing API Route Message To Peer Set Thread!");
                }
            }
          },
          Err(_) => {/* Got Nothing :( */}
        }
      }
    });
  }





  let host = env::get_host();
  let port: u32 = match env::get_port().parse() {
    Ok(v) => v,
    Err(err) => panic!("Invalid Server Port! {}", err.to_string())
  };
  info!("Starting Rubic Server at.({}:{})", &host, port);

  pub struct CORS;

  #[rocket::async_trait]
  impl Fairing for CORS {
    fn info(&self) -> Info {
      Info {
        name: "Add CORS headers to responses",
        kind: Kind::Response
      }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
      response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
      response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
      response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
      response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
  }


  let figment = rocket::Config::figment()
      .merge(("log_level", "critical"))
      .merge(("port", port))
      .merge(("address", host.as_str()));
  rocket::custom(figment)
      .mount("/", routes![
        
        routes::asset::all_asset_balances,
        routes::asset::balance,
        routes::asset::fetch_transfers,
        routes::asset::get_assets,
        routes::asset::transfer,

        routes::identity::balance,
        routes::identity::add_identity,
        routes::identity::add_identity_with_password,
        routes::identity::create_random_identity,
        routes::identity::delete_identity,
        routes::identity::get_identities,
        routes::identity::get_identity_from_seed,

        routes::info::info,
        routes::info::latest_tick,

        routes::peer::peers,
        routes::peer::add_peer,
        routes::peer::delete_peer,
        routes::peer::get_peer_limit,
        routes::peer::set_peer_limit,
          
        routes::qx::fetch_orders,  
        routes::qx::get_orderbook,  
        routes::qx::place_order,  

        routes::transaction::fetch_transfers,
        routes::transaction::transfer,

        routes::wallet::is_wallet_encrypted,
        routes::wallet::encrypt_wallet,
        routes::wallet::set_master_password,
        routes::wallet::download_wallet
      ])
      .manage(std::sync::Mutex::new(tx))
      .manage(std::sync::Mutex::new(rx_server_route_responses_from_thread))
      .attach(CORS)
      .launch().await.expect("Failed To Create Server");
}


