use std::io::Read;
use std::net::TcpStream;
use api::header::{EntityType, RequestResponseHeader};
use api::{response, QubicApiPacket};
use store::get_db_path;
use store::sqlite::crud::peer::set_peer_disconnected;
use crate::peer::Peer;

pub fn qubic_tcp_receive_data (peer: &Peer, stream: &TcpStream) {
    let mut peeked: [u8; 8] = [0; 8];
    match stream.peek(&mut peeked) {
        Ok(_) => {
            let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
            match peeked_header.recv_multiple_packets() {
                true => {
                    let mut data = recv_qubic_responses_until_end_response(peer, stream, 676);
                    //println!("Received Multiple Data: {} From Peer {}", data.len(), peer.get_ip_addr());
                    response::get_formatted_response_from_multiple(&mut data);
                },
                false => {
                    match recv_qubic_response(peer, stream) {
                        Some(mut data) => response::get_formatted_response(&mut data),
                        None => {}
                    }
                }
            };
            
        },
        Err(_) => {}
    }
}


fn recv_qubic_response(peer: &Peer, stream: &TcpStream) -> Option<QubicApiPacket> {
    let mut peeked: [u8; 8] = [0; 8];
    match stream.peek(&mut peeked) {
        Ok(_) => {
            let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
            //println!("RESPONSE: {:?}  {:?}, {} bytes", &peeked_header, &peeked_header.get_type(), &peeked_header.get_size());
            let mut result_size: Vec<u8> = vec![0; peeked_header.get_size()];
            match stream.try_clone() {  //1 worker thread per tcp stream, should be fine to clone
                Ok(mut stream) => {
                    match stream.read_exact(&mut result_size) {
                        Ok(_) => {
                            let api_response: Option<QubicApiPacket> = QubicApiPacket::format_response_from_bytes(peer.get_id(), result_size.to_vec());
                            api_response
                        },
                        Err(err) => {
                            eprintln!("Failed To Read Response! : {}", err.to_string());
                            None
                        }
                    }
                },
                Err(_) => {
                    eprintln!("Failed To Clone TCP Stream");
                    None
                }
            }
        },
        Err(_err) => {
            //println!("Failed To Peek! {}", _err);
            set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()).unwrap();
            None
        }
    }
}


fn recv_qubic_responses_until_end_response(peer: &Peer, stream: &TcpStream, max_packets_to_read: u32) -> Vec<QubicApiPacket> {
    let mut data: Vec<QubicApiPacket> = Vec::new();
    let mut peeked: [u8; 8] = [0; 8];
    let mut peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
    loop {
        match stream.peek(&mut peeked) {
            Ok(_) => {
                peeked_header = RequestResponseHeader::from_vec(&peeked.to_vec());
                if peeked_header.get_type().to_byte() == EntityType::ResponseEnd.to_byte() {
                    return data;
                }
                peeked = [0; 8];
                match recv_qubic_response(peer, stream) {
                    Some(packet) => {
                        //println!("read multiple packet");
                        data.push(packet);
                        if data.len() > max_packets_to_read as usize {
                            println!("Breaking");
                            break;
                        } else {
                            continue;
                        }
                    },
                    None => {
                        eprintln!("Failed To Read Multiple Data");
                    }
                }
                return data;
            },
            Err(_err) => {
                //println!("Failed To Peek! {}", _err);
                set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()).unwrap();
                break;
            }
        }
    }
    println!("Read {} Data Packets. Last Packet Type={:?}", &data.len(), peeked_header.get_type());
    data
}