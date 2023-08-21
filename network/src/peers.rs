use std::convert::TryInto;
use std::io::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use api::qubic_api_t;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::seq::SliceRandom;
use uuid::Uuid;


#[derive(Debug)]
pub struct Peer {
    stream: TcpStream,
    ping_time: u32,
    ip_addr: String,
    nick: String,
    whitelisted: bool,
    last_responded: SystemTime,
    id: String
}

impl Peer {
    fn new(ip: &str, stream: TcpStream ,nick: &str) -> Self {
        Peer {
            stream: stream,
            ping_time: 9999,
            ip_addr: ip.to_string(),
            nick: nick.to_string(),
            whitelisted: false,
            last_responded: SystemTime::now(),
            id: Uuid::new_v4().to_string()
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
        let sock: SocketAddr = ip.parse().unwrap();
        match TcpStream::connect_timeout(&sock, Duration::from_millis(5000)) {
            Ok(mut stream) => {
                println!("Adding Peer {}", ip);
                stream.set_read_timeout(Some(Duration::from_millis(5000))).unwrap();
                stream.set_write_timeout(Some(Duration::from_millis(5000))).unwrap();
                let mut request = qubic_api_t::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
                match self._send_request_via_stream(&mut stream, &mut request.as_bytes().to_vec()) {
                    Ok(_) => {
                        println!("peer responded!");
                        self.peers.push(Peer::new(ip, stream, ""));
                        Ok(())
                    }
                    Err(err) => {
                        println!("Error Getting Response From Peer! {}", err.as_str());
                        Err(err)
                    }
                }
            },
            Err(err) => Err(err.to_string())
        }
    }

    pub fn add_peer_do_not_send_request(&mut self, ip: &str) -> Result<(), String> {
        let sock: SocketAddr = ip.parse().unwrap();
        match TcpStream::connect_timeout(&sock, Duration::from_millis(5000)) {
            Ok(mut stream) => {
                println!("Adding Peer {}", ip);
                        println!("peer responded!");
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
    fn _listen_response(&mut self, id: &str) -> Result<Vec<u8>, String> {
        let mut peer: Option<&mut Peer> = None;
        for mut p in &mut self.peers.iter_mut() {
            if p.id.as_str() == id {
                peer = Some(p);
                break;
            }
        }
        if peer.is_none() {
            return Err("Invalid Peer Id".to_string())
        }
        println!("Reading From Peer <{}>", id);
        match peer.unwrap().read_stream() {
            Ok(result) => Ok(result),
            Err(err) => Err(err)
        }
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

    fn _send_request(&mut self, request: &Vec<u8>) -> Result<String, String> {
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
        println!("Writing Request To Stream: {:?}", request.as_slice());
        match peer.stream.write(request.as_slice()) {
            Ok(_) => {
                peer.last_responded = SystemTime::now();
                Ok(peer.id.to_owned())
            },
            Err(err) =>{
                println!("Failed To Send Data To Peer! {}", peer.ip_addr);
                Err(err.to_string())
            }
        }
    }
    pub fn make_request(&mut self, request: &mut qubic_api_t) -> Result<(), String> {
        match self._send_request(&mut request.as_bytes()) {
            Ok(peer_id) => {
                match self._listen_response(peer_id.as_str()) {
                    Ok(v) => Ok(request.set_response(v)),
                    Err(err) => Err(err)
                }
            },
            Err(err) => Err(err)
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