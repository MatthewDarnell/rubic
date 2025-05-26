pub mod header;
pub mod response;
pub mod transfer;
extern crate crypto;
extern crate identity;

use crypto::qubic_identities::get_public_key_from_identity;
use crate::header::{ EntityType, RequestResponseHeader };
use crate::transfer::TransferTransaction;

//Takes a public key
#[derive(Debug, Copy, Clone)]
pub struct RequestedEntity {
    pub public_key: [u8; 32]
}

#[derive(Debug, Clone)]
pub struct QubicApiPacket {
    pub api_type: EntityType,
    pub peer: Option<String>,
    pub header: RequestResponseHeader,
    pub data: Vec<u8>,
    pub response_data: Option<Vec<u8>>
}
impl QubicApiPacket {
    pub fn get_latest_tick() -> Self {
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestCurrentTickInfo);
        let size = std::mem::size_of::<RequestResponseHeader>();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::RequestCurrentTickInfo,
            peer: None,
            header: header,
            data: vec![],
            response_data: None
        }
    }

    pub fn get_identity_balance(id: &str) -> Self {
        //let entity: EntityType = EntityType::RequestEntity;
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestEntity);

        let data: Vec<u8> = get_public_key_from_identity(&String::from(id)).unwrap().to_vec();
        let size = std::mem::size_of::<RequestResponseHeader>() + data.len();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::RequestEntity,
            peer: None,
            header: header,
            data: data,
            response_data: None
        }
    }
    pub fn get_transaction(digest: &Vec<u8>) -> Self {
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestTransactionInfo);
        let size = std::mem::size_of::<RequestResponseHeader>() + digest.len();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::RequestTransactionInfo,
            peer: None,
            header: header,
            data: digest.clone(),
            response_data: None
        }
    }
    pub fn broadcast_transaction(transaction: &TransferTransaction) -> Self {
        //let entity: EntityType = EntityType::RequestEntity;
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::BroadcastTransaction);
        header.zero_dejavu();

        let data: Vec<u8> = transaction.as_bytes().to_vec();
        let size = std::mem::size_of::<RequestResponseHeader>() + data.len();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::BroadcastTransaction,
            peer: None,
            header: header,
            data: data,
            response_data: None
        }
    }
    
    pub fn request_tick_data(tick: u32) -> Self {
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestTickData);

        let data: Vec<u8> = tick.to_le_bytes().to_vec();
        let size = std::mem::size_of::<RequestResponseHeader>() + data.len();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::RequestTickData,
            peer: None,
            header: header,
            data: data,
            response_data: None
        }
    }
    
    pub fn as_bytes(&mut self) -> Vec<u8> {
        let mut res: Vec<u8> = self.header.as_bytes();
        res.append(&mut self.data);
        res
    }
    pub fn new(data: &Vec<u8>) -> Self {    //todo: remove this, should only be able to create by specific api call
        QubicApiPacket {
            api_type: EntityType::UNKNOWN,
            peer: None,
            header: RequestResponseHeader::new(),
            data: data.clone(),
            response_data: None
        }
    }
    pub fn format_response_from_bytes(peer_id: &String, data: Vec<u8>) -> Option<Self> {
        let header: RequestResponseHeader = RequestResponseHeader::from_vec(&data);
        //println!("RESPONSE: {:?}", &header.get_type());
        Some(QubicApiPacket {
            api_type: header.get_type().to_owned(),
            peer: Some(peer_id.to_owned()),
            header: header,
            data: data[8..].to_vec(),
            response_data: None
        })
    }
}

#[cfg(test)]
pub mod api_formatting_tests {
    use crate::header::EntityType;
    use crate::QubicApiPacket;
    #[test]
    fn create_identity_balance_request_entity() {
        let req = QubicApiPacket::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        assert_eq!(req.header._size[0], 40u8);
    }

    #[test]
    fn create_entity_get_full_request_as_bytes() {
        let mut req = QubicApiPacket::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        req.header.zero_dejavu();   //Dejavu is random 3 byte value
        let bytes = req.as_bytes();
        assert_eq!(bytes.len(), 40);
        assert_eq!(bytes.as_slice(),
                   vec![40, 0, 0, 31, 0, 0, 0, 0, 170, 135, 62, 76, 253, 55, 228, 191, 82, 138, 42, 160, 30, 236, 239, 54, 84, 124, 153, 202, 170, 189, 27, 189, 247, 37, 58, 101, 176, 65, 119, 26]
        );
    }

    #[test]
    fn create_request_Tick_data_packet() {
        let req = QubicApiPacket::request_tick_data(1000);
        let tick = u32::from_le_bytes([req.data[0], req.data[1], req.data[2], req.data[3]]);
        assert_eq!(req.header._type, EntityType::RequestTickData as u8);
        assert_eq!(tick, 1000);
    }
}