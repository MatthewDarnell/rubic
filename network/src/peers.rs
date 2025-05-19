use std::io::prelude::*;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::thread;
use api::QubicApiPacket;
use logger::{ debug, error };
use std::time::{Duration};
use store;

use crate::worker;
use crate::peer::Peer;


pub struct PeerSet {
  peers: Vec<Peer>,
  req_channel: (spmc::Sender<QubicApiPacket>, spmc::Receiver<QubicApiPacket>),
  threads: HashMap<String, std::thread::JoinHandle<()>>
}




impl PeerSet {
    pub fn new() -> Self {
        let _channel = std::sync::mpsc::channel::<QubicApiPacket>();
        let peer_set = PeerSet {
            peers: vec![],
            threads: HashMap::new(),
            req_channel: spmc::channel::<QubicApiPacket>(),
        };
        peer_set
    }
    pub fn get_peers(&self) -> Vec<&Peer> { self.peers.iter().map(|x| x).collect() }
    pub fn get_peer_ids(&self) -> Vec<String> { self.peers.iter().map(|x| x.get_id().to_owned()).collect() }
    pub fn num_peers(&self) -> usize {
        self.peers.len()
    }
    pub fn add_peer(&mut self, ip: &str) -> Result<(), String> {
        if let Ok(max_peers) = std::env::var("RUBIC_MAX_PEERS") {
            if self.get_peers().len() >= max_peers.parse::<usize>().unwrap() {
                return Err("Already At Max Capacity of Connected Peers".to_string());
            }
        } else {
            //Don't worry about max peers
        }
        for peer in &self.get_peers() {
            if peer.get_ip_addr() == ip {
                return Err("Duplicate Peer".to_string());
            }
        }
        let sock: SocketAddr = ip.parse().unwrap();
        match TcpStream::connect_timeout(&sock, Duration::from_millis(5000)) {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_millis(2500))).expect("set_write_timeout call failed");
                stream.set_write_timeout(Some(Duration::from_millis(2500))).expect("set_write_timeout call failed");
                stream.set_nodelay(true).expect("set_nodelay call failed");
                stream.set_ttl(100).expect("set_ttl call failed");
                let new_peer = Peer::new(ip, None, "");
                let id = new_peer.get_id().to_owned();
                {
                    let mut peer = new_peer.clone();
                    peer.set_stream(stream);
                    let rx = self.req_channel.1.clone();
                    let id = id.clone();
                    let copied_id = id.to_owned();
                    let t = std::thread::spawn(move || worker::handle_new_peer(id.to_owned(), peer, rx));
                    self.threads.insert(copied_id, t);
                }
                self.peers.push(new_peer);
                match store::sqlite::crud::peer::set_peer_connected(
                    store::get_db_path().as_str(),
                    id.as_str()
                ) {
                    Ok(_) => { debug(format!("Set Peer {} Connected.", id.as_str()).as_str()); },
                    Err(err) => { debug(format!("Error Setting Peer {} Connected! : {}", id.as_str(), err.as_str()).as_str()); }
                }
                Ok(())
            },
            Err(err) => {
                error!("Error Adding Peer! {}", err.to_string());
                Err(err.to_string())
            }
        }
    }

    pub fn delete_peer(&mut self, ip: &str) -> bool {
        for (index, connection) in self.peers.iter_mut().enumerate() {
            if let Some(stream) = &mut connection.get_stream() {
                match stream.peer_addr() {
                    Ok(conn) => {
                        if Ok(conn) == ip.parse() {
                            self.peers.remove(index);
                            return true;
                        }
                    },
                    Err(_) => {}
                }
            }
        }
        false
    }

    pub fn delete_peer_by_id(&mut self, id: &str) -> bool {
        //println!("Deleting Peer {}", id);
        for (index, connection) in self.peers.iter_mut().enumerate() {
            if connection.get_id().as_str() == id { //this is the peer, disconnect its stream
                if let Some(stream) = &mut connection.get_stream() {
                    match stream.shutdown(std::net::Shutdown::Both) {
                        Ok(_) => {},
                        Err(_) => {}
                    }
                }
                match store::sqlite::crud::peer::set_peer_disconnected(
                    store::get_db_path().as_str(),
                    id
                ) {
                    Ok(_) => {
                        //println!("Removed Peer {}", id);
                        self.peers.remove(index);
                        return true;
                    },
                    Err(err) => {
                        println!("Error Deleting Peer By Id.({}) : {}", id, err.as_str());
                    }
                }
            }
        }
        false
    }

    fn _send_request_via_stream(&mut self, stream: &mut TcpStream, request: &Vec<u8>) -> Result<(), String> {
        match stream.write(request.as_slice()) {
            Ok(_) => {
                let mut result: [u8; 256] = [0; 256];
                match stream.read(&mut result) {
                    Ok(bytes_read) => {
                        println!("Read {} bytes", bytes_read);
                        Ok(())
                    },
                    Err(e) => {
                        println!("{}", e.to_string());
                        Err(e.to_string())
                    }
                }
            },
            Err(err) =>{
                println!("Failed To Send Data To Peer!");
                Err(err.to_string())
            }
        }
    }

    pub fn make_request(&mut self, mut request: QubicApiPacket) -> Result<(), String> {
        if self.num_peers() < 1 {
            return Err("Cannot send request, 0 peers! Add some!".to_string())
        }
        let mut ids_to_delete: Vec<String> = vec![];
        for (_index, peer) in self.peers.iter().enumerate() {
            match store::sqlite::crud::peer::fetch_peer_by_id(store::get_db_path().as_str(), peer.get_id().as_str()) {
                Ok(p) => {
                    let connected = p.get("connected").unwrap() == "1";
                    if !connected {
                        ids_to_delete.push(peer.get_id().to_string());
                        continue;
                    }
                },
                Err(err) => {
                    println!("err {}", err);
                }
            }
            request.peer = Some(peer.get_id().to_owned());
            //thread::sleep(Duration::from_millis(50));
            match self.req_channel.0.send(request.clone()) {
                Ok(_) => { },
                Err(err) => {
                    println!("Failed To Send Request Data To Threads! : {}", err.to_string());
                }
            }
        }
        for id in ids_to_delete {
            self.delete_peer_by_id(id.as_str());
        }
        Ok(())
    }
}



#[cfg(test)]
pub mod peer_tests {
    use crate::peers::PeerSet;

    #[test]
    fn add_a_peer() {
        let mut p_set = PeerSet::new();
        match p_set.add_peer("127.0.0.1:8000") {
            Ok(_) => {
                assert_eq!(p_set.num_peers(), 1);
            },
            Err(err) => {
                assert!(err.contains("refused"));
                assert_eq!(p_set.num_peers(), 0);
            }
        }
    }
}