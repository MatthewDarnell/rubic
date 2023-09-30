use crate::qubic_api_t;
use crate::response::FormatQubicResponseDataToStructure;

#[derive(Debug, Clone)]
pub struct ExchangePeersEntity {
    pub peer: String
}

impl ExchangePeersEntity {
    pub fn new(peer: &str) -> ExchangePeersEntity {
        ExchangePeersEntity {
            peer: peer.to_string()
        }
    }
}

impl FormatQubicResponseDataToStructure for ExchangePeersEntity {
    fn format_qubic_response_data_to_structure(response: &mut qubic_api_t) -> Self {handle_exchange_peers(response).unwrap() /* check it's valid before calling! */ }
}

pub fn handle_exchange_peers(data: &mut qubic_api_t) -> Option<ExchangePeersEntity> {
    println!("Got Exchange Peer Data: {:?}", data);
    Some(ExchangePeersEntity::new(data.peer.as_ref().unwrap().as_str()))
}