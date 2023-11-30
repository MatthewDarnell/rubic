use std::time::SystemTime;
use crate::QubicApiPacket;
use crate::header::EntityType;
use crate::response::exchange_peers::ExchangePeersEntity;
use crate::response::response_entity::ResponseEntity;
use store::get_db_path;
use store::sqlite::crud::{create_response_entity, peer::update_peer_last_responded, insert_latest_tick};
pub mod exchange_peers;
pub mod response_entity;

pub trait FormatQubicResponseDataToStructure {
    fn format_qubic_response_data_to_structure(response: & mut QubicApiPacket) -> Self;
}

pub fn get_formatted_response(response: &mut QubicApiPacket) {
    let path = store::get_db_path();
    match response.api_type {
        EntityType::RespondCurrentTickInfo => {
            if let Some(peer_id) = &response.peer {
                if response.data.len() == 12 {
                    let mut data: [u8; 4] = [0; 4];
                    data[0] = response.data[4];
                    data[1] = response.data[5];
                    data[2] = response.data[6];
                    data[3] = response.data[7];
                    let value = u32::from_le_bytes(data);
                    match insert_latest_tick(get_db_path().as_str(), peer_id.as_str(), value) {
                        Ok(_) => {},
                        Err(err) => {}
                    }
                } else {
                    println!("Malformed Current Tick Response.");
                }
            }
        },
        EntityType::ExchangePeers => {
            let resp: ExchangePeersEntity = ExchangePeersEntity::format_qubic_response_data_to_structure(response);
            //println!("ExchangePeersEntity: {:?}", resp);
            match update_peer_last_responded(path.as_str(), resp.peer.as_str(), SystemTime::now()) {
                Ok(_) => {},
                Err(err) => { /* println!("Error Updating Peer {} Last Responded: {}", resp.peer.as_str(), err.as_str())*/ }
            }
        },
        EntityType::ResponseEntity => {
            let resp: ResponseEntity = ResponseEntity::format_qubic_response_data_to_structure(response);
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
        EntityType::ERROR => {
            let error_type = String::from_utf8(response.data.clone()).unwrap();
            if let Some(id) = &response.peer {
                store::sqlite::crud::peer::set_peer_disconnected(store::get_db_path().as_str(), id.as_str());
            }
            //panic!("exiting");
        },
        EntityType::BroadcastTransaction => {
            println!("GOT BROADCAST TRANSACTION RESPONSE : {:?}", response);
        }
        _ => {/*  println!("Unknown Entity Type"); */ }
    }
    /*
    if response.len() < 8 { //header size
        return ;
    }
    //todo: check header reported size against len of full data body
    let header = RequestResponseHeader::from_vec(response);
    let size = std::mem::size_of::<RequestResponseHeader>();
    if header.get_type().is_none() {
        println!("unknown type");
        return;
    }
    let resp_type = header.get_type().unwrap();
    let data_size = header.get_size() ;
    if data_size == 0 {
        return;
    }

    println!("Full Response: {:?}", response);
    let header_slice = &response.as_slice()[..8];
    let address_slice = &response.as_slice()[8..8+32];
     match resp_type {
        EntityType::ExchangePeers => {
           match exchange_peers::handle_exchange_peers(response) {
               Some(_) => {},
               None => println!("Error Formatting Exchange Peers!")
           }
        },
        EntityType::RequestEntity => { /* this isn't a response... */ },
        EntityType::ResponseEntity => {
            match response_entity::handle_response_entity(response) {
                Some(value) => {

                },
                None => println!("Error Formatting Response Entity!")
            }
        },
        _ => {
            None    //Unknown Response Type
        }
    };

    println!("Handling Response\nReading Response: {:?}", header);
    println!("Num Bytes Of Response: {}", response.len());
    println!("Response Type: {:?}", resp_type);
    println!("Data Size: {}", data_size);
    println!("Header: {:?}", header_slice);
*/

}