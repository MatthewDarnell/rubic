//! QX `EntityAskOrders` / `EntityBidOrders`: the resting orders of one identity
//! across every asset, straight from the contract - no need to scan asset books.
use std::ffi::CStr;
use crypto::qubic_identities::get_public_key_from_identity;
use protocol::AsBytes;
use crate::qx::orderbook::RequestContractFunction;
use crate::qx::{QxFunctions, QX_CONTRACT_INDEX};

/// Orders per reply page (the contract returns a fixed-size array).
pub const ENTITY_ORDERS_PER_PAGE: usize = 256;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct QxGetEntityOrderInput {
    pub entity: [u8; 32],
    pub offset: u64,
}

impl QxGetEntityOrderInput {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        QxGetEntityOrderInput {
            entity: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            offset: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
        }
    }
}

impl AsBytes for QxGetEntityOrderInput {
    fn as_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::with_capacity(40);
        out.extend_from_slice(&self.entity);
        out.extend_from_slice(&self.offset.to_le_bytes());
        out
    }
}

/// One resting order of the entity, as the contract reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct EntityOrder {
    pub issuer: [u8; 32],
    pub asset_name: u64,
    pub price: i64,
    pub num_shares: i64,
}

impl EntityOrder {
    pub const SIZE: usize = 56;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() != Self::SIZE {
            return Err(());
        }
        Ok(EntityOrder {
            issuer: <[u8; 32]>::try_from(&bytes[0..32]).unwrap(),
            asset_name: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            price: i64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            num_shares: i64::from_le_bytes(bytes[48..56].try_into().unwrap()),
        })
    }

    /// Asset name as text (the contract packs up to 7 ASCII bytes, zero padded).
    pub fn asset_name_str(&self) -> String {
        let bytes = self.asset_name.to_le_bytes();
        match CStr::from_bytes_until_nul(&bytes) {
            Ok(s) => s.to_string_lossy().into_owned(),
            Err(_) => String::from_utf8_lossy(&bytes).trim_end_matches('\0').to_string(),
        }
    }
}

/// Parses a `RespondContractFunction` payload for functions 4/5 into the
/// non-empty orders it holds.
pub fn parse_entity_orders(data: &[u8]) -> Option<Vec<EntityOrder>> {
    if data.len() != EntityOrder::SIZE * ENTITY_ORDERS_PER_PAGE {
        return None;
    }
    Some(
        data.chunks_exact(EntityOrder::SIZE)
            .filter_map(|chunk| EntityOrder::from_bytes(chunk).ok())
            .filter(|o| o.price > 0 && o.num_shares > 0)
            .collect(),
    )
}

#[derive(Debug)]
#[repr(C)]
pub struct EntityOrdersRequest {
    pub rcf: RequestContractFunction,
    pub input: QxGetEntityOrderInput,
}

impl EntityOrdersRequest {
    pub fn new(function: QxFunctions, entity_identity: &str, offset: u64) -> Result<Self, String> {
        let entity = get_public_key_from_identity(&entity_identity.to_string())
            .map_err(|e| format!("{:?}", e))?;
        Ok(EntityOrdersRequest {
            rcf: RequestContractFunction::new(QX_CONTRACT_INDEX, function as u16, size_of::<QxGetEntityOrderInput>() as u16),
            input: QxGetEntityOrderInput { entity, offset },
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        EntityOrdersRequest {
            rcf: RequestContractFunction::from_bytes(bytes),
            input: QxGetEntityOrderInput::from_bytes(&bytes[size_of::<RequestContractFunction>()..]),
        }
    }

    /// "ASK" or "BID" depending on the function requested.
    pub fn side(&self) -> Option<&'static str> {
        match QxFunctions::from_u16(self.rcf.input_type) {
            Ok(QxFunctions::QxGetEntityAskOrder) => Some("ASK"),
            Ok(QxFunctions::QxGetEntityBidOrder) => Some("BID"),
            _ => None,
        }
    }
}

impl AsBytes for EntityOrdersRequest {
    fn as_bytes(&self) -> Vec<u8> {
        let mut out = self.rcf.as_bytes();
        out.extend_from_slice(self.input.as_bytes().as_slice());
        out
    }
}
