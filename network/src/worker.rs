use std::io::prelude::*;
use std::io::ErrorKind;
use crate::peer::Peer;
use api::{QubicApiPacket};
use api::header::EntityType;

pub fn handle_new_peer(id: String, peer: Peer, rx: spmc::Receiver<QubicApiPacket>, tx: std::sync::mpsc::Sender<QubicApiPacket>) {
    println!("Handling New Peer! {}", id.as_str());
    if peer.get_stream().is_none() {
       println!("Peer {} Missing TcpStream! Shutting Down Worker Thread.", peer.get_id());
        return;
    }
    let mut stream = peer.get_stream().unwrap();
    let mut result: [u8; 1024];
    loop {
        match rx.recv() {
            Ok(mut request) => {
                if let Some(request_id) = &request.peer {
                    if request_id != id.as_str() {
                        continue;
                    }
                    //println!("Received Work For Peer {} ! (I am {})", request_id.as_str(), id.as_str());
                    match stream.write(request.as_bytes().as_slice()) {
                        Ok(_) => {
                            result = [0; 1024];
                            //let response = ["Peer ", id.as_str(), " Responded At Time ", Utc::now().to_string().as_str()].join("");
                            //println!( "Worker Thread Responding With {}", response.as_str());
                            match stream.read(&mut result) {
                                Ok(_) => {
                                    let api_response: Option<QubicApiPacket> = QubicApiPacket::format_response_from_bytes(peer.get_id(), result.to_vec());
                                    //TODO: auto format result into QubicApiPacket (add func QubicApiPacket::from_bytes(result))
                                    //println!("Worker Thread Read Back {} Bytes!", bytes_read);
                                    //println!("Read {:?}", result);
                                    if let Some(formatted_api_response) = api_response {
                                        match tx.send(formatted_api_response) {
                                            Ok(_) => {},
                                            Err(err) => println!("Failed to send Message from Worker Thread.({}) To Handler... ({})", peer.get_id().as_str(), err.to_string())
                                        }
                                    }
                                },
                                Err(err) => {
                                    println!("Worker Thread Failed To Read Response! : {}", err.to_string());
                                }
                            }
                        },
                        Err(err) => {
                            let error = match err.kind() {
                                ErrorKind::ConnectionAborted => {
                                    "Connection Aborted!".as_bytes()
                                },
                                ErrorKind::ConnectionRefused => {
                                    "Connection Refused!".as_bytes()
                                },
                                ErrorKind::ConnectionReset => {
                                    "Connection Reset!".as_bytes()
                                },
                                ErrorKind::NotConnected => {
                                    "Not Connected!".as_bytes()
                                },
                                _ => {
                                     println!("Unknown Error Kind! : {}", err.kind().to_string());
                                    "Unknown Peer Error!".as_bytes()
                                }
                            }
                                .to_vec();
                            println!("Failed To Send Data To Peer! {}", id.as_str());
                            println!("{}", err.to_string());
                            let mut response: QubicApiPacket = QubicApiPacket::new(&error);
                            response.api_type = EntityType::ERROR;
                            response.peer = Some(peer.get_id().to_owned());
                            match tx.send(response) {
                                Ok(_) => {},
                                Err(err) => println!("Failed to send Message from Worker Thread.({}) To Handler... ({})", peer.get_id().as_str(), err.to_string())
                            }
                        }
                    }
                }
            },
            Err(err) => {
                //println!("Failed To Receive Work In Thread! {}", err.to_string());
            }
        }
    }
}
