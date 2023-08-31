use std::convert::TryInto;
use std::io::prelude::*;
use std::thread;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use api::qubic_api_t;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::seq::SliceRandom;
use crate::worker;
use crate::receive_response_worker;
use uuid::Uuid;
use crate::peer::Peer;



#[derive(Debug)]
pub enum PeerStrategy {
    PRIORITIZE_LOW_PING = 0,
    PRIORITIZE_LONGEST_UNUSED_CONN = 1,
    RANDOM = 2
}

pub struct PeerSet {
  strategy: PeerStrategy,
  peers: Vec<Peer>,
  req_channel: (spmc::Sender<qubic_api_t>, spmc::Receiver<qubic_api_t>),
  resp_channel: std::sync::mpsc::Sender<qubic_api_t>,
  threads: HashMap<String, std::thread::JoinHandle<()>>
}




impl PeerSet {
    pub fn new(strategy: PeerStrategy) -> Self {
        let channel = std::sync::mpsc::channel::<qubic_api_t>();
        let peer_set = PeerSet {
            strategy: strategy,
            peers: vec![],
            threads: HashMap::new(),
            req_channel: spmc::channel::<qubic_api_t>(),
            resp_channel: channel.0
        };
        {
            let rx = channel.1;
            std::thread::spawn(move || receive_response_worker::listen_for_api_responses(rx));

        }
        peer_set
    }
    pub fn num_peers(&self) -> usize {
        self.peers.len()
    }
    pub fn add_peer(&mut self, ip: &str) -> Result<(), String> {
        let sock: SocketAddr = ip.parse().unwrap();
        match TcpStream::connect_timeout(&sock, Duration::from_millis(5000)) {
            Ok(mut stream) => {
                println!("Adding Peer {}", ip);
                let new_peer = Peer::new(ip, None, "");
                let id = new_peer.get_id().to_owned();
                {
                    let mut peer = new_peer.clone();
                    peer.set_stream(stream);
                    let rx = self.req_channel.1.clone();
                    let tx = self.resp_channel.clone();
                    let id = id;
                    let copied_id = id.to_owned();
                    let t = std::thread::spawn(move || worker::handle_new_peer(id.to_owned(), peer, rx, tx));
                    self.threads.insert(copied_id, t);
                }
                self.peers.push(new_peer);
                Ok(())
            },
            Err(err) => Err(err.to_string())
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
                    Err(error) => {}
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

    pub fn make_request(&mut self, mut request: qubic_api_t) -> Result<(), String> {
        if self.num_peers() < 1 {
            return Err("Cannot send request, 0 peers! Add some!".to_string())
        }
        let peer: &mut Peer = match self.strategy {
            PeerStrategy::PRIORITIZE_LOW_PING => {
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
            PeerStrategy::PRIORITIZE_LONGEST_UNUSED_CONN => {
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
        //println!("Using Peer: {}", peer.get_ip_addr().as_str());
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