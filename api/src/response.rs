use consensus::computor::BroadcastComputors;
use std::time::SystemTime;
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

pub mod exchange_peers;
pub mod response_entity;
pub mod broadcast_transaction;
pub mod request_tick_data;
mod tick;

pub trait FormatQubicResponseDataToStructure {
    fn format_qubic_response_data_to_structure(response: & mut QubicApiPacket) -> Option<Self> where Self: Sized;
}


pub fn get_formatted_response_from_multiple(response: &mut Vec<QubicApiPacket>) {
    let packet = response.first().unwrap();
    let peer = match &packet.peer {
        Some(peer) => peer.clone(),
        None => "".to_string(),
    };
    let api_type = response.first().unwrap().api_type;
    match api_type {
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
                        println!("Failed to format Tick!");
                    }
                };
            }
            
            let first_tick = tick_data.first().unwrap();
            let epoch = first_tick.epoch;
            let tick = first_tick.tick - 1;
            match store::sqlite::computors::fetch_computors_by_epoch(get_db_path().as_str(), epoch) {
                Ok(bytes) => {
                    let bc: BroadcastComputors = BroadcastComputors::new(&bytes);
                    match consensus::quorum_votes::get_quorum_votes(&bc, &tick_data) {
                        Ok(votes) => {
                            //println!("Quorum Votes For Epoch {} Validated - {}", epoch, votes);
                            if votes {
                                //In case we missed this tick, perhaps we weren't running when it executed
                                match store::sqlite::tick::insert_tick(get_db_path().as_str(), peer.as_str(), tick) {
                                    Ok(_) => {},
                                    Err(_) => {}
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
        _ => {}
    }
}

pub fn get_formatted_response(response: &mut QubicApiPacket) {
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
                        Ok(_) => {},
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
                    let _bc = store::sqlite::computors::fetch_computors_by_epoch(get_db_path().as_str(), resp.epoch).unwrap();
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
                            Err(err) => {
                                println!("Failed to fetch expired/broadcast/unknown_status transfers for Tick {}! <{}>", resp.tick, err);
                            }
                        }
                    } else {
                        //TODO: Blacklist peer? Why is he sending bogus data?
                        println!("Failed to Verify Tick Data");
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
        _ => { 
            //println!("Unknown Entity Type {:?}", response.api_type);
            //println!("{:?}", response);
        }
    }

}