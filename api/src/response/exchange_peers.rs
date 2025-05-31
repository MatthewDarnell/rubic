use crate::QubicApiPacket;
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
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_exchange_peers(response) /* check it's valid before calling! */ }
}

pub fn handle_exchange_peers(data: &mut QubicApiPacket) -> Option<ExchangePeersEntity> {
    Some(ExchangePeersEntity::new(data.peer.as_ref().unwrap().as_str()))
}