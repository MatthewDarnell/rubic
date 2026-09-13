use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use api::header::{EntityType, RequestResponseHeader};
use api::request::QubicApiPacket;
use api::response;
use crate::peer::Peer;

/// What a non-blocking peek at the socket told us.
enum Peek {
    Data,
    /// Read timeout elapsed: the peer is slow or done sending, not gone.
    NoData,
    /// EOF or a hard socket error: the peer is gone.
    Closed,
}

fn peek_header(stream: &TcpStream, buf: &mut [u8; 8]) -> Peek {
    match stream.peek(buf) {
        Ok(0) => Peek::Closed,
        Ok(_) => Peek::Data,
        Err(err) => match err.kind() {
            ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted => Peek::NoData,
            _ => Peek::Closed,
        },
    }
}

/// Reads the response(s) to the request just written. Returns `false` only when
/// the socket is actually dead; a timeout just means the response was incomplete.
pub fn qubic_tcp_receive_data (peer: &Peer, requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, stream: &TcpStream) -> bool {
    let mut peeked: [u8; 8] = [0; 8];
    match peek_header(stream, &mut peeked) {
        Peek::Data => {
            let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
            match peeked_header.recv_multiple_packets() {
                true => {
                    let (mut data, alive) = recv_qubic_responses_until_end_response(peer, stream, 676);
                    response::get_formatted_response_from_multiple(requests, &mut data);
                    alive
                },
                false => {
                    match recv_qubic_response(peer, stream) {
                        Some(mut data) => {
                            response::get_formatted_response(requests, &mut data);
                            true
                        },
                        None => true,
                    }
                }
            }
        },
        Peek::NoData => true,
        Peek::Closed => false,
    }
}


fn recv_qubic_response(peer: &Peer, stream: &TcpStream) -> Option<QubicApiPacket> {
    let mut peeked: [u8; 8] = [0; 8];
    match peek_header(stream, &mut peeked) {
        Peek::Data => {
            let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
            let mut result_size: Vec<u8> = vec![0; peeked_header.get_size()];
            match stream.try_clone() {  //1 worker thread per tcp stream, should be fine to clone
                Ok(mut stream) => {
                    match stream.read_exact(&mut result_size) {
                        Ok(_) => {
                            QubicApiPacket::format_response_from_bytes(peer.get_id(), result_size.to_vec())
                        },
                        Err(_err) => None,
                    }
                },
                Err(_) => {
                    eprintln!("Failed To Clone TCP Stream");
                    None
                }
            }
        },
        _ => None,
    }
}


/// Collects packets until a ResponseEnd marker. The second value is `false` when
/// the socket closed underneath us.
fn recv_qubic_responses_until_end_response(peer: &Peer, stream: &TcpStream, max_packets_to_read: u32) -> (Vec<QubicApiPacket>, bool) {
    let mut data: Vec<QubicApiPacket> = Vec::new();
    let mut peeked: [u8; 8] = [0; 8];
    loop {
        match peek_header(stream, &mut peeked) {
            Peek::Data => {
                let peeked_header = RequestResponseHeader::from_vec(&peeked.to_vec());
                if peeked_header.get_type().to_byte() == EntityType::ResponseEnd.to_byte() {
                    return (data, true);
                }
                peeked = [0; 8];
                match recv_qubic_response(peer, stream) {
                    Some(packet) => {
                        data.push(packet);
                        if data.len() > max_packets_to_read as usize {
                            return (data, true);
                        }
                    },
                    // Could not read a full packet; keep what we have.
                    None => return (data, true),
                }
            },
            // Slow peer or truncated response: keep the partial data, the peer stays.
            Peek::NoData => return (data, true),
            Peek::Closed => return (data, false),
        }
    }
}
