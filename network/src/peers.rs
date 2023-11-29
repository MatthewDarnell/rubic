use std::io::prelude::*;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use api::QubicApiPacket;
use logger::debug;
use std::time::{Duration, SystemTime};
use store;
use rand::seq::SliceRandom;
use crate::worker;
use crate::receive_response_worker;
use crate::peer::Peer;




#[derive(Debug)]
pub enum PeerStrategy {
    PrioritizeLowPing = 0,
    PrioritizeLongestUnusedConn = 1,
    RANDOM = 2
}

pub struct PeerSet {
  strategy: PeerStrategy,
  peers: Vec<Peer>,
  req_channel: (spmc::Sender<QubicApiPacket>, spmc::Receiver<QubicApiPacket>),
  resp_channel: std::sync::mpsc::Sender<QubicApiPacket>,
  threads: HashMap<String, std::thread::JoinHandle<()>>
}




impl PeerSet {
    pub fn new(strategy: PeerStrategy) -> Self {
        let channel = std::sync::mpsc::channel::<QubicApiPacket>();
        let peer_set = PeerSet {
            strategy: strategy,
            peers: vec![],
            threads: HashMap::new(),
            req_channel: spmc::channel::<QubicApiPacket>(),
            resp_channel: channel.0,
        };
        {
            let rx = channel.1;
            std::thread::spawn(move || receive_response_worker::listen_for_api_responses(rx));

        }
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
                let new_peer = Peer::new(ip, None, "");
                let id = new_peer.get_id().to_owned();
                {
                    let mut peer = new_peer.clone();
                    peer.set_stream(stream);
                    let rx = self.req_channel.1.clone();
                    let tx = self.resp_channel.clone();
                    let id = id.clone();
                    let copied_id = id.to_owned();
                    let t = std::thread::spawn(move || worker::handle_new_peer(id.to_owned(), peer, rx, tx));
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
                println!("Error Adding Peer! {}", err.to_string());
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
        println!("Deleting Peer {}", id);
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
                        println!("Removed Peer {}", id);
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
        let peer: &mut Peer = match self.strategy {
            PeerStrategy::PrioritizeLowPing => {
                let mut temp = &self.peers[0];
                let mut temp_index = 0;
                for (index, peer) in self.peers.iter().enumerate() {
                    if index > 0 && peer.get_ping_time() < temp.get_ping_time() {
                        temp = &self.peers[index];
                        temp_index = index;
                    }
                }
                &mut self.peers[temp_index]
            },
            PeerStrategy::PrioritizeLongestUnusedConn => {
                let mut temp = &self.peers[0];
                let mut temp_index = 0;
                for (index, peer) in self.peers.iter().enumerate() {
                    if index > 0 &&
                        (SystemTime::now().duration_since(peer.get_last_responded()).expect("Failed To Get System Time! Memory Corruption?") >
                        SystemTime::now().duration_since(temp.get_last_responded()).expect("Failed To Get System Time! Memory Corruption?"))
                    {
                        temp_index = index;
                        temp = &self.peers[index];
                    }
                }
                &mut self.peers[temp_index]
            },
            PeerStrategy::RANDOM => {
                match self.peers.choose_mut(&mut rand::thread_rng()) {
                    Some(v) => v,
                    None => return Err("Failed To Find Random Peer From Set! Strange!".to_string())
                }
            }
        };
        println!("Using Peer: {}", peer.get_ip_addr().as_str());
        request.peer = Some(peer.get_id().to_owned());
        match self.req_channel.0.send(request) {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("Failed To Send Request Data To Threads! : {}", err.to_string());
                Err(err.to_string())
            }
        }
    }
}



#[cfg(test)]
pub mod peer_tests {
    use crate::peers::PeerSet;
    use crate::peers::PeerStrategy::RANDOM;

    #[test]
    fn add_a_peer() {
        let mut p_set = PeerSet::new(RANDOM);
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