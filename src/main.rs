#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use rocket::{get, routes};
#[macro_use]
extern crate dotenv_codegen;
use network::peers::{PeerStrategy, PeerSet};
use store::sqlite::crud;
use store::get_db_path;
use std::thread;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
mod env;
mod routes;





/*

  TODO:
    Create Worker Threads and queue:
      1 for Receiving Requests from Server Routes
      1 for PeerSet Work
*/

#[rocket::main]
async fn main() {
  let path = store::get_db_path();
  crud::Peer::set_all_peers_disconnected(path.as_str());
  let peer_ips = vec!["85.10.199.154:21841",
                      "148.251.184.163:21841",
                      "62.2.98.75:21841",
                      "193.135.9.63:21841",
                      "144.2.106.163:21841"];
  println!("Creating Peer Set");

  let mut peer_set = PeerSet::new(PeerStrategy::RANDOM);
  for ip in peer_ips {
    println!("Adding Peer {}", ip);
    peer_set.add_peer(ip);
    println!("Peer Added");
  }

  let (tx, rx) = mpsc::channel::<std::collections::HashMap<String, String>>();
  let (tx2, rx_server_route_responses_from_thread) = spmc::channel::<std::collections::HashMap<String, String>>();

  {
    let mut tx = tx2;
    let rx = rx;  //Move rx into scope and then thread
    let t = std::thread::spawn(move || {
      let delay = Duration::from_millis(500);
      loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
          Ok(map) => {
            println!("Received New Map: {:?}", map);
            if let Some(method) = map.get(&"method".to_string()) {
              if method == &"add_peer".to_string() {
                println!("Adding Peer!");
                let peer_ip = map.get(&"peer_ip".to_string()).unwrap();
                //todo: validate peer_ip
                let message_id = map.get(&"message_id".to_string()).unwrap();
                let mut response: HashMap<String, String> = HashMap::new();
                match peer_set.add_peer(peer_ip) {
                  Ok(_) => {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), "200".to_string());
                    println!("Sending {:?}", &response);
                    tx.send(response);
                  },
                  Err(err) => {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), err);
                    println!("Sending {:?}", &response);
                    tx.send(response);
                  }
                }
              }
            }
          },
          Err(err) => {
            //No Error, just timed out due to no web requests
            //println!("Read TimeOut Error: {}", err.to_string());
          }
        }


        let mut request = api::qubic_api_t::get_identity_balance("BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID");
        match peer_set.make_request(request) {
          Ok(_) => {},
          //Ok(_) => println!("{:?}", request.response_data),
          Err(err) => println!("{}", err)
        }

        std::thread::sleep(delay);
      }
    });
  }

/*
  let t = std::thread::spawn(move || {
    let delay = Duration::from_secs(3);
    loop {
      let mut request = api::qubic_api_t::get_identity_balance("BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID");
      match peer_set.make_request(request) {
        Ok(_) => {},
        //Ok(_) => println!("{:?}", request.response_data),
        Err(err) => println!("{}", err)
      }
      std::thread::sleep(delay);
    }
  });
*/
  let host = env::get_host();
  let port: u32 = match env::get_port().parse() {
    Ok(v) => v,
    Err(err) => panic!("Invalid Server Port! {}", err.to_string())
  };
  println!("Starting Rubic Server at.({}:{})", &host, port);
  let figment = rocket::Config::figment()
      .merge(("port", port))
      .merge(("address", host.as_str()));
  let rock = rocket::custom(figment)
      .mount("/", routes![
        routes::info::info,
        routes::info::balance,
        routes::info::add_peer
      ])
      .manage(std::sync::Mutex::new(tx))
      .manage(std::sync::Mutex::new(rx_server_route_responses_from_thread))
      .launch().await;
  //t.join();
}


