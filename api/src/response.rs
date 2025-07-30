use std::collections::HashMap;
use std::ffi::CStr;
use std::ops::Index;
use std::ptr::copy_nonoverlapping;
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
use smart_contract::qx::orderbook::{AssetOrdersRequest, OrderBook};
use store::sqlite::asset::{asset_issuance};
use smart_contract::qx::asset::{IssuedAsset, OwnedAsset, PossessedAsset};

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
                                Ok(id) => {
                                    //println!("Created Asset Issuance! <{}> : <{}>", asset.asset.issuance.get_name().as_str(), id);
                                },
                                Err(_err) => {
                                    eprintln!("Failed to create asset_issuance! {:?}", _err);
                                }
                            }
                        }
                    },
                    None => {
                        println!("Failed to format IssuedAsset!");
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

                    let _name = CStr::from_bytes_until_nul(&asset.issuance.issuance.name);
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
                            if(issued_asset.is_empty()) {
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
                                            match store::sqlite::tick::fetch_tick(get_db_path().as_str(), resp.tick) {
                                                Ok(tick) => {
                                                    let transaction_digests_hash = tick.get(&"transaction_digests_hash".to_string()).unwrap();
                                                    if resp.validate_vs_tick_tx_digests_hash(transaction_digests_hash) {
                                                        //This Tick Data tx hash Matches The Verified Tx Digests Hash From Tick. We are all good to go!
                                                        match store::sqlite::tick::set_tick_transaction_digests(get_db_path().as_str(), resp.tick, &dg) {
                                                            Ok(_) => {
                                                                //println!("Set Tx Digests For Tick {}", resp.tick);
                                                            },
                                                            Err(_err) => {
                                                                println!("Failed to set Tick Transaction Digests for Tick {}!", resp.tick);
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(_err) => {
                                                    //eprintln!("Failed To Fetch Tick For TickData.({})", resp.tick);
                                                }
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
                    println!("Error Formatting Tick Data Response");
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
                                            store::sqlite::qx::orderbook::create_qx_orderbook(get_db_path().as_str(), asset_name.to_str().unwrap(), side, &_v).expect("Failed to create Orderbook");
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
            println!("Unknown Entity Type {:?}", response.api_type);
            //println!("{:?}", response);
        }
    }
    delete_request_from_matcher(response.header._dejavu, requests.clone());
}