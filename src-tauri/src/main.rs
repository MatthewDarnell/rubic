use std::collections::HashMap;
use rocket::routes;

extern crate dotenv_codegen;
use std::sync::mpsc;

use rubic::logger::{setup_logger, info};
use rubic::store::{sqlite};
use rubic::peer_loop::start_peer_set_thread;
use rubic::env;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};



use tauri::Manager;

#[rocket::main]
async fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = rt.enter();
    let api_server_handle = rt.spawn(async move {
        run_api_server().await;
    });
    fn setup<'a>(_app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
        let window = _app.get_webview_window("main").unwrap();
        window.open_devtools();
        window.close_devtools();
        // This one
       // let handle = app.handle();

        tauri::async_runtime::spawn(async move { // also added move here
            println!("Local Server is running");
        });
        Ok(())
    }
    tauri::Builder::default()
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    rt.block_on(async {
        let _ = api_server_handle.await;
    });
    async fn run_api_server() {
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
        info(format!("Starting Rubic v{} - A Qubic Wallet", version).as_str());
        println!("{}", qubic_ascii_art_logo);
        println!("Starting Rubic v{} - A Qubic Wallet", version);
        println!("Warning! This software comes with no warranty, real or implied. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.");
        info("Warning! This software comes with no warranty, real or implied. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.");

        let path = rubic::store::get_db_path();
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

        rubic::routes::asset::all_asset_balances,
        rubic::routes::asset::balance,
        rubic::routes::asset::fetch_transfers,
        rubic::routes::asset::get_assets,
        rubic::routes::asset::transfer,

        rubic::routes::identity::balance,
        rubic::routes::identity::add_identity,
        rubic::routes::identity::add_identity_with_password,
        rubic::routes::identity::create_random_identity,
        rubic::routes::identity::delete_identity,
        rubic::routes::identity::get_identities,
        rubic::routes::identity::get_identity_from_seed,

        rubic::routes::info::info,
        rubic::routes::info::latest_tick,

        rubic::routes::peer::peers,
        rubic::routes::peer::add_peer,
        rubic::routes::peer::delete_peer,
        rubic::routes::peer::get_peer_limit,
        rubic::routes::peer::set_peer_limit,

        rubic::routes::qx::fetch_orders,
        rubic::routes::qx::get_orderbook,
        rubic::routes::qx::place_order,

        rubic::routes::transaction::fetch_transfers,
        rubic::routes::transaction::transfer,

        rubic::routes::wallet::is_wallet_encrypted,
        rubic::routes::wallet::encrypt_wallet,
        rubic::routes::wallet::set_master_password,
        rubic::routes::wallet::download_wallet,
        rubic::routes::wallet::is_unlocked,
        rubic::routes::wallet::unlock
      ])
            .manage(std::sync::Mutex::new(tx))
            .manage(std::sync::Mutex::new(rx_server_route_responses_from_thread))
            .attach(CORS)
            .launch().await.expect("Failed To Create Server");
    }
}


