use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::io::prelude::*;

use chrono::prelude::*;
use crate::peer::Peer;
use api::{qubic_api_t, response};

pub fn listen_for_api_responses(rx: std::sync::mpsc::Receiver<qubic_api_t>) {
    loop {
        match rx.recv() {
            Ok(mut r) => response::get_formatted_response(&mut r),
            Err(err) => println!("{}", err.to_string())
        }
    }
}