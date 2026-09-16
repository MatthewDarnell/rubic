use std::io::prelude::*;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use api::request::QubicApiPacket;
use logger::debug;
use std::time::{Duration};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use store;

use crate::worker;
use crate::queue::RequestQueue;
use crate::peer::Peer;

pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(1500);

/// Opens a socket to a peer. This blocks for up to `timeout`, so call it
/// without holding the PeerSet mutex and hand the stream to `add_connected_peer`.
pub fn connect(ip: &str, timeout: Duration) -> Result<TcpStream, String> {
    let sock: SocketAddr = ip.parse().map_err(|e| format!("Invalid peer address <{}>: {}", ip, e))?;
    let stream = TcpStream::connect_timeout(&sock, timeout).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_millis(2500))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_millis(2500))).map_err(|e| e.to_string())?;
    stream.set_nodelay(true).ok();
    stream.set_ttl(100).ok();
    Ok(stream)
}

/// Requests queued for the worker threads beyond which routine polling is
/// dropped rather than queued. Every request occupies a peer's single socket for
/// a full round trip (up to the 2.5 s read timeout), so an unbounded queue turns
/// into seconds of latency for everything behind it - including transaction
/// broadcasts and the tick poll that decides whether a transfer is still in time.
pub const MAX_REQUEST_BACKLOG: usize = 64;
/// Queue depth above which low-priority polling (the order-book sweep over every
/// asset) yields to balances and everything else.
pub const LOW_PRIORITY_BACKLOG: usize = 8;
/// Copies of a high-priority request put on the shared work queue. Workers take
/// the next item as soon as their socket is free, so the copies land on the
/// peers that are answering fastest right now without any routing.
pub const HIGH_PRIORITY_COPIES: usize = 2;

/// Process-wide view of the request queue, for the /health route (the PeerSet
/// itself lives inside the peer-loop thread and is not reachable from routes).
static GLOBAL_BACKLOG: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_TICK_BACKLOG: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_PEERS: AtomicUsize = AtomicUsize::new(0);

/// `(queued requests, of which tick polls, connected peers)`.
pub fn queue_stats() -> (usize, usize, usize) {
    (
        GLOBAL_BACKLOG.load(Ordering::Relaxed),
        GLOBAL_TICK_BACKLOG.load(Ordering::Relaxed),
        GLOBAL_PEERS.load(Ordering::Relaxed),
    )
}

/// How eagerly a routine request is queued when the workers are behind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestPriority {
    /// Background sweeps: only while the routine queue is nearly empty.
    Low,
    /// Balances, holdings, asset lists.
    Normal,
    /// What the user is watching right now.
    High,
    /// A transaction's first broadcast: to every peer, ahead of the queue.
    Urgent,
}

pub struct PeerSet {
  peers: Vec<Peer>,
  queue: Arc<RequestQueue>,
  request_matcher: Arc<Mutex<HashMap<u32, QubicApiPacket>>>,
  threads: HashMap<String, std::thread::JoinHandle<()>>,
  /// Requests handed to the channel that no worker has picked up yet.
  backlog: Arc<AtomicUsize>,
  /// The subset of `backlog` that are current-tick polls.
  tick_backlog: Arc<AtomicUsize>,
}




impl PeerSet {
    pub fn new() -> Self {
        let peer_set = PeerSet {
            peers: vec![],
            threads: HashMap::new(),
            request_matcher: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(RequestQueue::new()),
            backlog: Arc::new(AtomicUsize::new(0)),
            tick_backlog: Arc::new(AtomicUsize::new(0)),
        };
        peer_set
    }
    /// Number of queued requests no worker has started on yet.
    pub fn backlog(&self) -> usize {
        self.backlog.load(Ordering::Relaxed)
    }
    /// Copies the counters into the process-wide stats read by /health.
    fn publish_stats(&self) {
        GLOBAL_BACKLOG.store(self.backlog.load(Ordering::Relaxed), Ordering::Relaxed);
        GLOBAL_TICK_BACKLOG.store(self.tick_backlog.load(Ordering::Relaxed), Ordering::Relaxed);
        GLOBAL_PEERS.store(self.peers.len(), Ordering::Relaxed);
    }
    /// Queued requests other than the constant current-tick polls: what the
    /// low-priority sweeps have to wait behind.
    pub fn routine_backlog(&self) -> usize {
        self.backlog.load(Ordering::Relaxed).saturating_sub(self.tick_backlog.load(Ordering::Relaxed))
    }
    pub fn get_peers(&self) -> Vec<&Peer> { self.peers.iter().map(|x| x).collect() }
    pub fn get_peer_ids(&self) -> Vec<String> { self.peers.iter().map(|x| x.get_id().to_owned()).collect() }
    pub fn get_peer_ips(&self) -> Vec<String> { self.peers.iter().map(|x| x.get_ip_addr().to_owned()).collect() }
    pub fn num_peers(&self) -> usize {
        self.peers.len()
    }
    pub fn num_connected_peers(&self) -> usize {
        self.peers.iter().filter(|p| p.is_connected()).count()
    }
    pub fn has_peer(&self, ip: &str) -> bool {
        self.peers.iter().any(|p| p.get_ip_addr() == ip)
    }

    /// Convenience for callers that are not holding the mutex (tests, one-offs).
    /// Blocks for the connect timeout; the peer loops use `connect` + `add_connected_peer`.
    pub fn add_peer(&mut self, ip: &str) -> Result<(), String> {
        let stream = connect(ip, Duration::from_millis(5000))?;
        self.add_connected_peer(ip, stream)
    }

    /// Registers an already-open socket. Cheap: no network I/O under the lock.
    pub fn add_connected_peer(&mut self, ip: &str, stream: TcpStream) -> Result<(), String> {
        if let Ok(max_peers) = std::env::var("RUBIC_MAX_PEERS") {
            if let Ok(max) = max_peers.parse::<usize>() {
                if self.peers.len() >= max {
                    return Err("Already At Max Capacity of Connected Peers".to_string());
                }
            }
        }
        if self.has_peer(ip) {
            return Err("Duplicate Peer".to_string());
        }
        let new_peer = Peer::new(ip, None, "");
        new_peer.set_connected(true);
        let request_matcher = Arc::clone(&self.request_matcher);
        let id = new_peer.get_id().to_owned();
        {
            let mut peer = new_peer.clone();
            peer.set_stream(stream);
            let queue = Arc::clone(&self.queue);
            let thread_id = id.clone();
            let backlog = Arc::clone(&self.backlog);
            let tick_backlog = Arc::clone(&self.tick_backlog);
            let t = std::thread::spawn(move || worker::handle_new_peer(thread_id, request_matcher, peer, queue, backlog, tick_backlog));
            self.threads.insert(id.clone(), t);
        }
        self.peers.push(new_peer);
        match store::sqlite::peer::set_peer_connected(
            store::get_db_path().as_str(),
            id.as_str()
        ) {
            Ok(_) => { debug(format!("Set Peer {} Connected.", id.as_str()).as_str()); },
            Err(err) => { debug(format!("Error Setting Peer {} Connected! : {}", id.as_str(), err.as_str()).as_str()); }
        }
        Ok(())
    }

    pub fn delete_peer(&mut self, ip: &str) -> bool {
        for (index, connection) in self.peers.iter_mut().enumerate() {
            if let Some(stream) = &mut connection.get_stream() {
                match stream.peer_addr() {
                    Ok(conn) => {
                        if Ok(conn) == ip.parse() {
                            self.peers.remove(index);
                            return true;
                        }
                    },
                    Err(_) => {}
                }
            }
        }
        false
    }

    pub fn delete_peer_by_id(&mut self, id: &str) -> bool {
        for (index, connection) in self.peers.iter_mut().enumerate() {
            if connection.get_id().as_str() == id { //this is the peer, disconnect its stream
                connection.set_connected(false);
                if let Some(stream) = &mut connection.get_stream() {
                    match stream.shutdown(std::net::Shutdown::Both) {
                        Ok(_) => {},
                        Err(_) => {}
                    }
                }
                match store::sqlite::peer::set_peer_disconnected(
                    store::get_db_path().as_str(),
                    id
                ) {
                    Ok(_) => {
                        self.peers.remove(index);
                        self.threads.remove(id);
                        return true;
                    },
                    Err(err) => {
                        println!("Error Deleting Peer By Id.({}) : {}", id, err.as_str());
                    }
                }
            }
        }
        false
    }

    /// Drops peers whose worker thread has flagged the socket dead.
    pub fn prune_disconnected(&mut self) -> usize {
        let dead: Vec<String> = self.peers.iter()
            .filter(|p| !p.is_connected())
            .map(|p| p.get_id().to_owned())
            .collect();
        for id in &dead {
            self.delete_peer_by_id(id.as_str());
        }
        dead.len()
    }

    fn _send_request_via_stream(&mut self, stream: &mut TcpStream, request: &Vec<u8>) -> Result<(), String> {
        match stream.write(request.as_slice()) {
            Ok(_) => {
                let mut result: [u8; 256] = [0; 256];
                match stream.read(&mut result) {
                    Ok(bytes_read) => {
                        println!("Read {} bytes", bytes_read);
                        Ok(())
                    },
                    Err(e) => {
                        println!("{}", e.to_string());
                        Err(e.to_string())
                    }
                }
            },
            Err(err) =>{
                println!("Failed To Send Data To Peer!");
                Err(err.to_string())
            }
        }
    }

    pub fn make_request(&mut self, request: QubicApiPacket) -> Result<(), String> {
        self.make_request_with_priority(request, RequestPriority::Normal)
    }

    /// For background sweeps whose freshness matters least: sent only while the
    /// queue is nearly empty, so identity balances always get through first.
    pub fn make_request_low_priority(&mut self, request: QubicApiPacket) -> Result<(), String> {
        self.make_request_with_priority(request, RequestPriority::Low)
    }

    /// For what the user is looking at right now (the order book on screen):
    /// queued even when routine polling is being held back.
    pub fn make_request_high_priority(&mut self, request: QubicApiPacket) -> Result<(), String> {
        self.make_request_with_priority(request, RequestPriority::High)
    }

    /// A transaction's first broadcast: every peer, ahead of everything queued.
    pub fn make_request_urgent(&mut self, request: QubicApiPacket) -> Result<(), String> {
        self.make_request_with_priority(request, RequestPriority::Urgent)
    }

    fn make_request_with_priority(&mut self, mut request: QubicApiPacket, priority: RequestPriority) -> Result<(), String> {
        // Evict anything the workers have flagged dead first; no database round trips here.
        self.prune_disconnected();
        self.publish_stats();
        if self.peers.is_empty() {
            return Err("Cannot send request, 0 peers! Add some!".to_string())
        }

        // Requests that go to a single random peer. The current-tick poll is
        // deliberately NOT in this list: asking every peer and keeping the highest
        // answer keeps the wallet's tick within a tick or two of the network
        // (one random peer at a time left it 6-15 ticks behind, which is what
        // decides whether a transfer's expiration tick is still in the future).
        // Duplicate tick replies are harmless: the store ignores conflicts.
        let spam_all: bool = match request.api_type {
            api::header::EntityType::RequestedQuorumTick => false,
            api::header::EntityType::RequestTickData => false,
            api::header::EntityType::RequestContractFunction => false,
            api::header::EntityType::RequestAssets => false,
            _ => true
        };

        // Transaction broadcasts are never dropped. The tick poll goes to every
        // peer every second, so it is capped at two rounds' worth: enough to keep
        // the tick current, never enough to crowd out balances and the rest,
        // which wait for the next pass whenever the workers are behind.
        let is_tick_poll = matches!(request.api_type, api::header::EntityType::RequestCurrentTickInfo);
        let (queued, limit) = match request.api_type {
            api::header::EntityType::BroadcastTransaction => (0, usize::MAX),
            api::header::EntityType::RequestCurrentTickInfo => (self.tick_backlog.load(Ordering::Relaxed), (2 * self.peers.len()).max(2)),
            // The confirmer's lookups are rare (one per pending transfer per pass)
            // and are what turns "pending" into confirmed/failed: never starve them.
            api::header::EntityType::RequestedQuorumTick | api::header::EntityType::RequestTickData => (0, usize::MAX),
            _ => (
                self.routine_backlog(),
                match priority {
                    RequestPriority::Low => LOW_PRIORITY_BACKLOG,
                    RequestPriority::Normal => MAX_REQUEST_BACKLOG,
                    RequestPriority::High => MAX_REQUEST_BACKLOG * 4,
                    RequestPriority::Urgent => usize::MAX,
                },
            ),
        };
        if queued >= limit {
            return Err(format!("Request backlog full ({} queued); skipping {:?} until the workers catch up", queued, request.api_type));
        }

        // Nodes answer contract-function (order book) requests slowly and
        // unevenly; for the book the user is watching, queue a couple of copies
        // so the idle (responsive) peers pick them up, instead of waiting on one
        // random peer - or tying up every socket, which asking all peers did.
        let targets: Vec<String> = if spam_all {
            self.peers.iter().map(|p| p.get_id().to_owned()).collect()
        } else if priority == RequestPriority::High {
            self.peers.iter().choose_multiple(&mut thread_rng(), HIGH_PRIORITY_COPIES)
                .into_iter().map(|p| p.get_id().to_owned()).collect()
        } else {
            match self.peers.iter().choose(&mut thread_rng()) {
                Some(p) => vec![p.get_id().to_owned()],
                None => vec![],
            }
        };

        // A transaction's first send is the one thing the user is waiting on: it
        // goes to the front of the queue, ahead of every poll already waiting.
        // Resends take the normal path so they cannot starve the tick polls.
        let urgent = priority == RequestPriority::Urgent;
        for id in targets {
            request.peer = Some(id);
            if urgent {
                self.queue.push_front(request.clone());
            } else {
                self.queue.push_back(request.clone());
            }
            self.backlog.fetch_add(1, Ordering::Relaxed);
            if is_tick_poll {
                self.tick_backlog.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }
}



#[cfg(test)]
pub mod peer_tests {
    use crate::peers::PeerSet;

    #[test]
    fn add_a_peer() {
        let mut p_set = PeerSet::new();
        match p_set.add_peer("127.0.0.1:8000") {
            Ok(_) => {
                assert_eq!(p_set.num_peers(), 1);
            },
            Err(err) => {
                assert!(err.contains("refused"));
                assert_eq!(p_set.num_peers(), 0);
            }
        }
    }
}
