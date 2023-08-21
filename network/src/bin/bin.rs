use std::time::Duration;
use std::io::prelude::*;
use std::net::TcpStream;
extern crate network;
extern crate crypto;
use network::entity::qubic_request;
use network::peers::{PeerStrategy, Peer, PeerSet};

fn main() {
    let peer_ips = vec!["136.243.81.157:21841", "5.199.134.150:21841", "91.43.76.142:21841"];
    println!("Creating Peer Set");
    let mut peer_set = PeerSet::new(PeerStrategy::PRIORITIZE_LONGEST_UNUSED_CONN);
    for ip in peer_ips {
        peer_set.add_peer(ip);
    }
    println!("Number Of Peers: {}", peer_set.num_peers());
    loop {
        println!("Writing");
        let request = qubic_request::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON").as_bytes();
        let mut peer = peer_set.send_request(&request).unwrap();
        println!("Wrote");
        println!("Reading");
        match peer.read_stream() {
            Ok(result) => {
                println!("{:?}", result.as_slice());
            },
            Err(err) => {
                println!("Failed to Read!");
            }
        }
    }

}