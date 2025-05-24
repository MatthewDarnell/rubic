use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use crate::transfer::TransferTransaction;

pub type BroadcastTransactionEntity = TransferTransaction;

impl FormatQubicResponseDataToStructure for BroadcastTransactionEntity {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {
        match handle_broadcast_transaction_entity(response) {
            Some(v) => Some(v),
            None => None 
        }
    }
}


pub fn handle_broadcast_transaction_entity(response: &mut QubicApiPacket) -> Option<BroadcastTransactionEntity> {
    if response.header.as_bytes().len() + response.data.len() < 8 + 80 {
        return None;
    }
    Some(BroadcastTransactionEntity::from_bytes(response.data.as_slice()))
}