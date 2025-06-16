mod connected_peer_maintainer;
mod latest_tick_monitor;
mod balance_updater;
mod disconnected_peer_handler;
mod transaction_broadcaster;
mod transaction_confirmer;
mod broadcast_computors_updater;

use std::sync::{mpsc, Arc, Mutex};
use network::peers::PeerSet;

use crate::peer_loop::balance_updater::update_balances;
use crate::peer_loop::broadcast_computors_updater::update_broadcast_computors;
use crate::peer_loop::connected_peer_maintainer::maintain_peers;
use crate::peer_loop::disconnected_peer_handler::handle_disconnected_peers;
use crate::peer_loop::latest_tick_monitor::monitor_latest_tick;
use crate::peer_loop::transaction_broadcaster::broadcast_transactions;
use crate::peer_loop::transaction_confirmer::confirm_transactions;

pub fn start_peer_set_thread(_: &mpsc::Sender<std::collections::HashMap<String, String>>, _: mpsc::Receiver<std::collections::HashMap<String, String>>) {
    {
        std::thread::spawn(move || {

                /*
                *
                *   SECTION <Add Initial Seeded Peers, Figure Out Our Latest Known Tick>
                *
                */
            
            
            let peer_set: Arc<Mutex<PeerSet>> = Arc::new(Mutex::new(PeerSet::new()));
            
            //Worker Loops
            monitor_latest_tick(peer_set.clone());
            confirm_transactions(peer_set.clone());
            broadcast_transactions(peer_set.clone());
            maintain_peers(peer_set.clone());
            handle_disconnected_peers(peer_set.clone());
            update_broadcast_computors(peer_set.clone());
            update_balances(peer_set.clone());
        });
    }
    
}