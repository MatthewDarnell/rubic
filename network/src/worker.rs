use std::io::prelude::*;
use std::io::ErrorKind;
use std::time::Duration;
use crate::peer::Peer;
use api::QubicApiPacket;
use store::get_db_path;
use store::sqlite::peer::set_peer_disconnected;
use crate::tcp_recv::qubic_tcp_receive_data;

pub fn handle_new_peer(_id: String, peer: Peer, rx: spmc::Receiver<QubicApiPacket>) {
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
                match stream.write(request.as_bytes().as_slice()) {
                    Ok(_) => {
                        stream.flush().unwrap();
                        //let response = ["Peer ", id.as_str(), " Responded At Time ", Utc::now().to_string().as_str()].join("");
                        //println!( "Worker Thread Responding");
                        qubic_tcp_receive_data(&peer, stream);
                    },
                    Err(err) => {   //Probably the peer closed the tcp connection
                        let _error = match err.kind() {
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
                                //println!("Unknown Error Kind! : {}", err.kind().to_string());
                                "Unknown Peer Error!".as_bytes()
                            }
                        }
                            .to_vec();
                        //   println!("Failed To Send Data To Peer! {}", id.as_str());
                        // println!("{}", err.to_string());
                        match set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()) {
                            Ok(_) => {},
                            Err(err) => {
                                println!("Failed To Set Peer {} disconnected: {}", peer.get_id().as_str(), err);
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
