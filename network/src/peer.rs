use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::io::prelude::*;
use std::thread;
use chrono::prelude::*;
use uuid::Uuid;
use store::get_db_path;
use store::sqlite::crud::Peer::{create_peer, fetch_peer_by_id, fetch_peer_by_ip};

#[derive(Debug)]
pub struct Peer {
    stream: Option<TcpStream>,
    ping_time: u32,
    ip_addr: String,
    nick: String,
    whitelisted: bool,
    last_responded: SystemTime,
    id: String,
    thread_handle: Option<thread::JoinHandle<()>>
}


impl Clone for Peer {
    fn clone(&self) -> Self {
        Peer {
            stream: None,
            ping_time: self.get_ping_time(),
            ip_addr: self.get_ip_addr().to_owned(),
            nick: self.get_nick().to_string().to_owned(),
            whitelisted: self.get_whitelisted(),
            last_responded: self.get_last_responded(),
            id: self.get_id().to_owned(),
            thread_handle: None
        }
    }
}



impl Peer {
    pub fn new(ip: &str, stream: Option<TcpStream> ,nick: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut peer = Peer {
            stream: stream,
            ping_time: 9999,
            ip_addr: ip.to_string(),
            nick: nick.to_string(),
            whitelisted: false,
            last_responded: UNIX_EPOCH,
            id: Uuid::new_v4().to_string(),
            thread_handle: None
        };
        match create_peer(get_db_path().as_str(),
            id.as_str(),
            ip,
            nick,
            9999,
            false,
            UNIX_EPOCH
        ) {
            Ok(_) => {
                match fetch_peer_by_ip(get_db_path().as_str(), ip) {
                    Ok(result) => {
                        peer.ip_addr = result.get("ip").unwrap().to_string();
                        peer.id = result.get("id").unwrap().to_string();
                        peer.ping_time = result.get("ping").unwrap().to_string().parse().unwrap();
                        peer.nick = result.get("nick").unwrap().to_string();
                        peer.whitelisted = result.get("whitelisted").unwrap() == "true";
                        let last_responded: u64 = result.get("last_responded").unwrap().parse().unwrap();
                        peer.last_responded = UNIX_EPOCH + Duration::from_secs(last_responded);
                        peer
                    },
                    Err(err) => {
                        println!("Failed To Fetch Created Peer.({})!\nError: {}", ip, err);
                        peer
                    }
                }
            },
            Err(err) => {
                println!("Failed To Create Peer.({})!\nError: {}", ip, err);
                peer
            }
        }
    }
    pub fn read_stream(&mut self) -> Result<Vec<u8>, String>{
        let mut result: [u8; 256] = [0; 256];
        if let Some(stream) = &mut self.stream {
            match stream.read(&mut result) {
                Ok(bytes_read) => {
                    println!("Read {} bytes", bytes_read);
                    Ok(result.to_vec())
                },
                Err(e) => {
                    println!("{}", e.to_string());
                    Err(e.to_string())
                }
            }
        } else {
            Err("Peer Stream Not Connected! Cannot Read Data!".to_string())
        }
    }
    pub fn get_id(&self  ) -> &String {&self.id }
    pub fn get_stream(&self  ) -> Option<&TcpStream> {
        if let Some(t) = &self.stream {
            return Some(t);
        } else {
            return None;
        }
    }
    pub fn get_ping_time(&self  ) -> u32 { self.ping_time }
    pub fn get_nick(&self  ) -> &String { &self.nick }
    pub fn get_ip_addr(&self  ) -> &String { &self.ip_addr }
    pub fn get_last_responded(&self  ) -> SystemTime { self.last_responded }
    pub fn get_whitelisted(&self  ) -> bool { self.whitelisted }


    pub fn set_stream(&mut self, stream: TcpStream) { self.stream = Some(stream) }
    pub fn set_ping_time(&mut self, ping: u32) { self.ping_time = ping; }
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