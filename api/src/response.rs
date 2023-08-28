use crate::{qubic_api_t, request_response_header};
use crate::header::entity_type;
pub mod exchange_peers;
pub mod response_entity;


pub fn handle_response(response: &Vec<u8>) -> Option<qubic_api_t> {
    let mut return_value = None;

    let header = request_response_header::from_vec(response);
    let size = std::mem::size_of::<request_response_header>();
    if header.get_type().is_none() {
        println!("unknown type");
        return return_value;
    }
    let resp_type = header.get_type().unwrap();
    let data_size = header.get_size() ;
    if data_size == 0 {
        return return_value;
    }

    println!("Full Response: {:?}", response);
    let header_slice = &response.as_slice()[..8];
    let address_slice = &response.as_slice()[8..8+32];
    match resp_type {
        entity_type::EXCHANGE_PEERS => { exchange_peers::handle_exchange_peers(response)},
        entity_type::REQUEST_ENTITY => {
            println!("request entity");

        },
        entity_type::RESPONSE_ENTITY => response_entity::handle_response_entity(response),
        _ => {
            println!("unknown type");
        }
    }

    println!("Handling Response\nReading Response: {:?}", header);
    println!("Num Bytes Of Response: {}", response.len());

    println!("Response Type: {:?}", resp_type);
    println!("Data Size: {}", data_size);

    println!("Header: {:?}", header_slice);
    return return_value;
}