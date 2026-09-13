use std::collections::HashMap;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use crate::peer::Peer;
use api::request::QubicApiPacket;
use store::get_db_path;
use store::sqlite::peer::{set_peer_disconnected, update_peer_responded};
use crate::tcp_recv::{drain_buffered, qubic_tcp_receive_data};

/// How often a responsive peer's "last responded" / latency is written to the
/// database. Replies arrive many times a second; the UI only needs seconds.
const RESPONDED_WRITE_INTERVAL: Duration = Duration::from_secs(5);

pub fn handle_new_peer(_id: String, request_matcher: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, peer: Peer, rx: spmc::Receiver<QubicApiPacket>, backlog: Arc<AtomicUsize>, tick_backlog: Arc<AtomicUsize>) {
    if peer.get_stream().is_none() {
       println!("Peer {} Missing TcpStream! Shutting Down Worker Thread.", peer.get_id());
        return;
    }
    let mut stream = peer.get_stream().unwrap();
    let mut last_responded_write: Option<Instant> = None;
    loop {
        std::thread::sleep(Duration::from_millis(100));
        // The PeerSet dropped this peer (or the flag was cleared elsewhere): stop consuming work.
        if !peer.is_connected() {
            break;
        }
        //Block until we receive work
        match rx.clone().recv() {
            Ok(mut request) => {
                // Picked up: no longer part of the backlog the PeerSet throttles on.
                let _ = backlog.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| Some(n.saturating_sub(1)));
                if matches!(request.api_type, api::header::EntityType::RequestCurrentTickInfo) {
                    let _ = tick_backlog.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| Some(n.saturating_sub(1)));
                }
                match request_matcher.lock() {
                    Ok(mut matcher) => {
                        matcher.insert(request.header._dejavu, request.clone());
                    },
                    Err(_) => {}
                }
                match stream.write(request.as_bytes().as_slice()) {
                    Ok(_) => {
                        stream.flush().ok();
                        // Nodes never answer a broadcast; waiting would just burn the
                        // 2.5 s read timeout on every peer for every re-send.
                        if matches!(request.api_type, api::header::EntityType::BroadcastTransaction) {
                            continue;
                        }
                        let sent_at = Instant::now();
                        let outcome = qubic_tcp_receive_data(&peer, request_matcher.clone(), stream);
                        if !outcome.alive {
                            mark_disconnected(&peer);
                            break;
                        }
                        // Any reply proves the peer is alive; record it (and the round
                        // trip) at most every few seconds so the Network tab stays honest.
                        if outcome.replied
                            && last_responded_write.map_or(true, |at| at.elapsed() >= RESPONDED_WRITE_INTERVAL)
                        {
                            let ping_ms = sent_at.elapsed().as_millis().min(9998) as u32;
                            if let Err(err) = update_peer_responded(get_db_path().as_str(), peer.get_id().as_str(), ping_ms) {
                                println!("Failed to record response from peer {}: {}", peer.get_id(), err);
                            }
                            last_responded_write = Some(Instant::now());
                        }
                        // Nodes push tick/transaction gossip on their own; swallow what
                        // is already buffered now rather than one packet per request.
                        if !drain_buffered(&peer, request_matcher.clone(), stream) {
                            mark_disconnected(&peer);
                            break;
                        }
                    },
                    Err(_err) => {   //Probably the peer closed the tcp connection
                        mark_disconnected(&peer);
                        break;
                    }
                }
            },
            Err(err) => {
                println!("Failed To Receive Work In Thread! {}", err.to_string());
                break;
            }
        }
    }
   // println!("Worker Peer Thread Exiting!");
}

fn mark_disconnected(peer: &Peer) {
    peer.set_connected(false);
    match set_peer_disconnected(get_db_path().as_str(), peer.get_id().as_str()) {
        Ok(_) => {},
        Err(err) => {
            println!("Failed To Set Peer {} disconnected: {}", peer.get_id().as_str(), err);
        }
    }
}
