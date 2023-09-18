use crate::{qubic_api_t, request_response_header};
use crate::header::entity_type;
use crate::response::exchange_peers::ExchangePeersEntity;
use crate::response::response_entity::ResponseEntity;
use store::sqlite::crud::{create_response_entity};
pub mod exchange_peers;
pub mod response_entity;

pub trait FormatQubicResponseDataToStructure {
    fn format_qubic_response_data_to_structure(response: & mut qubic_api_t) -> Self;
}

pub fn get_formatted_response(response: &mut qubic_api_t) {
    let path = store::get_db_path();
    println!("API MODULE GOT PATH {}", path.as_str());
    match response.api_type {
        entity_type::EXCHANGE_PEERS => {
            let resp: ExchangePeersEntity = ExchangePeersEntity::format_qubic_response_data_to_structure(response);
            println!("ExchangePeersEntity: {:?}", resp);

        },
        entity_type::RESPONSE_ENTITY => {
            let resp: ResponseEntity = ResponseEntity::format_qubic_response_data_to_structure(response);
            create_response_entity(path.as_str(),
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
            );
            println!("Inserted Response Entity!");
        },
        _ => {/*  println!("Unknown Entity Type"); */ }
    }
    /*
    if response.len() < 8 { //header size
        return ;
    }
    //todo: check header reported size against len of full data body
    let header = request_response_header::from_vec(response);
    let size = std::mem::size_of::<request_response_header>();
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
        entity_type::EXCHANGE_PEERS => {
           match exchange_peers::handle_exchange_peers(response) {
               Some(_) => {},
               None => println!("Error Formatting Exchange Peers!")
           }
        },
        entity_type::REQUEST_ENTITY => { /* this isn't a response... */ },
        entity_type::RESPONSE_ENTITY => {
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