use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::io::prelude::*;
use std::thread;
use chrono::prelude::*;
use uuid::Uuid;

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
        Peer {
            stream: stream,
            ping_time: 9999,
            ip_addr: ip.to_string(),
            nick: nick.to_string(),
            whitelisted: false,
            last_responded: SystemTime::now(),
            id: Uuid::new_v4().to_string(),
            thread_handle: None
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