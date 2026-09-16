use std::collections::HashMap;
use std::ffi::CStr;
use std::ops::Index;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use consensus::computor::BroadcastComputors;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::QubicApiPacket;
use crate::header::EntityType;
use crate::response::exchange_peers::ExchangePeersEntity;
use crate::response::response_entity::ResponseEntity;
//use crate::response::broadcast_transaction::BroadcastTransactionEntity;

use store::get_db_path;
use store::sqlite::response_entity::create_response_entity;
use store::sqlite::peer::update_peer_last_responded;
use store::sqlite::tick::insert_tick;
use crate::response::broadcast_transaction::BroadcastTransactionEntity;
use consensus::tick::Tick;
use consensus::tick_data::{TickData, TransactionDigest};
use crypto::qubic_identities::get_identity;
use uuid::Uuid;
use logger::error;
use smart_contract::qx::orderbook::{AssetOrdersRequest, OrderBook};
use store::sqlite::asset::{asset_issuance};
use smart_contract::qx::asset::{IssuedAsset, PossessedAsset};

pub mod exchange_peers;
pub mod response_entity;
pub mod broadcast_transaction;
pub mod request_tick_data;
mod tick;
mod asset;
mod asset_order;

pub trait FormatQubicResponseDataToStructure {
    fn format_qubic_response_data_to_structure(response: & mut QubicApiPacket) -> Option<Self> where Self: Sized;
}


/// When each order book side was last stored from a peer reply, keyed by
/// (asset, side 'A'|'B'). Lets the UI show how fresh the book on screen is.
static BOOK_REFRESHED: std::sync::OnceLock<Mutex<HashMap<(String, String), std::time::Instant>>> = std::sync::OnceLock::new();

/// How far behind the network each peer was the last time it answered a tick
/// poll, keyed by peer id: `(ticks behind, measured at)`. A node that is out of
/// sync serves an out-of-date order book, so its books are not stored.
static PEER_LAG: std::sync::OnceLock<Mutex<HashMap<String, (u32, std::time::Instant)>>> = std::sync::OnceLock::new();
/// Order books from a peer more than this many ticks behind are dropped.
const MAX_BOOK_PEER_LAG: u32 = 10;
/// A lag measurement older than this no longer counts against a peer.
const PEER_LAG_MEASUREMENT_TTL: std::time::Duration = std::time::Duration::from_secs(60);

/// Records how far `peer`'s reported tick trails the highest tick the wallet
/// already knows. Both numbers are taken when the reply arrives, so a slow
/// poll does not make a healthy peer look behind.
fn note_peer_tick(peer: &str, reported: u32) {
    let latest = store::sqlite::tick::fetch_latest_tick(get_db_path().as_str()).ok()
        .and_then(|t| t.parse::<u32>().ok())
        .unwrap_or(0);
    let lag = latest.saturating_sub(reported);
    if let Ok(mut map) = PEER_LAG.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        map.insert(peer.to_string(), (lag, std::time::Instant::now()));
    }
}

/// Ticks `peer` was behind the network at its last tick reply, if that reply
/// is recent enough to count; `None` when the peer has not been measured.
pub fn peer_tick_lag(peer: &str) -> Option<u32> {
    PEER_LAG.get().and_then(|m| m.lock().ok())
        .and_then(|map| map.get(peer).copied())
        .filter(|(_, at)| at.elapsed() < PEER_LAG_MEASUREMENT_TTL)
        .map(|(lag, _)| lag)
}

/// Ticks `peer` was behind at its last recent tick reply; `0` when unmeasured.
fn peer_lag(peer: &str) -> u32 {
    peer_tick_lag(peer).unwrap_or(0)
}

fn note_book_refreshed(asset: &str, side: &str) {
    if let Ok(mut map) = BOOK_REFRESHED.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        map.insert((asset.to_string(), side.to_string()), std::time::Instant::now());
    }
}

/// Seconds since each side of `asset`'s book was last stored: `(ask, bid)`,
/// `None` when that side has never been received.
pub fn book_age_seconds(asset: &str) -> (Option<u64>, Option<u64>) {
    let map = match BOOK_REFRESHED.get().and_then(|m| m.lock().ok()) {
        Some(map) => map,
        None => return (None, None),
    };
    let age = |side: &str| map.get(&(asset.to_string(), side.to_string())).map(|at| at.elapsed().as_secs());
    (age("A"), age("B"))
}

fn delete_request_from_matcher(dejavu: u32, requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>) {
    match requests.lock() {
        Ok(mut guard) => { guard.remove(&dejavu); },
        Err(_err) => {}
    }
}

pub fn get_formatted_response_from_multiple(requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, response: &mut Vec<QubicApiPacket>) {
    let packet = response.first().unwrap();
    let peer = match &packet.peer {
        Some(peer) => peer.clone(),
        None => "".to_string(),
    };
    let api_type = response.first().unwrap().api_type;
    let deja_vu = response.first().unwrap().header._dejavu;
    match api_type {
        EntityType::RespondAssets => {
            for mut asset in response {
                match IssuedAsset::format_qubic_response_data_to_structure(&mut asset) {
                    Some(asset) => {
                        unsafe {
                            /*
                            let contract_index = asset.asset.possession.issuance_index;
                            let managing_index = asset.asset.possession.managing_contract_index;
                            
                            println!("Asset: {}: Contract Index.({}) - Managing Index.({})", asset.asset.issuance.get_name().as_str(), contract_index, managing_index);
                            */
                            
                            match asset_issuance::create_asset_issuance(
                                get_db_path().as_str(),
                                peer.as_str(),
                                get_identity(& asset.asset.issuance.pub_key).as_str(),
                                asset.asset.issuance._type,
                                asset.asset.issuance.get_name().as_str(),
                                asset.asset.issuance.number_of_decimal_places,
                                asset.asset.issuance.pad_unit_of_measurement_to_u64(),
                            ) {
                                Ok(_id) => {
                                    //println!("Created Asset Issuance! <{}> : <{}>", asset.asset.issuance.get_name().as_str(), id);
                                },
                                Err(_err) => {
                                    eprintln!("Failed to create asset_issuance! {:?}", _err);
                                }
                            }
                        }
                    },
                    None => {
                        //println!("Failed to format IssuedAsset!");
                    }
                };
            }
        },
        EntityType::RespondOwnedAssets => {},   //TODO
        EntityType::RespondPossessedAssets => {
            let mut assets_data: Vec<PossessedAsset> = Vec::with_capacity(response.len());
            for entry in response.iter_mut() {
                match PossessedAsset::format_qubic_response_data_to_structure(entry) {
                    Some(data) => {
                        assets_data.push(data)
                    },
                    None => {
                        println!("Failed to format PossessedAsset!");
                    }
                };
            }
            for (index, asset) in assets_data.iter().enumerate() {
                unsafe {
                    let _siblings = crypto::encoding::bytes_to_hex(&asset.siblings.as_flattened().to_vec());
                    let _peer = &response.index(index).peer.clone().unwrap();
                    let mut _temp_name: [u8; 8] = [0u8; 8];
                    _temp_name[0..7].copy_from_slice(&asset.issuance.issuance.name);
                    let _name = CStr::from_bytes_until_nul(&_temp_name);
                    if _name.is_err() {
                        eprintln!("Failed To Parse AssetRecord Issuance Name: {:?}", asset.issuance.issuance.name);
                        continue;
                    }
                    
                    let name = _name.unwrap().to_str().unwrap();
                    match asset_issuance::fetch_issued_asset(
                        get_db_path().as_str(),
                        name.to_string().as_str(),
                        get_identity(& asset.issuance.issuance.pub_key).as_str(),
                    ) {
                        Ok(issued_asset) => {
                            if issued_asset.is_empty() {
                                println!("failed to insert Possessed Asset For Unknown Issuance {}", name);
                                continue;
                            }
                            let _id = issued_asset.get(&"id".to_string()).unwrap();
                            let id = u64::from_str(_id).unwrap();
                            let _issuance = &asset.issuance.issuance;
                            let _possession = &asset.asset.possession;
                            //println!("Got Possession For {}", get_identity(&_possession.pub_key).as_str());
                            match store::sqlite::asset::asset_record::create_asset_possession(
                                get_db_path().as_str(),
                                id,
                                get_identity(&_possession.pub_key).as_str(),
                                _possession.managing_contract_index,
                                _possession.issuance_index,
                                _possession.number_of_shares as u64,
                                asset.tick
                            ) {
                                Ok(_) => {},
                                Err(_err) => { eprintln!("Failed to store asset Possession! {:?}", _err); }
                            }
                        },
                        Err(_err) => {
                            eprintln!("Failed to create PossessedAsset! {:?}", _err);
                        }
                    }
                }
            }
        },
        EntityType::BroadcastTick => {
            let mut tick_data: Vec<Tick> = Vec::with_capacity(response.len());
            if tick_data.len() > 0 {
                println!("Received Quorum Tick {}", &tick_data[0].tick);
            } else {
                //println!("Got 0 Length Quorum Tick...");
            }
            for entry in response.iter_mut() {
                match Tick::format_qubic_response_data_to_structure(entry) {
                    Some(data) => {
                        tick_data.push(data)
                    },
                    None => {
                        //println!("Failed to format Tick!");
                    }
                };
            }
            
            let first_tick = tick_data.first().unwrap();
            let epoch = first_tick.epoch;
            let tick = first_tick.tick;
            let tx_digest = &first_tick.transaction_digest;
            
            let tx_digest_hash = get_identity(tx_digest);
            match store::sqlite::computors::fetch_computors_by_epoch(get_db_path().as_str(), epoch) {
                Ok(bytes) => {
                    let bc: BroadcastComputors = BroadcastComputors::new(&bytes);
                    match consensus::quorum_votes::get_quorum_votes(&bc, &tick_data) {
                        Ok(votes) => {
                            //println!("Quorum Votes For Epoch {} Validated - {}", epoch, votes);
                            if votes {
                                //In case we missed this tick, perhaps we weren't running when it executed
                                match store::sqlite::tick::insert_tick(get_db_path().as_str(), peer.as_str(), tick) {
                                    Ok(_) => {
                                        match store::sqlite::tick::set_tick_tx_digest_hash(get_db_path().as_str(), &tx_digest_hash, tick) {
                                            Ok(_) => {},
                                            Err(_err) => {
                                                eprintln!("Failed To Set Transaction Digest For Tick.({})\n\t({})\n", tick, _err);
                                            }
                                        }
                                    },
                                    Err(_) => {
                                        eprintln!("Failed To Insert Tick.({})\n", tick);
                                    }
                                }
                                match store::sqlite::tick::set_tick_validated(get_db_path().as_str(), tick) {
                                    Ok(_) => { 
                                        //println!("Setting Tick.({}) Valid", tick);
                                    },
                                    Err(err) => println!("Failed to set Tick.({}) Validated: {}", tick, err)
                                }
                            }
                        },
                        Err(err) => {
                            println!("Error Validating Quorum Votes for Tick {}! <{}>", tick, err);
                        }
                    }
                },
                Err(err) => {
                    println!("Failed to fetch computor by epoch: {}", err);
                }

            }
        },
        _ => {
            println!("Got Response Multiple Type {:?}", response.first().unwrap().api_type);
        }
    }
    delete_request_from_matcher(deja_vu, requests.clone());
}

pub fn get_formatted_response(requests: Arc<Mutex<HashMap<u32, QubicApiPacket>>>, response: &mut QubicApiPacket) {
    let path = store::get_db_path();
    match response.api_type {
        EntityType::BroadcastComputors => {
            match response.peer.clone() {
                Some(peer) => {
                    if response.data.len() == std::mem::size_of::<BroadcastComputors>() {
                        let data: [u8; size_of::<BroadcastComputors>()] = response.data.as_slice().try_into().unwrap();
                        let bc: BroadcastComputors = BroadcastComputors::new(&data);  
                        if bc.validate() {
                            match store::sqlite::computors::insert_computors_from_bytes(get_db_path().as_str(), peer.as_str(), &response.data) {
                                Ok(_) => {
                                    //println!("Updating Computor List for Epoch {}.", bc.epoch);
                                },
                                Err(_) => {}
                            }
                        } else {
                            println!("Failed to Validate Computor List for Epoch {}!", bc.epoch);
                        }
                    }
                },
                None => {}
            }
        },
        EntityType::RespondCurrentTickInfo => {
            if let Some(peer_id) = &response.peer {
                if response.data.len() < 12 {
                    println!("Malformed Current Tick Response.");
                } else {
                    let mut data: [u8; 4] = [0; 4];
                    data[0] = response.data[4];
                    data[1] = response.data[5];
                    data[2] = response.data[6];
                    data[3] = response.data[7];
                    let value = u32::from_le_bytes(data);
                    note_peer_tick(peer_id.as_str(), value);
                    match insert_tick(get_db_path().as_str(), peer_id.as_str(), value) {
                        Ok(_) => {},
                        Err(_err) => {}
                    }
                }
            }
        },
        EntityType::ExchangePeers => {
            match ExchangePeersEntity::format_qubic_response_data_to_structure(response) {
                Some(resp) => {
                    //println!("ExchangePeersEntity: {:?}", resp);
                    match update_peer_last_responded(path.as_str(), resp.peer.as_str(), SystemTime::now()) {
                        Ok(_) => {
                            for i in resp.ip_addresses {
                                let address: String = format!("{}.{}.{}.{}:21841", i[0], i[1], i[2], i[3]);
                                //println!("Adding Peer to db {}", address.as_str());
                                match std::net::SocketAddrV4::from_str(address.as_str()) {
                                    Ok(_) => {
                                        match store::sqlite::peer::create_peer(
                                            get_db_path().as_str(),
                                            Uuid::new_v4().to_string().as_str(),
                                            address.as_str(),
                                            "",
                                            9999,
                                            false,
                                            UNIX_EPOCH
                                        ) {
                                            Ok(_) => {},
                                            Err(_err) => {
                                                println!("Failed To Create Peer From ExchangePublicPeers: {}", _err);
                                            }
                                        }
                                    },
                                    Err(_) => {}
                                }
                            }
                        },
                        Err(_err) => { /* println!("Error Updating Peer {} Last Responded: {}", resp.peer.as_str(), err.as_str())*/ }
                    }
                },
                None => {
                    
                }
            }
        },
        EntityType::BroadcastFutureTickData => { 
            //println!("{:?}", response);
            match TickData::format_qubic_response_data_to_structure(response) {
                Some(mut resp) => {
                    match store::sqlite::computors::fetch_computors_by_epoch(get_db_path().as_str(), resp.epoch) {
                        Ok(_bc) => {
                            let bc = BroadcastComputors::new(&_bc);
                            let verified = resp.validate(&bc);
                            if verified {
                                match store::sqlite::transfer::fetch_expired_and_broadcasted_transfers_with_unknown_status_and_specific_tick(get_db_path().as_str(), resp.tick) {
                                    Ok(transfers) => {
                                        if transfers.len() > 0 {    //We have made at least 1 transfer that executes on this tick!
                                            //Let's store the tx digests
                                            let digests: &[TransactionDigest] = resp.transaction_digests.as_slice();
                                            let mut dg: [u8; size_of::<TransactionDigest>()*1024] = [0u8; size_of::<TransactionDigest>()*1024];
                                            for (index, digest) in digests.iter().enumerate() {
                                                dg[index*size_of::<TransactionDigest>()..index*size_of::<TransactionDigest>() + size_of::<TransactionDigest>()].copy_from_slice(digest);
                                            }
                                            // The tick data is signed by the tick's leader computor (verified
                                            // above against the quorum-signed computor list). Public nodes no
                                            // longer answer quorum-tick requests, so that signature is what a
                                            // wallet has to go on; when quorum votes did arrive (gossip), the
                                            // data must also match their transaction digest hash.
                                            let quorum_hash: Option<String> = store::sqlite::tick::fetch_tick(get_db_path().as_str(), resp.tick)
                                                .ok()
                                                .and_then(|t| t.get(&"transaction_digests_hash".to_string()).cloned())
                                                .filter(|h| h.len() >= 8);
                                            let accepted = match &quorum_hash {
                                                Some(hash) => resp.validate_vs_tick_tx_digests_hash(hash),
                                                None => true,
                                            };
                                            if accepted {
                                                let peer_id = response.peer.clone().unwrap_or_default();
                                                let _ = insert_tick(get_db_path().as_str(), peer_id.as_str(), resp.tick);
                                                if quorum_hash.is_none() {
                                                    let data_hash = get_identity(&resp.hash_with_signature_bytes());
                                                    if let Err(err) = store::sqlite::tick::set_tick_tx_digest_hash(get_db_path().as_str(), &data_hash, resp.tick) {
                                                        eprintln!("Failed To Set Transaction Digest Hash For Tick.({}): {}", resp.tick, err);
                                                    }
                                                }
                                                match store::sqlite::tick::set_tick_transaction_digests(get_db_path().as_str(), resp.tick, &dg) {
                                                    Ok(_) => {
                                                        match store::sqlite::tick::set_tick_validated(get_db_path().as_str(), resp.tick) {
                                                            Ok(_) => println!("Tick {} data accepted ({}).", resp.tick, if quorum_hash.is_some() { "matches quorum votes" } else { "leader-signed" }),
                                                            Err(err) => println!("Failed to set Tick.({}) Validated: {}", resp.tick, err),
                                                        }
                                                    },
                                                    Err(_err) => {
                                                        println!("Failed to set Tick Transaction Digests for Tick {}!", resp.tick);
                                                    }
                                                }
                                            } else {
                                                println!("Tick data for {} does not match the quorum transaction digest hash; ignoring.", resp.tick);
                                            }
                                        }
                                    },
                                    Err(_err) => {
                                        //println!("Failed to fetch expired/broadcast/unknown_status transfers for Tick {}! <{}>", resp.tick, _err);
                                    }
                                }
                            } else {
                                //TODO: Blacklist peer? Why is he sending bogus data?
                                println!("Failed to Verify Tick Data");
                            }
                        },
                        Err(_err) => {
                            eprintln!("Failed To Fetch Computors From Db. ({})", _err);
                        }
                    }
                },
                None => {  
                    //println!("Error Formatting Tick Data Response");
                }
            }
        },
        EntityType::ResponseEntity => {
            match ResponseEntity::format_qubic_response_data_to_structure(response) {
                Some(resp) => {
                    //println!("Got ResponseEntity: {:?}", &resp);
                    match create_response_entity(path.as_str(),
                                                 resp.peer.as_str(),
                                                 resp.identity.as_str(),
                                                 resp.incoming,
                                                 resp.outgoing,
                                                 resp.final_balance,
                                                 resp.number_incoming_transactions,
                                                 resp.number_outgoing_transactions,
                                                 resp.latest_incoming_transfer_tick,
                                                 resp.latest_outgoing_transfer_tick,
                                                 resp.tick,
                                                 resp.spectrum_index
                    ) {
                        Ok(_) => {
                            update_peer_last_responded(path.as_str(), resp.peer.as_str(), SystemTime::now()).ok();
                        },
                        Err(err) => {
                            println!("Failed To Insert Response Entity: {}", err);
                        }
                    }
                },
                None => {}
            }
        },
        EntityType::ERROR => {
            let _error_type = String::from_utf8(response.data.clone()).unwrap();
            if let Some(id) = &response.peer {
                store::sqlite::peer::set_peer_disconnected(store::get_db_path().as_str(), id.as_str()).ok();
            }
            //panic!("exiting");
        },
        EntityType::BroadcastTransaction => {
            match BroadcastTransactionEntity::format_qubic_response_data_to_structure(response) {
                Some(_) => {
                    //TODO: Insert this tx into db and update status as succeeded
                },
                None => {}
            }
        },
        EntityType::RespondContractFunction => {
            // Which QX function was asked decides how the reply is laid out. The
            // request is looked up by dejavu; copy it out so the matcher lock is
            // not held across database writes.
            let request_data: Option<Vec<u8>> = requests.lock().ok()
                .and_then(|guard| guard.get(&response.header._dejavu).map(|r| r.data.clone()));
            let asked = request_data.as_ref()
                .filter(|d| d.len() >= 8)
                .map(|d| smart_contract::qx::orderbook::RequestContractFunction::from_bytes(d));
            let function = asked.as_ref().map(|r| r.input_type);
            if asked.as_ref().map(|r| r.contract_index == miner::random::RANDOM_CONTRACT_INDEX && r.input_type == miner::random::GET_PROVIDER_STATUS).unwrap_or(false) {
                // RANDOM GetProviderStatus: the identity asked about is the request input.
                let data = request_data.unwrap();
                if data.len() >= 40 {
                    let identity = get_identity(&<[u8; 32]>::try_from(&data[8..40]).unwrap());
                    match miner::random::parse_provider_status(&response.data) {
                        Some(slots) => {
                            // The report reflects the answering peer's state, which may trail
                            // the network: date it by that peer's tick, not the wallet's.
                            let latest: u32 = store::sqlite::tick::fetch_latest_tick(get_db_path().as_str()).ok().and_then(|t| t.parse().ok()).unwrap_or(0);
                            let checked_tick = latest.saturating_sub(response.peer.as_deref().map(peer_lag).unwrap_or(0));
                            if let Err(err) = store::sqlite::random_session::set_provider_status(get_db_path().as_str(), &identity, checked_tick, &miner::random::ProviderSlot::list_to_text(&slots)) {
                                error(format!("Failed To Store RANDOM Provider Status: {}", err).as_str());
                            }
                        },
                        None => println!("Failed To Read RANDOM Provider Status ({} bytes)!", response.data.len()),
                    }
                }
                delete_request_from_matcher(response.header._dejavu, requests.clone());
                return;
            }
            if matches!(function, Some(4) | Some(5)) {
                // EntityAskOrders / EntityBidOrders: one identity's resting orders.
                let data = request_data.unwrap();
                let entity_request = smart_contract::qx::entity_orders::EntityOrdersRequest::from_bytes(&data);
                let side = match entity_request.side() { Some("ASK") => "A", Some("BID") => "B", _ => "" };
                match smart_contract::qx::entity_orders::parse_entity_orders(&response.data) {
                    Some(orders) if !side.is_empty() => {
                        let identity = get_identity(&entity_request.input.entity);
                        if let Err(err) = store::sqlite::qx::entity_orders::replace_entity_orders(get_db_path().as_str(), &identity, side, &orders) {
                            error(format!("Failed To Store Entity Orders!: {}", err).as_str());
                        }
                    },
                    Some(_) => {},
                    None => println!("Failed To Read Entity Orders ({} bytes)!", response.data.len()),
                }
                delete_request_from_matcher(response.header._dejavu, requests.clone());
                return;
            }
            // A peer that trails the network serves a book the network has moved
            // past; storing it would overwrite a current one with stale rows.
            let lag = response.peer.as_deref().map(peer_lag).unwrap_or(0);
            if lag > MAX_BOOK_PEER_LAG {
                logger::debug(format!("Ignoring an order book from peer {} ({} ticks behind the network)", response.peer.clone().unwrap_or_default(), lag).as_str());
                // The request stays in the matcher: the other queued copy of it
                // may still be answered by a peer that is up to date.
                return;
            }
            //todo: as we implement more contracts, this might not be just for Qx Orderbook
            match OrderBook::format_qubic_response_data_to_structure(response) {
                Some(_v) => {
                    match requests.lock() {
                        Ok(guard) => {
                            if let Some(request) = guard.get(&response.header._dejavu) {
                                let asset_orders_request: AssetOrdersRequest = AssetOrdersRequest::from_bytes(request.data.as_slice());
                                let side = match asset_orders_request.get_orderbook_side() {
                                    "ASK" => "A",
                                    "BID" => "B",
                                    _ => "UNKNOWN"
                                };
                                if side != "UNKNOWN" {  //some other request, why did we match on this
                                    let a_bytes = asset_orders_request.input.asset_name.to_le_bytes();
                                    match CStr::from_bytes_until_nul(&a_bytes) {
                                        Ok(asset_name) => {
                                            match store::sqlite::qx::orderbook::create_qx_orderbook(get_db_path().as_str(), asset_name.to_str().unwrap(), side, &_v) {
                                                Ok(_) => {
                                                    note_book_refreshed(asset_name.to_str().unwrap_or(""), side);
                                                },
                                                Err(_err) => error(format!("Failed To Create OrderBook!: {}", _err).as_str())
                                                
                                            }
                                        },
                                        Err(_) => {}
                                    }   
                                }
                            } else {
                                //println!("Requests Tracker Missing Request {}", &response.header._dejavu);
                            }
                        },
                        Err(_err) => {
                            //println!("Failed To Get Order book Mutex Lock");
                        }
                    }
                },
                None => println!("Failed To Read Orderbook!")
            }
        },
        EntityType::ResponseEnd => {},
        _ => { 
            //println!("Unknown Entity Type {:?}", response.api_type);
            //println!("{:?}", response);
        }
    }
    delete_request_from_matcher(response.header._dejavu, requests.clone());
}