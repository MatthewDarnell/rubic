use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;

const ASSETS_DEPTH: usize = 24;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Issuance {
    pub pub_key: [u8; 32],
    pub _type: u8,
    pub name: [u8; 7],
    pub number_of_decimal_places: u8,
    pub unit_of_measurement: [u8; 7]
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Ownership {
    pub pub_key: [u8; 32],
    pub _type: u8,
    pub padding: u8,
    pub managing_contract_index: u16,
    pub issuance_index: u32,
    pub number_of_shares: u64
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Possession {
    pub pub_key: [u8; 32],
    pub _type: u8,
    pub padding: u8,
    pub managing_contract_index: u16,
    pub issuance_index: u32,
    pub number_of_shares: u64
}

impl Issuance {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Issuance {
            pub_key: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            _type: bytes[32],
            name: <[u8; 7]>::try_from(&bytes[33..40]).unwrap(),
            number_of_decimal_places: bytes[40],
            unit_of_measurement:<[u8; 7]>::try_from(&bytes[40..47]).unwrap()
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(size_of::<Issuance>());
        vec.resize(size_of::<Issuance>(), 0);
        vec[0..32].copy_from_slice(self.pub_key.as_ref());
        vec[32] = self._type;
        vec[33..40].copy_from_slice(self.name.as_ref());
        vec[40] = self.number_of_decimal_places;
        vec[41..48].copy_from_slice(self.unit_of_measurement.as_ref());
        vec
    }
}

impl Ownership {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Ownership {
            pub_key: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            _type: bytes[32],
            padding: bytes[33],
            managing_contract_index: u16::from_le_bytes([bytes[34], bytes[35]]),
            issuance_index: u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]),
            number_of_shares: u64::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43], bytes[44], bytes[45], bytes[46], bytes[47]])
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(size_of::<Ownership>());
        vec.resize(size_of::<Ownership>(), 0);
        vec[0..32].copy_from_slice(self.pub_key.as_ref());
        vec[32] = self._type;
        vec[33] = self.padding;
        vec[34..36].copy_from_slice(&<[u8; 2]>::try_from(self.managing_contract_index.to_le_bytes()).unwrap());
        vec[36..40].copy_from_slice(&<[u8; 4]>::try_from(self.issuance_index.to_le_bytes()).unwrap());
        vec[40..48].copy_from_slice(&<[u8; 8]>::try_from(self.number_of_shares.to_le_bytes()).unwrap());
        vec
    }
}

impl Possession {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Possession {
            pub_key: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            _type: bytes[32],
            padding: bytes[33],
            managing_contract_index: u16::from_le_bytes([bytes[34], bytes[35]]),
            issuance_index: u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]),
            number_of_shares: u64::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43], bytes[44], bytes[45], bytes[46], bytes[47]])
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(size_of::<Possession>());
        vec.resize(size_of::<Possession>(), 0);
        vec[0..32].copy_from_slice(self.pub_key.as_ref());
        vec[32] = self._type;
        vec[33] = self.padding;
        vec[34..36].copy_from_slice(&<[u8; 2]>::try_from(self.managing_contract_index.to_le_bytes()).unwrap());
        vec[36..40].copy_from_slice(&<[u8; 4]>::try_from(self.issuance_index.to_le_bytes()).unwrap());
        vec[40..48].copy_from_slice(&<[u8; 8]>::try_from(self.number_of_shares.to_le_bytes()).unwrap());
        vec
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union AssetRecord {
    pub issuance: Issuance,
    pub ownership: Ownership,
    pub possession: Possession,
}

#[derive( Copy, Clone)]
pub struct IssuedAsset {
    pub asset: AssetRecord,
    pub tick: u32,
    pub universe_index: u32,
    pub siblings: [[u8; ASSETS_DEPTH]; 32]
}

#[derive( Copy, Clone)]
pub struct OwnedAsset {
    pub asset: AssetRecord,
    pub issuance: AssetRecord,
    pub tick: u32,
    pub universe_index: u32,
    pub siblings: [[u8; ASSETS_DEPTH]; 32]
}

#[derive(Copy, Clone)]
pub struct PossessedAsset {
    pub asset: AssetRecord,
    pub ownership: AssetRecord,
    pub issuance: AssetRecord,
    pub tick: u32,
    pub universe_index: u32,
    pub siblings: [[u8; ASSETS_DEPTH]; 32]
}

impl IssuedAsset {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (_issuance, right) = bytes.split_at(size_of::<AssetRecord>());
        let (_tick, right) = right.split_at(size_of::<u32>());
        let (_universe_index, _siblings) = right.split_at(size_of::<u32>());
        let issuance: Issuance = Issuance::from_bytes(&_issuance);
        let siblings: Vec<[u8; ASSETS_DEPTH]> = _siblings.chunks_exact(ASSETS_DEPTH).map(|chunk| <[u8; ASSETS_DEPTH]>::try_from(chunk).unwrap()).collect();
        IssuedAsset {
            asset: AssetRecord { issuance },
            tick: u32::from_le_bytes(<[u8; 4]>::try_from(_tick).unwrap()),
            universe_index: u32::from_le_bytes(<[u8; 4]>::try_from(_universe_index).unwrap()),
            siblings: siblings.try_into().unwrap()
        }

    }
}

impl OwnedAsset {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (_asset, right) = bytes.split_at(size_of::<AssetRecord>());
        let (_issuance, right) = right.split_at(size_of::<AssetRecord>());
        let (_tick, right) = right.split_at(size_of::<u32>());
        let (_universe_index, _siblings) = right.split_at(size_of::<u32>());
        let ownership: Ownership = Ownership::from_bytes(&_asset);
        let issuance: Issuance = Issuance::from_bytes(&_issuance);
        let siblings: Vec<[u8; ASSETS_DEPTH]> = _siblings.chunks_exact(ASSETS_DEPTH).map(|chunk| <[u8; ASSETS_DEPTH]>::try_from(chunk).unwrap()).collect();
        OwnedAsset {
            asset: AssetRecord { ownership },
            issuance: AssetRecord { issuance },
            tick: u32::from_le_bytes(<[u8; 4]>::try_from(_tick).unwrap()),
            universe_index: u32::from_le_bytes(<[u8; 4]>::try_from(_universe_index).unwrap()),
            siblings: siblings.try_into().unwrap()
        }

    }
}

impl PossessedAsset {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (_asset, right) = bytes.split_at(size_of::<AssetRecord>());
        let (_ownership, right) = right.split_at(size_of::<AssetRecord>());
        let (_issuance, right) = right.split_at(size_of::<AssetRecord>());
        let (_tick, right) = right.split_at(size_of::<u32>());
        let (_universe_index, _siblings) = right.split_at(size_of::<u32>());
        let ownership: Ownership = Ownership::from_bytes(&_asset);
        let issuance: Issuance = Issuance::from_bytes(&_issuance);
        let siblings: Vec<[u8; ASSETS_DEPTH]> = _siblings.chunks_exact(ASSETS_DEPTH).map(|chunk| <[u8; ASSETS_DEPTH]>::try_from(chunk).unwrap()).collect();
        PossessedAsset {
            asset: AssetRecord { ownership },
            ownership: AssetRecord { ownership },
            issuance: AssetRecord { issuance },
            tick: u32::from_le_bytes(<[u8; 4]>::try_from(_tick).unwrap()),
            universe_index: u32::from_le_bytes(<[u8; 4]>::try_from(_universe_index).unwrap()),
            siblings: siblings.try_into().unwrap()
        }

    }
}

impl FormatQubicResponseDataToStructure for IssuedAsset {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_issued_asset(response) }
}

pub fn handle_issued_asset(response: &mut QubicApiPacket) -> Option<IssuedAsset> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<OwnedAsset>()) {
        //println!("Wrong Size! {}, {:?}", data_len, &response.data[0..8]);
        return None;
    }
    Some(IssuedAsset::from_bytes(&response.data))
}

impl FormatQubicResponseDataToStructure for OwnedAsset {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_owned_asset(response) }
}

pub fn handle_owned_asset(response: &mut QubicApiPacket) -> Option<OwnedAsset> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<OwnedAsset>()) {
        //println!("Wrong Size! {}, {:?}", data_len, &response.data[0..8]);
        return None;
    }
    Some(OwnedAsset::from_bytes(&response.data))
}

impl FormatQubicResponseDataToStructure for PossessedAsset {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_possessed_asset(response) }
}

pub fn handle_possessed_asset(response: &mut QubicApiPacket) -> Option<PossessedAsset> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<PossessedAsset>()) {
        println!("{} vs {}", size_of::<OwnedAsset>(), size_of::<PossessedAsset>());
        println!("Wrong Size! {}, {:?} vs size of {}", data_len, &response.data[0..8],  std::mem::size_of::<PossessedAsset>());
        return None;
    }
    Some(PossessedAsset::from_bytes(&response.data))
}