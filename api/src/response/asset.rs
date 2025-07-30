#![allow(dead_code)]

use smart_contract::qx::asset::{IssuedAsset, OwnedAsset, PossessedAsset};
use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;

impl FormatQubicResponseDataToStructure for IssuedAsset {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_issued_asset(response) }
}

pub fn handle_issued_asset(response: &mut QubicApiPacket) -> Option<IssuedAsset> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<IssuedAsset>()) &&
        data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<IssuedAsset>() - IssuedAsset::siblings_size() ) {
        println!("Wrong Size! {}, {:?}", data_len, &response.data[0..8]);
        return None;
    } else {
        //println!("Data Size = {} vs {}", std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<IssuedAsset>() - IssuedAsset::siblings_size(), data_len);
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