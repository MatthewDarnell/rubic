use std::io::prelude::*;
use std::io::ErrorKind;
use std::time::Duration;
use crate::peer::Peer;
use api::{QubicApiPacket, response};
use api::header::{ RequestResponseHeader, EntityType };
use store::get_db_path;
use store::sqlite::crud::peer::set_peer_disconnected;

pub fn handle_new_peer(id: String, peer: Peer, rx: spmc::Receiver<QubicApiPacket>, tx: std::sync::mpsc::Sender<QubicApiPacket>) {
    if peer.get_stream().is_none() {
       println!("Peer {} Missing TcpStream! Shutting Down Worker Thread.", peer.get_id());
        return;
    }
    let mut stream = peer.get_stream().unwrap();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        //Block until we receive work
        match rx.clone().recv() {
            Ok(mut request) => {
                //println!("REQUEST : {:?}", &request.as_bytes());
                match stream.write(request.as_bytes().as_slice()) {
                    Ok(_) => {
                        stream.flush().unwrap();
                        //println!("Bytes Written, Attempting To Peek.");
                        //let response = ["Peer ", id.as_str(), " Responded At Time ", Utc::now().to_string().as_str()].join("");
                        //println!( "Worker Thread Responding");
                        let mut peeked: [u8; 8] = [0; 8];
                        match stream.peek(&mut peeked) {
                            Ok(_) => {
                                let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
                                //println!("RESPONSE: {:?}  {:?}", &peeked_header, &peeked_header.get_type());
                                let mut result_size: Vec<u8> = vec![0; peeked_header.get_size()];
                                match stream.read_exact(&mut result_size) {
                                    Ok(_) => {
                                        let api_response: Option<QubicApiPacket> = QubicApiPacket::format_response_from_bytes(peer.get_id(), result_size.to_vec());
                                        if let Some(mut formatted_api_response) = api_response {
                                            response::get_formatted_response(&mut formatted_api_response);
                                        }
                                    },
                                    Err(err) => {
                                        println!("Worker Thread Failed To Read Response! : {}", err.to_string());
                                    }
                                }
                            },
                            Err(err) => {
                                 println!("Failed To Peek! {}", err);
                                //set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()).unwrap();
                                // break;
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
                        //   println!("Failed To Send Data To Peer! {}", id.as_str());
                        // println!("{}", err.to_string());
                        let mut response: QubicApiPacket = QubicApiPacket::new(&error);
                        response.api_type = EntityType::ERROR;
                        response.peer = Some(peer.get_id().to_owned());
                        match set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()) {
                            Ok(_) => {},
                            Err(err) => {
                                println!("Failed To Set Peer {} disconnected: {}", peer.get_id().as_str(), err);
                            }
                        }
                        let mut counter = 0;
                        loop {
                            match tx.clone().send(response.clone()) {
                                Ok(_) => { break; },
                                Err(err) => {
                                    counter = counter + 1;
                                    //println!("Failed to send Message from Worker Thread.({}) To Handler... ({})", peer.get_id().as_str(), err.to_string())
                                }
                            }
                            if counter > 10 {
                                //println!("Failed to send Message from Worker Thread.({}) To Handler... Bailing Out", peer.get_id().as_str());
                                break;
                            }
                        }
                        break;
                    }
                }
            },
            Err(err) => {
                println!("Failed To Receive Work In Thread! {}", err.to_string());
            }
        }
    }
   // println!("Worker Peer Thread Exiting!");
}
