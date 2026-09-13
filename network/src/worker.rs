use std::collections::HashMap;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::peer::Peer;
use api::request::QubicApiPacket;
use store::get_db_path;
use store::sqlite::peer::set_peer_disconnected;
use crate::tcp_recv::qubic_tcp_receive_data;

pub fn handle_new_peer(_id: String, request_matcher: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, peer: Peer, rx: spmc::Receiver<QubicApiPacket>) {
    if peer.get_stream().is_none() {
       println!("Peer {} Missing TcpStream! Shutting Down Worker Thread.", peer.get_id());
        return;
    }
    let mut stream = peer.get_stream().unwrap();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        // The PeerSet dropped this peer (or the flag was cleared elsewhere): stop consuming work.
        if !peer.is_connected() {
            break;
        }
        //Block until we receive work
        match rx.clone().recv() {
            Ok(mut request) => {
                match request_matcher.lock() {
                    Ok(mut matcher) => {
                        matcher.insert(request.header._dejavu, request.clone());
                    },
                    Err(_) => {}
                }
                match stream.write(request.as_bytes().as_slice()) {
                    Ok(_) => {
                        stream.flush().ok();
                        if !qubic_tcp_receive_data(&peer, request_matcher.clone(), stream) {
                            mark_disconnected(&peer);
                            break;
                        }
                    },
                    Err(_err) => {   //Probably the peer closed the tcp connection
                        mark_disconnected(&peer);
                        break;
                    }
                }
            },
            Err(err) => {
                println!("Failed To Receive Work In Thread! {}", err.to_string());
                break;
            }
        }
    }
   // println!("Worker Peer Thread Exiting!");
}

fn mark_disconnected(peer: &Peer) {
    peer.set_connected(false);
    match set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()) {
        Ok(_) => {},
        Err(err) => {
            println!("Failed To Set Peer {} disconnected: {}", peer.get_id().as_str(), err);
        }
    }
}
