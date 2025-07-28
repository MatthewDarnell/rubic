use crypto::qubic_identities::get_public_key_from_identity;
use protocol::AsBytes;
use std::io::Write;

#[allow(dead_code)]
enum RequestAssetType {
    RequestIssuanceRecords = 0,
    RequestOwnershipRecords = 1,
    RequestPossessionRecords = 2,
    RequestByUniverseIdx = 3
}

enum RequestAssetIssuanceFlags {
    AnyIssuer = 0b10,
    AnyAssetName = 0b100
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestAssetByFilter {
    asset_req_type: u16,
    flags: u16,
    ownership_managing_contract: u16,
    possession_managing_contract: u16,
    issuer: [u8; 32],
    asset_name: u64,
    owner: [u8; 32],
    possessor: [u8; 32]
}

impl RequestAssetByFilter {
    pub fn zeroed() -> Self {
        RequestAssetByFilter {
            asset_req_type: 0,
            flags: 0,
            ownership_managing_contract: 0,
            possession_managing_contract: 0,
            issuer: [0; 32],
            asset_name: 0,
            owner: [0; 32],
            possessor: [0; 32]
        }
    }
}

impl AsBytes for RequestAssetByFilter {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for k in self.asset_req_type.to_le_bytes().to_vec() {
            bytes.push(k);
        }
        for k in self.flags.to_le_bytes().to_vec() {
            bytes.push(k);
        }
        for k in self.ownership_managing_contract.to_le_bytes().to_vec() {
            bytes.push(k);
        }
        for k in self.possession_managing_contract.to_le_bytes().to_vec() {
            bytes.push(k);
        }
        for k in self.issuer.as_slice() {
            bytes.push(*k);
        }
        for k in self.asset_name.to_le_bytes().to_vec() {
            bytes.push(k);
        }
        for k in self.owner.as_slice() {
            bytes.push(*k);
        }
        for k in self.possessor.as_slice() {
            bytes.push(*k);
        }
        bytes
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RequestAssetByUniverseIdx {
    asset_req_type: u16,
    flags: u16,
    universe_idx: u32
}

#[repr(C)]
pub union RequestAssets {
    pub asset_req_type: u16,
    pub by_filter: RequestAssetByFilter,
    pub by_universe_idx: RequestAssetByUniverseIdx,
}
fn fill_from_str(mut bytes: &mut [u8], s: &[u8]) {
    bytes.write(s).unwrap();
}

impl RequestAssets {
    #[allow(unused_unsafe)]
    pub fn request_all_issued_assets(issuer: Option<[u8; 32]>, asset_name: Option<String>) -> Self {
        let mut req: RequestAssets = RequestAssets { asset_req_type: RequestAssetType::RequestIssuanceRecords as u16 };
        unsafe {
            req.by_filter.flags = RequestAssetIssuanceFlags::AnyIssuer as u16 | RequestAssetIssuanceFlags::AnyAssetName as u16;

            if let Some(pub_key_issuer) = issuer {
                req.by_filter.flags &= !(RequestAssetIssuanceFlags::AnyIssuer as u16);
                req.by_filter.issuer.copy_from_slice(&pub_key_issuer);
            }
            if let Some(asset) = asset_name {
                req.by_filter.flags &= !(RequestAssetIssuanceFlags::AnyAssetName as u16);
                let asset_name_bytes_unpadded = match asset.len() > 7 {
                    true => asset.split_at(7).0.as_bytes(),
                    false => asset.as_bytes()
                };
                let mut asset_name_bytes_padded: [u8; 8] = [0u8; 8];
                fill_from_str(&mut asset_name_bytes_padded, asset_name_bytes_unpadded);
                req.by_filter.asset_name = u64::from_le_bytes(asset_name_bytes_padded.try_into().unwrap());
            }
        }
        req
    }
}

#[test]
fn test_requested_entity_size() {
    assert_eq!(112, size_of::<RequestAssets>());
}

#[test]
fn test_request_all_issued_assets() {
    let s = RequestAssets::request_all_issued_assets(None, None);
    let bytes = vec![
        0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0]
    ;
    unsafe {
        assert_eq!(bytes.as_slice(), s.by_filter.as_bytes().as_slice());
        assert_eq!(6, s.by_filter.flags);
        assert_eq!(0, s.asset_req_type);
    }
}

#[test]
fn test_request_issued_assets_by_issuer_and_name() {
    let id = get_public_key_from_identity(&"EPYWDREDNLHXOFYVGQUKPHJGOMPBSLDDGZDPKVQUMFXAIQYMZGEHPZTAAWON".to_string()).unwrap();
    let s = RequestAssets::request_all_issued_assets(Some(id), Some(String::from("name")));
    unsafe {
        assert_eq!(0, s.by_filter.flags);
        assert_eq!(0, s.asset_req_type);
    }
}