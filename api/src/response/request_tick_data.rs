use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use consensus::tick_data::TickData;

impl FormatQubicResponseDataToStructure for TickData {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_tick_data(response) }
}

pub fn handle_tick_data(response: &mut QubicApiPacket) -> Option<TickData> {
    let data: &Vec<u8> = &response.as_bytes();
    //println!("{:?}", data);
    if data.len() != size_of::<RequestResponseHeader>() + size_of::<TickData>() {
        println!("Tick data length mismatch");
        return None;
    }
    let (_, right) = data.split_at(std::mem::size_of::<RequestResponseHeader>());
    Some(TickData::new(&right.to_vec()))
}