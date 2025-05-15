use api::transfer::TransferTransaction;
use std::collections::HashMap;
use std::sync::Mutex;
use logger::error;



fn transaction_monitor_thread() {
    
}
pub struct TransactionMonitor {
    //TODO: move to thread pool model
    txs: Mutex<HashMap<String, u32>>, //(txid, expiration tick)
    thread_handle: Option<std::thread::JoinHandle<()>>
}
impl TransactionMonitor {
    pub fn new() -> Self {
        TransactionMonitor {
            txs: Mutex::new(HashMap::new()),
            thread_handle: None
        }
    }
    
    pub fn add_tx(&mut self, tx: &TransferTransaction) {
        let txid = tx.txid();
        let expiration_tick = tx._tick;
        match self.txs.lock() {
            Ok(mut txs) => {
                if !txs.contains_key(&txid) {
                    txs.insert(txid, expiration_tick);   
                }
            },
            Err(_) => {
                error!("Failed to acquire lock for tx_monitor!");
            }
        }
        if !self.is_started() {
            self.thread_handle = Some(std::thread::spawn(transaction_monitor_thread));
        }
        
    }
    
    pub fn is_started(&self) -> bool { self.thread_handle.is_some() }
    pub fn stop(&self) -> bool { 
        match &self.thread_handle {
            Some(handle) => {
                
            },
            None => {}
        }
    }
}