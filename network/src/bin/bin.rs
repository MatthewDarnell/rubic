use std::time::Duration;
use std::thread::sleep;
use std::io::prelude::*;
use std::net::TcpStream;
extern crate api;
extern crate network;
extern crate crypto;
use api::qubic_api_t;
use network::peers::{PeerStrategy, PeerSet};

fn main() {
    let peer_ips = vec!["85.10.199.154:21841", "148.251.184.163:21841"];
    println!("Creating Peer Set");
    let mut peer_set = PeerSet::new(PeerStrategy::RANDOM);
    for ip in peer_ips {
        println!("Adding Peer {}", ip);
        peer_set.add_peer(ip);
        println!("Peer Added");
    }
    println!("Number Of Peers: {}", peer_set.num_peers());
    let delay = Duration::from_secs(3);


    loop {
        println!("Writing");
        let mut request = qubic_api_t::get_identity_balance("BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARMID");
        match peer_set.make_request(request) {
            Ok(_) => println!("Request Completed."),
            //Ok(_) => println!("{:?}", request.response_data),
            Err(err) => println!("{}", err)
        }
        sleep(delay);
    }

}