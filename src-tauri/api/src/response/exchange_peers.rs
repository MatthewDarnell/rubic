use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;


#[derive(Debug, Clone)]
pub struct ExchangePeersEntity {
    pub peer: String,
    pub ip_addresses: [[u8; 4]; 4]
}


impl ExchangePeersEntity {
    pub fn new(peer: &str, addresses: [u8; 16]) -> ExchangePeersEntity {
        let mut ip_addresses: [[u8; 4]; 4] = [[0u8; 4]; 4];
        for i in 0..4 {
            ip_addresses[i].copy_from_slice(&addresses[i*4..i*4 + 4]);
        }
        ExchangePeersEntity {
            peer: peer.to_string(),
            ip_addresses
        }
    }
}

impl FormatQubicResponseDataToStructure for ExchangePeersEntity {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_exchange_peers(response) /* check it's valid before calling! */ }
}

pub fn handle_exchange_peers(data: &mut QubicApiPacket) -> Option<ExchangePeersEntity> {
    Some(ExchangePeersEntity::new(data.peer.as_ref().unwrap().as_str(), <[u8; 16]>::try_from(data.data.as_slice()).unwrap()))
}