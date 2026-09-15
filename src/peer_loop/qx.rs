use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use logger::{debug, error};
use once_cell::sync::Lazy;
use smart_contract::qx::orderbook::AssetOrdersRequest;
use network::peers::{PeerSet, LOW_PRIORITY_BACKLOG};
use smart_contract::qx::QxFunctions;
use store::get_db_path;
use store::sqlite::tick::fetch_latest_tick;

/// Assets the UI is looking at, with when the order-book route last reported
/// each. The UI polls the route every second while the QX tab is open, so a
/// lease of a few seconds keeps an asset "viewed" across polls and lets it
/// lapse soon after the user moves on.
static VIEWED_ASSETS: Lazy<Mutex<HashMap<String, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Called by the order-book API route: the asset it served is what the UI shows.
pub fn request_priority_refresh(asset: &str) {
    if let Ok(mut viewed) = VIEWED_ASSETS.lock() {
        viewed.insert(asset.to_string(), Instant::now());
    }
}

// One pass every half second: the viewed assets plus one asset of the
// round-robin sweep. With ~150 issued assets that is a few requests per second
// instead of the ~100/s that asking for every book every 3 s produced, which
// flooded the peer request queue and delayed transaction broadcasts.
const PASS_INTERVAL: Duration = Duration::from_millis(500);
/// How long an asset stays "viewed" after the route last reported it.
const VIEW_LEASE: Duration = Duration::from_secs(10);
/// A viewed book only changes when the network processes a tick, so it is
/// re-fetched once per tick as the wallet learns of it: at most every half
/// second (a burst of ticks), and at least every few seconds (a stalled tick).
const VIEWED_MIN_INTERVAL: Duration = Duration::from_millis(500);
const VIEWED_MAX_INTERVAL: Duration = Duration::from_secs(3);
const VIEWED_PER_PASS: usize = 4;
const SWEEP_MIN_INTERVAL: Duration = Duration::from_millis(1000);
const ASSET_LIST_REFRESH: Duration = Duration::from_secs(60);

/// Asks peers for both sides of an asset's book. The book the UI is viewing goes
/// at high priority (ahead of balances, a couple of copies each so the idle peers
/// answer); the background sweep is low priority, so identity balances are
/// always served ahead of it.
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
        // Per asset: when its book was last requested, and the tick known then.
        let mut last_refresh: HashMap<String, (Instant, u32)> = HashMap::new();

        loop {
            std::thread::sleep(PASS_INTERVAL);
            let tick: u32 = fetch_latest_tick(get_db_path().as_str()).ok()
                .and_then(|t| t.parse::<u32>().ok())
                .unwrap_or(0);

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

            // Viewed: what the UI is showing, once per tick while the lease holds.
            let viewed: Vec<String> = VIEWED_ASSETS.lock().map(|mut v| {
                v.retain(|_, at| at.elapsed() < VIEW_LEASE);
                v.keys().cloned().collect()
            }).unwrap_or_default();
            let mut served = 0;
            for name in viewed {
                if served >= VIEWED_PER_PASS {
                    break;
                }
                let due = match last_refresh.get(&name) {
                    None => true,
                    Some((at, at_tick)) => at.elapsed() >= VIEWED_MIN_INTERVAL
                        && (tick > *at_tick || at.elapsed() >= VIEWED_MAX_INTERVAL),
                };
                if !due {
                    continue;
                }
                if let Some(issuer) = issuer_of(&name) {
                    request_book(&peer_set, &name, &issuer, false);
                    last_refresh.insert(name, (Instant::now(), tick));
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
            if last_refresh.get(&name).map_or(true, |(at, _)| at.elapsed() >= SWEEP_MIN_INTERVAL) {
                request_book(&peer_set, &name, &issuer, true);
                last_refresh.insert(name, (Instant::now(), tick));
            }
        }
    });
}
