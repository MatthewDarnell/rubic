use std::cmp::Ordering;
use std::collections::BTreeSet;
use crypto::qubic_identities::get_public_key_from_identity;
use protocol::AsBytes;
use crate::qx::{QxFunctions, QX_CONTRACT_INDEX};

#[derive(Debug)]
#[repr(C)]
pub struct RequestContractFunction {
    pub contract_index: u32,
    pub input_type: u16,
    pub input_size: u16
}
impl RequestContractFunction {
    pub fn new(contract_index: u32, input_type: u16, input_size: u16) -> RequestContractFunction {
        RequestContractFunction {
            contract_index,
            input_type,
            input_size
        }
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        RequestContractFunction {
            contract_index: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            input_type: u16::from_le_bytes([bytes[4], bytes[5]]),
            input_size: u16::from_le_bytes([bytes[6], bytes[7]])
        }
    }
}

impl AsBytes for RequestContractFunction {
    fn as_bytes(&self) -> Vec<u8> {
        let mut ret_val: Vec<u8> = Vec::new();
        ret_val.extend_from_slice(&self.contract_index.to_le_bytes());
        ret_val.extend_from_slice(&self.input_type.to_le_bytes());
        ret_val.extend_from_slice(&self.input_size.to_le_bytes());
        ret_val
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
#[repr(C)]
pub struct AssetOrder {
    pub entity: [u8; 32],
    pub price: i64,
    pub num_shares: i64
}

impl PartialOrd for AssetOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.price.cmp(&other.price))
    }
}
impl Ord for AssetOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl AssetOrder {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() != size_of::<AssetOrder>() {
            Err(())
        } else {
            Ok(AssetOrder {
                entity: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
                price: i64::from_le_bytes((&bytes[32..40]).try_into().unwrap()),
                num_shares: i64::from_le_bytes((&bytes[40..48]).try_into().unwrap())
            })
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct QxGetAssetOrderInput {
    pub issuer: [u8; 32],
    pub asset_name: u64,
    pub offset: i64
}

impl QxGetAssetOrderInput {
    pub fn new(issuer: [u8; 32], asset_name: &str, offset: i64) -> QxGetAssetOrderInput {
        let mut name: [u8; 8] = [0; 8];
        name[0..asset_name.len()].copy_from_slice(asset_name.as_bytes());
        QxGetAssetOrderInput {
            issuer,
            asset_name: u64::from_le_bytes(name),
            offset
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        QxGetAssetOrderInput {
            issuer: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            asset_name: u64::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35], bytes[36], bytes[37], bytes[38], bytes[39]]),
            offset: i64::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43], bytes[44], bytes[45], bytes[46], bytes[47]])
        }
    }
}

impl AsBytes for QxGetAssetOrderInput {
    fn as_bytes(&self) -> Vec<u8> {
        let mut ret_val: Vec<u8> = Vec::new();
        ret_val.extend_from_slice(&self.issuer.as_slice());
        ret_val.extend_from_slice(&self.asset_name.to_le_bytes());
        ret_val.extend_from_slice(&self.offset.to_le_bytes());
        ret_val
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AssetOrdersRequest {
    pub rcf: RequestContractFunction,
    pub input: QxGetAssetOrderInput
}

impl AssetOrdersRequest {
    pub fn get_orderbook_side(&self) -> &str {
        let _type = QxFunctions::from_u16(self.rcf.input_type).unwrap();
        match _type {
            QxFunctions::QxGetAssetAskOrder => "ASK",
            QxFunctions::QxGetEntityAskOrder => "ASK",
            QxFunctions::QxGetAssetBidOrder => "BID",
            QxFunctions::QxGetEntityBidOrder => "BID",
            _ => "UNKNOWN"
        }    
    }
    
    pub fn new(function: QxFunctions, asset_name: &str, issuer: &str, offset: i64) -> Self {
        let issuer = get_public_key_from_identity(&issuer.to_string()).unwrap();
        let input: QxGetAssetOrderInput = QxGetAssetOrderInput::new(issuer, asset_name, offset);
        let rcf: RequestContractFunction = RequestContractFunction::new(QX_CONTRACT_INDEX, function as u16, size_of::<QxGetAssetOrderInput>() as u16);
        AssetOrdersRequest {
            rcf,
            input
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        AssetOrdersRequest {
            rcf: RequestContractFunction::from_bytes(bytes),
            input: QxGetAssetOrderInput::from_bytes(&bytes[size_of::<RequestContractFunction>()..])
        }
    }
}

impl AsBytes for AssetOrdersRequest {
    fn as_bytes(&self) -> Vec<u8> {
        let mut ret_val: Vec<u8> = Vec::new();
        ret_val.extend_from_slice(self.rcf.as_bytes().as_slice());
        ret_val.extend_from_slice(self.input.as_bytes().as_slice());
        ret_val
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub full_order_list: Vec<AssetOrder>,
    pub cached_order_set: BTreeSet<AssetOrder>
}
