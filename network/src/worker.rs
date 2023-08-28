use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::io::prelude::*;

use chrono::prelude::*;
use crate::peer::Peer;
use api::{qubic_api_t, qubic_api_type};

pub fn handle_new_peer(id: String, mut peer: Peer, rx: spmc::Receiver<qubic_api_t>, tx: std::sync::mpsc::Sender<qubic_api_t>) {
    println!("Handling New Peer! {}", id.as_str());
    if peer.get_stream().is_none() {
       println!("Peer {} Missing TcpStream! Shutting Down Worker Thread.", peer.get_id());
        return;
    }
            let mut stream = peer.get_stream().unwrap();
    loop {
        match rx.recv() {
            Ok(mut request) => {
                if let Some(request_id) = &request.peer {
                    println!("Received Work For Peer {} ! (I am {})", request_id.as_str(), id.as_str());
                    if request_id != id.as_str() {
                        continue;
                    }
                    match stream.write(request.as_bytes().as_slice()) {
                        Ok(_) => {
                            let response = ["Peer ", id.as_str(), " Responded At Time ", Utc::now().to_string().as_str()].join("");
                            println!( "Worker Thread Responding With {}", response.as_str());
                            let mut result: [u8; 256] = [0; 256];
                            match stream.read(&mut result) {
                                Ok(bytes_read) => {
                                    let api_response: Option<qubic_api_t> = qubic_api_t::format_response_from_bytes(result.to_vec());
                                    //TODO: auto format result into qubic_api_t (add func qubic_api_t::from_bytes(result))
                                    println!("Worker Thread Read Back {} Bytes!", bytes_read);
                                    println!("Read {:?}", result);
                                    if let Some(formatted_api_response) = api_response {
                                        tx.send(formatted_api_response);
                                    }
                                },
                                Err(err) => {
                                    println!("Worker Thread Failed To Read Response! : {}", err.to_string());
                                }
                            }
                        },
                        Err(err) =>{
                            println!("Failed To Send Data To Peer! {}", id.as_str());
                            println!("{}", err.to_string());
                        }
                    }
                }
            },
            Err(err) => {
                println!("Failed To Receive Work In Thread! {}", err.to_string());
            }
        }
    }
}
