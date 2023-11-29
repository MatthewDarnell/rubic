#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use rocket::routes;

extern crate dotenv_codegen;
use network::peers::{PeerStrategy, PeerSet};
use logger::{info, debug, error};
use store::sqlite::crud;
use store::get_db_path;
use identity;
use api;
use std::sync::mpsc;
use std::time::Duration;
mod env;
mod routes;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};


#[rocket::main]
async fn main() {
  info("Starting Rubic Application");
  let path = store::get_db_path();
  crud::peer::set_all_peers_disconnected(path.as_str()).unwrap();
  let peer_ips = vec!["144.76.107.105:21841",
   // "85.10.199.154:21841",
                      "148.251.184.163:21841",
                      "62.2.98.75:21841",
     //                 "193.135.9.63:21841",
     //                 "144.2.106.163:21841"
  ];
  debug("Creating Peer Set");

  let mut peer_set = PeerSet::new(PeerStrategy::RANDOM);
  for ip in peer_ips {
    debug(format!("Adding Peer {}", ip).as_str());
    peer_set.add_peer(ip).unwrap();
    debug("Peer Added");
  }

  let (tx, rx) = mpsc::channel::<std::collections::HashMap<String, String>>();
  let (tx2, rx_server_route_responses_from_thread) = spmc::channel::<std::collections::HashMap<String, String>>();

  {
    let mut tx = tx2;
    let rx = rx;  //Move rx into scope and then thread
    std::thread::spawn(move || {
      let delay = Duration::from_millis(500);

      //Main Thread Loop
      loop {

        //Try To Receive Messages From Server Api
        match rx.recv_timeout(Duration::from_secs(5)) {
          Ok(map) => {
            if let Some(method) = map.get(&"method".to_string()) {
              if method == &"add_peer".to_string() {
                let peer_ip = map.get(&"peer_ip".to_string()).unwrap();
                //todo: validate peer_ip
                let message_id = map.get(&"message_id".to_string()).unwrap();
                let mut response: HashMap<String, String> = HashMap::new();
                match peer_set.add_peer(peer_ip) {
                  Ok(_) => {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), "Peer Added".to_string());
                    tx.send(response).unwrap();
                  },
                  Err(err) => {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), err.as_str().to_string());
                    error(format!("Failed To Add Peer.({}) = ({})", peer_ip.as_str(), err.as_str()).as_str());
                    println!("Sending {:?}", &response);
                    tx.send(response).unwrap();
                  }
                }
              }
              else if method == &"transfer".to_string() {
                //    let mut map: HashMap<String, String> = HashMap::new();
                //     map.insert("method".to_string(), "transfer".to_string());
                //     map.insert("source".to_string(), source_identity);
                //     map.insert("dest".to_string(), dest_identity);
                //     map.insert("amount".to_string(), string_amount);
                //     map.insert("expiration".to_string(), string_expiration);
                //
                let message_id = map.get(&"message_id".to_string()).unwrap();
                let mut response: HashMap<String, String> = HashMap::new();

                let source = map.get(&"source".to_string()).unwrap();
                let dest = map.get(&"dest".to_string()).unwrap();
                let amount = map.get(&"amount".to_string()).unwrap();
                let expiration = map.get(&"expiration".to_string()).unwrap();
                ////

                let mut id: identity::Identity = match store::sqlite::crud::fetch_identity(get_db_path().as_str(), source.as_str()) {
                  Ok(identity) => identity,
                  Err(err) => {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), "Unknown Source Identity!".to_string());
                    error(format!("Failed To Make Transfer, Unknown Identity {}", source.as_str()).as_str());
                    tx.send(response).unwrap();
                    continue;
                  }
                };

                //println!("Transfer: Fetched Identity: {:?}", &id);
                if(id.encrypted) {
                  if let Some(pass) = map.get(&"password".to_string()) {
                    id = match crud::master_password::get_master_password(get_db_path().as_str()) {
                      Ok(master_password) => {
                        //println!("{} : {:?}", pass.as_str(), &master_password);
                        match crypto::passwords::verify_password(pass.as_str(), master_password[1].as_str()) {
                          Ok(verified) => {
                            if !verified {
                              response.insert("message_id".to_string(), message_id.to_string());
                              response.insert("status".to_string(), "Invalid Password!".to_string());
                              error("Failed To Create Transfer; Invalid Password");
                              tx.send(response).unwrap();
                              continue;
                            } else {
                              match id.decrypt_identity(pass.as_str()) {
                                Ok(identity) => identity,
                                Err(err) => {
                                  response.insert("message_id".to_string(), message_id.to_string());
                                  response.insert("status".to_string(), "Invalid Password For This Identity!".to_string());
                                  error("Failed To Create Transfer; Invalid Password For This Identity");
                                  tx.send(response).unwrap();
                                  continue;
                                }
                              }
                            }
                          },
                          Err(err) => {
                            response.insert("message_id".to_string(), message_id.to_string());
                            response.insert("status".to_string(), "Failed To Verify Master Password Vs Supplied Password!".to_string());
                            error("Failed To Verify Master Password Vs Supplied Password");
                            tx.send(response).unwrap();
                            continue;
                          }
                        }
                      },
                      Err(err) => {
                        response.insert("message_id".to_string(), message_id.to_string());
                        response.insert("status".to_string(), "Identity Is Encrypted, Yet No Master Password Set! Weird!".to_string());
                        error("Identity Is Encrypted, Yet No Master Password Set! Weird");
                        tx.send(response).unwrap();
                        continue;
                      }
                    };
                  } else {
                    response.insert("message_id".to_string(), message_id.to_string());
                    response.insert("status".to_string(), "Must Enter A Password!".to_string());
                    error("Failed To Decrypt Password For Transfer; No Password Supplied");
                    tx.send(response).unwrap();
                    continue;
                  }
                } else {
                  println!("is Not encrypted!");
                }
println!("About To Make Tx!");
                let amt: u64 = amount.parse().unwrap();
                let tck: u32 = expiration.parse().unwrap();

                info(format!("Creating Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &id.identity.as_str(), amt.to_string().as_str(), dest.as_str(), tck.to_string().as_str()).as_str());
                println!("Creating Transfer: {} .({}) ---> {} (Expires At Tick.<{}>)", &id.identity.as_str(), amt.to_string().as_str(), dest.as_str(), tck.to_string().as_str());
                let transfer_tx = api::transfer::TransferTransaction::from_vars(&id, &dest, amt, tck);
                println!("{:?}", &transfer_tx);
                println!("{:?}", &transfer_tx.as_bytes());
                response.insert("message_id".to_string(), message_id.to_string());
                response.insert("status".to_string(), "200".to_string());

                let request = api::QubicApiPacket::broadcast_transaction(&transfer_tx);
                match peer_set.make_request(request) {
                  Ok(_) => { info("Transaction Sent!"); println!("Transaction Sent!"); },
                  //Ok(_) => println!("{:?}", request.response_data),
                  Err(err) => println!("{}", err)
                }
                std::thread::sleep(delay);

                tx.send(response).unwrap();
                continue;



                ////
              }
              else if method == &"add_identity".to_string() {
                let seed = map.get(&"seed".to_string()).unwrap();
                let mut id: identity::Identity = identity::Identity::new(seed.as_str());
                let message_id = map.get(&"message_id".to_string()).unwrap();
                let mut response: HashMap<String, String> = HashMap::new();
                if let Some(pass) = map.get(&"password".to_string()) {
                  match crud::master_password::get_master_password(get_db_path().as_str()) {
                    Ok(master_password) => {
                      match crypto::passwords::verify_password(pass.as_str(), master_password[1].as_str()) {
                        Ok(verified) => {
                          if !verified {
                            response.insert("message_id".to_string(), message_id.to_string());
                            response.insert("status".to_string(), "Invalid Password!".to_string());
                          } else {
                            id = id.encrypt_identity(pass.as_str()).unwrap();
                            match crud::insert_new_identity(get_db_path().as_str(), &id) {
                              Ok(v) => {
                                response.insert("message_id".to_string(), message_id.to_string());
                                response.insert("status".to_string(), "200".to_string());
                              },
                              Err(err) => {
                                response.insert("message_id".to_string(), message_id.to_string());
                                response.insert("status".to_string(), err.to_string());
                                error(format!("Failed To Insert! {:?}", err).as_str());
                              }
                            }
                          }
                        },
                        Err(err) => {
                          response.insert("message_id".to_string(), message_id.to_string());
                          response.insert("status".to_string(), err.to_string());
                          error("Failed To Verify Master Password!");
                        }
                      }
                    },
                    Err(err) => {
                      response.insert("message_id".to_string(), message_id.to_string());
                      response.insert("status".to_string(), "No Master Password Set!".to_string());
                    }
                  }
                } else {
                  match crud::insert_new_identity(get_db_path().as_str(), &id) {
                    Ok(v) => {
                      response.insert("message_id".to_string(), message_id.to_string());
                      response.insert("status".to_string(), "200".to_string());
                    },
                    Err(err) => {
                      response.insert("message_id".to_string(), message_id.to_string());
                      response.insert("status".to_string(), err.to_string());
                      error(format!("Failed To Insert! {:?}", err).as_str());
                    }
                  }
                }
                tx.send(response).unwrap();
              }
            }
          },
          Err(_) => {
            //No Error, just timed out due to no web requests
            //println!("Read TimeOut Error: {}", err.to_string());
          }
        }

        //Update Balances For All Stored Identities
        match crud::fetch_all_identities(get_db_path().as_str()) {
          Ok(identities) => {
            for identity in identities {
              let request = api::QubicApiPacket::get_identity_balance(identity.as_str());
              match peer_set.make_request(request) {
                Ok(_) => {},
                //Ok(_) => println!("{:?}", request.response_data),
                Err(err) => println!("{}", err)
              }
              std::thread::sleep(delay);
            }
          },
          Err(err) => {
            error(format!("Error: {:?}", err).as_str());
          }
        }

        //Check if Any Peers Have Been Set As 'Disconnected'. This means The TcpStream Connection Terminated. Delete them.
        for peer in peer_set.get_peer_ids() {
          match crud::peer::fetch_peer_by_id(get_db_path().as_str(), peer.as_str()) {
            Ok(temp_peer) => {
              if let Some(connected) = temp_peer.get(&"connected".to_string()) {
                if connected.as_str() != "1" {
                  //println!("{:?}", &temp_peer);
                  error(format!("Is Peer {} connected? {}", peer.as_str(), &connected).as_str());
                  peer_set.delete_peer_by_id(peer.as_str());
                }
              }
            },
            Err(err) => panic!("{}", err)
          }
        }


        //Try To Spin up New Peers Until We Reach The Min Number
        let min_peers: usize = env::get_min_peers();
        let num_peers: usize = peer_set.get_peers().len();
        if num_peers < min_peers {
          debug(format!("Number Of Peers.({}) Less Than Min Peers.({}). Adding More...", num_peers, min_peers).as_str());
          match crud::peer::fetch_disconnected_peers(get_db_path().as_str()) {
            Ok(disconnected_peers) => {
              debug(format!("Fetched {} Disconnected Peers", disconnected_peers.len()).as_str());
              for p in disconnected_peers {
                let peer_id = &p[0];
                let peer_ip = &p[1];
                match peer_set.add_peer(peer_ip.as_str()) {
                  Ok(_) => {
                    debug(format!("Peer.({}) Added {}", peer_ip.as_str(), peer_id.as_str()).as_str());
                  },
                  Err(err) => {
                    debug(format!("Failed To Add Peer.({}) : ({:?})", peer_ip.as_str(), err).as_str());
                  }
                }
              }
            },
            Err(_) => println!("Db Error Fetching Disconnected Peers")
          }
        }

        std::thread::sleep(delay);
      }
    });
  }

/*
  let t = std::thread::spawn(move || {
    let delay = Duration::from_secs(3);
    loop {
      let mut request = api::QubicApiPacket::get_identity_balance("BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID");
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
  info(format!("Starting Rubic Server at.({}:{})", &host, port).as_str());



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
        routes::info::latest_tick,
        routes::info::is_wallet_encrypted,
        routes::info::encrypt_wallet,
        routes::info::set_master_password,
        routes::info::info,
        routes::info::peers,
        routes::info::download_wallet,
        routes::info::balance,
        routes::info::add_peer,
        routes::info::add_identity,
        routes::info::create_random_identity,
        routes::info::add_identity_with_password,
        routes::info::get_identities,
        routes::info::get_identity_from_seed,
        routes::info::transfer
      ])
      .manage(std::sync::Mutex::new(tx))
      .manage(std::sync::Mutex::new(rx_server_route_responses_from_thread))
      .attach(CORS)
      .launch().await;
  //t.join();
}


