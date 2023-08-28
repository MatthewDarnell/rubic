use crate::qubic_api_t;
use crate::response::FormatQubicResponseDataToStructure;

#[derive(Debug, Copy, Clone)]
pub struct ExchangePeersEntity {
}

impl ExchangePeersEntity {
    pub fn new() -> ExchangePeersEntity {
        ExchangePeersEntity {}
    }
}

impl FormatQubicResponseDataToStructure for ExchangePeersEntity {
    fn format_qubic_response_data_to_structure(response: &mut qubic_api_t) -> Self {handle_exchange_peers(response).unwrap() /* check it's valid before calling! */ }
}

pub fn handle_exchange_peers(data: &mut qubic_api_t) -> Option<ExchangePeersEntity> {
    println!("Got Exchange Peer Data: {:?}", data);
    Some(ExchangePeersEntity::new())
}