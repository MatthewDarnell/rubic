use std::convert::TryInto;
use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use crate::entity::qubic_request;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::seq::SliceRandom;


#[derive(Debug)]
pub struct Peer {
    stream: TcpStream,
    ping_time: u32,
    ip_addr: String,
    nick: String,
    whitelisted: bool,
    last_responded: SystemTime
}

impl Peer {
    fn new(ip: &str, stream: TcpStream ,nick: &str) -> Self {
        Peer {
            stream: stream,
            ping_time: 9999,
            ip_addr: ip.to_string(),
            nick: nick.to_string(),
            whitelisted: false,
            last_responded: SystemTime::now()
        }
    }
    pub fn read_stream(&mut self) -> Result<Vec<u8>, String>{
        let mut result: [u8; 256] = [0; 256];
        match self.stream.read(&mut result) {
            Ok(bytes_read) => {
                println!("Read {} bytes", bytes_read);
                Ok(result.to_vec())
            },
            Err(e) => {
                println!("{}", e.to_string());
                Err(e.to_string())
            }
        }
    }
    pub fn set_ping_time(&mut self, ping: u32) {
        self.ping_time = ping;
    }
    pub fn set_nick(&mut self, nick: &str) {
        self.nick = nick.to_string();
    }
    pub fn set_whitelisted(&mut self) {
        self.whitelisted = true;
    }
    pub fn remove_whitelist(&mut self) {
        self.whitelisted = false;
    }
}

#[derive(Debug)]
pub enum PeerStrategy {
    PRIORITIZE_LOW_PING = 0,
    PRIORITIZE_LONGEST_UNUSED_CONN = 1,
    RANDOM = 2
}

#[derive(Debug)]
pub struct PeerSet {
  strategy: PeerStrategy,
  peers: Vec<Peer>
}

impl PeerSet {
    pub fn new(strategy: PeerStrategy) -> Self {
        PeerSet {
            strategy: strategy,
            peers: vec![],
        }
    }
    pub fn num_peers(&self) -> usize {
        self.peers.len()
    }
    pub fn add_peer(&mut self, ip: &str) -> Result<(), String> {
        match TcpStream::connect(ip) {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
                self.peers.push(Peer::new(ip, stream, ""));
                Ok(())
            },
            Err(err) => Err(err.to_string())
        }
    }
    pub fn delete_peer(&mut self, ip: &str) -> bool {
        for (index, connection) in self.peers.iter().enumerate() {
            match connection.stream.peer_addr() {
                Ok(conn) => {
                    if Ok(conn) == ip.parse() {
                        self.peers.remove(index);
                        return true;
                    }
                },
                Err(error) => {}
            }
        }
        false
    }
    pub fn send_request(&mut self, request: &Vec<u8>) -> Result<&mut Peer, String> {
        if self.num_peers() < 1 {
            return Err("Cannot send request, 0 peers! Add some!".to_string())
        }
        let peer: &mut Peer = match self.strategy {
            PeerStrategy::PRIORITIZE_LOW_PING => {
                let mut temp = &self.peers[0];
                let mut temp_index = 0;
                for (index, peer) in self.peers.iter().enumerate() {
                    if index > 0 && peer.ping_time < temp.ping_time {
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
                        (SystemTime::now().duration_since(peer.last_responded).expect("Failed To Get System Time! Memory Corruption?") >
                        SystemTime::now().duration_since(temp.last_responded).expect("Failed To Get System Time! Memory Corruption?"))
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
        println!("Using Peer: {}", peer.ip_addr.as_str());
        match peer.stream.write(request.as_slice()) {
            Ok(_) => {
                peer.last_responded = SystemTime::now();
                Ok(peer)
            },
            Err(err) =>{
                println!("Failed To Send Data To Peer! {}", peer.ip_addr);
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
                assert!(err.contains("Connection refused"));
                assert_eq!(p_set.num_peers(), 0);
            }
        }
    }
}