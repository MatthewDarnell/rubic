use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use logger::{debug, error};
use once_cell::sync::Lazy;
use smart_contract::qx::orderbook::AssetOrdersRequest;
use network::peers::{PeerSet, LOW_PRIORITY_BACKLOG};
use smart_contract::qx::QxFunctions;
use store::get_db_path;

/// Assets the UI is looking at right now (the order-book route reports each
/// request). They are refreshed ahead of the slow sweep over every asset.
static PRIORITY_ASSETS: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));

/// Called by the order-book API route: the asset it served is what the UI shows.
pub fn request_priority_refresh(asset: &str) {
    if let Ok(mut queue) = PRIORITY_ASSETS.lock() {
        if !queue.iter().any(|a| a == asset) {
            queue.push_back(asset.to_string());
        }
    }
}

// One pass every half second: a handful of priority assets plus one asset of
// the round-robin sweep. With ~150 issued assets that is ~2-10 requests per
// second instead of the ~100/s that asking for every book every 3 s produced,
// which flooded the peer request queue and delayed transaction broadcasts.
const PASS_INTERVAL: Duration = Duration::from_millis(500);
const PRIORITY_MIN_INTERVAL: Duration = Duration::from_millis(1500);
const PRIORITY_PER_PASS: usize = 4;
const ASSET_LIST_REFRESH: Duration = Duration::from_secs(60);

/// Asks peers for both sides of an asset's book. The book the UI is viewing goes
/// at high priority (ahead of balances); the background sweep is low priority,
/// so identity balances are always served ahead of it.
fn request_book(peer_set: &Arc<Mutex<PeerSet>>, name: &str, issuer: &str, low_priority: bool) {
    for function in [QxFunctions::QxGetAssetBidOrder, QxFunctions::QxGetAssetAskOrder] {
        let request = api::QubicApiPacket::get_asset_qx_orders(&AssetOrdersRequest::new(function, name, issuer, 0));
        let mut lock = peer_set.lock().unwrap();
        let result = if low_priority { lock.make_request_low_priority(request) } else { lock.make_request_high_priority(request) };
        match result {
            Ok(_) => {},
            Err(err) => debug!("{}", err),
        }
    }
}

pub fn monitor_qx_orderbook(peer_set: Arc<Mutex<PeerSet>>) {
    std::thread::spawn(move || {
        let mut assets: Vec<(String, String)> = Vec::new(); // (name, issuer)
        let mut assets_loaded_at: Option<Instant> = None;
        let mut next_index: usize = 0;
        let mut last_refresh: HashMap<String, Instant> = HashMap::new();

        loop {
            std::thread::sleep(PASS_INTERVAL);

            if assets_loaded_at.map_or(true, |at| at.elapsed() >= ASSET_LIST_REFRESH) {
                match store::sqlite::asset::asset_issuance::fetch_issued_assets_with_data(get_db_path().as_str()) {
                    Ok(list) => {
                        assets = list.iter().filter_map(|a| {
                            Some((a.get(&"name".to_string())?.clone(), a.get(&"issuer".to_string())?.clone()))
                        }).collect();
                        assets_loaded_at = Some(Instant::now());
                    },
                    Err(err) => error!("Failed to load issued assets for the order-book monitor: {}", err),
                }
            }
            if assets.is_empty() {
                continue;
            }
            let issuer_of = |name: &str| assets.iter().find(|(n, _)| n == name).map(|(_, i)| i.clone());

            // Priority: what the UI is showing, at most every few seconds per asset.
            let mut served = 0;
            while served < PRIORITY_PER_PASS {
                let next = PRIORITY_ASSETS.lock().ok().and_then(|mut q| q.pop_front());
                let Some(name) = next else { break };
                let fresh = last_refresh.get(&name).map_or(false, |at| at.elapsed() < PRIORITY_MIN_INTERVAL);
                if fresh {
                    continue;
                }
                if let Some(issuer) = issuer_of(&name) {
                    request_book(&peer_set, &name, &issuer, false);
                    last_refresh.insert(name, Instant::now());
                    served += 1;
                }
            }

            // Sweep: one asset per pass, so every book is refreshed within a minute or
            // two - but only while the request queue is idle; balances come first.
            if peer_set.lock().unwrap().routine_backlog() >= LOW_PRIORITY_BACKLOG {
                continue;
            }
            if next_index >= assets.len() {
                next_index = 0;
            }
            let (name, issuer) = assets[next_index].clone();
            next_index += 1;
            if last_refresh.get(&name).map_or(true, |at| at.elapsed() >= PRIORITY_MIN_INTERVAL) {
                request_book(&peer_set, &name, &issuer, true);
                last_refresh.insert(name, Instant::now());
            }
        }
    });
}
