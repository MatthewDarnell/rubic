use std::convert::TryInto;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use crate::entity::qubic_request;

#[derive(Debug)]
pub struct PeerSet(Vec<TcpStream>);

impl PeerSet {
    fn new() -> Self {
        PeerSet {
            0: vec![],
        }
    }
    pub fn num_peers(&self) -> usize {
        self.0.len()
    }
    pub fn add_peer(&mut self, ip: &str) -> Result<(), String> {
        match TcpStream::connect(ip) {
            Ok(stream) => Ok(self.0.push(stream)),
            Err(err) => Err(err.to_string())
        }
    }
    pub fn delete_peer(&mut self, ip: &str) -> bool {
        for (index, connection) in self.0.iter().enumerate() {
            match connection.peer_addr() {
                Ok(conn) => {
                    if Ok(conn) == ip.parse() {
                        self.0.remove(index);
                        return true;
                    }
                },
                Err(error) => {}
            }
        }
        false
    }
    pub fn send_request(&self, request: qubic_request) {

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
                assert!(err.contains("Connection refused"));
                assert_eq!(p_set.num_peers(), 0);
            }
        }
    }
}