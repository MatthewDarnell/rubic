use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use api::header::{EntityType, RequestResponseHeader};
use api::request::QubicApiPacket;
use api::response;
use crate::peer::Peer;

/// How long to wait for the first packet of a reply to a request we just sent.
pub const RESPONSE_TIMEOUT: Duration = Duration::from_millis(2500);
/// How long to wait for the *next* packet of a multi-packet reply. Nodes also
/// push unsolicited multi-packet types (BroadcastTick gossip) that never end in
/// a ResponseEnd; waiting the full response timeout for each of those was the
/// single biggest drain on worker throughput.
const CONTINUATION_TIMEOUT: Duration = Duration::from_millis(500);
/// How long to wait when draining packets the node pushed on its own.
const DRAIN_TIMEOUT: Duration = Duration::from_millis(50);
/// Upper bounds on a drain, so a chatty node cannot starve requests: whichever
/// of packet count or elapsed time is hit first ends it (leftovers wait for the
/// next request cycle).
const MAX_DRAIN_PACKETS: usize = 32;
const MAX_DRAIN_TIME: Duration = Duration::from_millis(200);

/// What a non-blocking peek at the socket told us.
enum Peek {
    Data,
    /// Read timeout elapsed: the peer is slow or done sending, not gone.
    NoData,
    /// EOF or a hard socket error: the peer is gone.
    Closed,
}

fn set_timeout(stream: &TcpStream, timeout: Duration) {
    stream.set_read_timeout(Some(timeout)).ok();
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

/// Result of waiting for the reply to one request.
#[derive(Clone, Copy, Debug)]
pub struct RecvOutcome {
    /// `false` only when the socket is actually dead.
    pub alive: bool,
    /// Whether the peer sent anything back before the read timeout.
    pub replied: bool,
}

/// Reads one packet (or one multi-packet group) with whatever read timeout is
/// currently set on the socket, and hands it to the response handlers.
fn receive_one(peer: &Peer, requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, stream: &TcpStream) -> RecvOutcome {
    let mut peeked: [u8; 8] = [0; 8];
    match peek_header(stream, &mut peeked) {
        Peek::Data => {
            let peeked_header: RequestResponseHeader = RequestResponseHeader::from_vec(&peeked.to_vec());
            match peeked_header.recv_multiple_packets() {
                true => {
                    let (mut data, alive) = recv_qubic_responses_until_end_response(peer, stream, 676);
                    // An immediate ResponseEnd yields no packets; the handler expects at least one.
                    if !data.is_empty() {
                        response::get_formatted_response_from_multiple(requests, &mut data);
                    }
                    RecvOutcome { alive, replied: true }
                },
                false => {
                    match recv_qubic_response(peer, stream) {
                        Some(mut data) => {
                            response::get_formatted_response(requests, &mut data);
                            RecvOutcome { alive: true, replied: true }
                        },
                        None => RecvOutcome { alive: true, replied: true },
                    }
                }
            }
        },
        Peek::NoData => RecvOutcome { alive: true, replied: false },
        Peek::Closed => RecvOutcome { alive: false, replied: false },
    }
}

/// Waits (up to `RESPONSE_TIMEOUT`) for the reply to the request just written and
/// processes it. `alive` is `false` only when the socket is dead; a timeout just
/// means the peer did not answer in time.
pub fn qubic_tcp_receive_data (peer: &Peer, requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, stream: &TcpStream) -> RecvOutcome {
    set_timeout(stream, RESPONSE_TIMEOUT);
    receive_one(peer, requests, stream)
}

/// Processes packets the node has already pushed to us on its own (tick and
/// transaction gossip, peer lists) without blocking on the socket for long.
/// Returns `false` when the socket turned out to be dead.
pub fn drain_buffered(peer: &Peer, requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, stream: &TcpStream) -> bool {
    let started = std::time::Instant::now();
    for _ in 0..MAX_DRAIN_PACKETS {
        if started.elapsed() >= MAX_DRAIN_TIME {
            break;
        }
        set_timeout(stream, DRAIN_TIMEOUT);
        let outcome = receive_one(peer, requests.clone(), stream);
        if !outcome.alive {
            set_timeout(stream, RESPONSE_TIMEOUT);
            return false;
        }
        if !outcome.replied {
            break;
        }
    }
    set_timeout(stream, RESPONSE_TIMEOUT);
    true
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


/// Collects packets until a ResponseEnd marker (which is consumed). After the
/// first packet only `CONTINUATION_TIMEOUT` is allowed between packets. The
/// second value is `false` when the socket closed underneath us.
fn recv_qubic_responses_until_end_response(peer: &Peer, stream: &TcpStream, max_packets_to_read: u32) -> (Vec<QubicApiPacket>, bool) {
    let mut data: Vec<QubicApiPacket> = Vec::new();
    let mut peeked: [u8; 8] = [0; 8];
    let result = loop {
        match peek_header(stream, &mut peeked) {
            Peek::Data => {
                let peeked_header = RequestResponseHeader::from_vec(&peeked.to_vec());
                if peeked_header.get_type().to_byte() == EntityType::ResponseEnd.to_byte() {
                    // Consume the marker so it is not mistaken for the next reply.
                    let _ = recv_qubic_response(peer, stream);
                    break (data, true);
                }
                peeked = [0; 8];
                match recv_qubic_response(peer, stream) {
                    Some(packet) => {
                        data.push(packet);
                        if data.len() > max_packets_to_read as usize {
                            break (data, true);
                        }
                        set_timeout(stream, CONTINUATION_TIMEOUT);
                    },
                    // Could not read a full packet; keep what we have.
                    None => break (data, true),
                }
            },
            // Slow peer, gossip without an end marker, or a truncated response:
            // keep the partial data, the peer stays.
            Peek::NoData => break (data, true),
            Peek::Closed => break (data, false),
        }
    };
    set_timeout(stream, RESPONSE_TIMEOUT);
    result
}
