use protocol::AsBytes;
use crate::header::{EntityType, RequestResponseHeader};

pub mod asset;

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
    pub fn request_issued_assets(issuer: Option<[u8; 32]>, asset_name: Option<String>) -> Self {
        let req = asset::RequestAssets::request_all_issued_assets(issuer, asset_name);
        let bytes = unsafe {
          req.by_filter.as_bytes()  
        };
        let mut header = RequestResponseHeader::new();
        header.set_type(EntityType::RequestAssets);
        let size = std::mem::size_of::<RequestResponseHeader>() + bytes.len();
        header.set_size(size);
        QubicApiPacket {
            api_type: EntityType::RequestAssets,
            peer: None,
            header,
            data: bytes,
            response_data: None
        }
    }
}