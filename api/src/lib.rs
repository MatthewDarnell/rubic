pub mod header;
pub mod response;
pub mod transfer;
extern crate crypto;
extern crate identity;
extern crate core;

use identity::get_public_key_from_identity;
use crate::header::{ EntityType, RequestResponseHeader };

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
    pub fn get_identity_balance(id: &str) -> Self {
        //let entity: EntityType = EntityType::RequestEntity;
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestEntity);

        let data: Vec<u8> = get_public_key_from_identity(id).unwrap();
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
    use crate::QubicApiPacket;
    #[test]
    fn create_identity_balance_request_entity() {
        let req = QubicApiPacket::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        println!("{:?}", req);
    }

    #[test]
    fn create_entity_get_full_request_as_bytes() {
        let mut req = QubicApiPacket::get_identity_balance("EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON");
        req.header.zero_dejavu();   //Dejavu is random 3 byte value
        let bytes = req.as_bytes();
        assert_eq!(bytes.len(), 68);
        assert_eq!(bytes.as_slice(),
                   vec![68, 0, 0, 0, 0, 0, 0, 31, 69, 80, 89, 87, 68, 82, 69, 68, 78, 76, 72, 88, 79, 70, 89, 86, 71, 81, 85, 75, 80, 72, 74, 71, 79, 77, 80, 66, 83, 76, 68, 68, 71, 90, 68, 80, 75, 86, 81, 85, 77, 70, 88, 65, 73, 81, 89, 77, 90, 71, 69, 72, 80, 90, 84, 65, 65, 87, 79, 78]);
    }
}