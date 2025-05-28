use std::time::SystemTime;
use crate::QubicApiPacket;
use crate::header::EntityType;
use crate::response::exchange_peers::ExchangePeersEntity;
use crate::response::response_entity::ResponseEntity;
//use crate::response::broadcast_transaction::BroadcastTransactionEntity;

use store::get_db_path;
use store::sqlite::crud::{create_response_entity, peer::update_peer_last_responded, insert_latest_tick};
use crate::response::broadcast_transaction::BroadcastTransactionEntity;
use crate::response::request_tick_data::TickData;
use crate::response::tick::Tick;

pub mod exchange_peers;
pub mod response_entity;
pub mod broadcast_transaction;
pub mod request_tick_data;
mod tick;

pub trait FormatQubicResponseDataToStructure {
    fn format_qubic_response_data_to_structure(response: & mut QubicApiPacket) -> Option<Self> where Self: Sized;
}


pub fn get_formatted_response_from_multiple(response: &mut Vec<QubicApiPacket>) {
    let path = store::get_db_path();
    let api_type = response.first().unwrap().api_type;
    match api_type {
        EntityType::BroadcastTick => {
            println!("Broadcast Quorum Tick Data Response");
            let mut tick_data: Vec<Tick> = Vec::with_capacity(response.len());
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
            //println!("Tick Data: {:?}", &tick_data);
            //println!("\tAfter Mapping: First Data: \n\t{:?}", &tick_data.first().unwrap().print());

            //let _ = response.iter().map(|mut value|
              //  tick_data.push(TickData::format_qubic_response_data_to_structure(&mut value).unwrap())
            //);
            
            /*
            match TickData::format_qubic_response_data_to_structure(response) {
                Some(resp) => {
                    println!("{:?}", resp);
                },
                None => {
                    println!("Error Formatting Tick Data Response");
                }
            }
            */
        },
        _ => {}
    }
}

pub fn get_formatted_response(response: &mut QubicApiPacket) {
    let path = store::get_db_path();
    match response.api_type {
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
                    match insert_latest_tick(get_db_path().as_str(), peer_id.as_str(), value) {
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
            println!("Requesting Tick Data Response");
            match TickData::format_qubic_response_data_to_structure(response) {
                Some(resp) => {
                    resp.print();
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
                store::sqlite::crud::peer::set_peer_disconnected(store::get_db_path().as_str(), id.as_str()).ok();
            }
            //panic!("exiting");
        },
        EntityType::RequestTransactionInfo => {
            println!("GOT RequestTransactionInfo");
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