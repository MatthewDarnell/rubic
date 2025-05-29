use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;
use consensus::tick::Tick;

impl FormatQubicResponseDataToStructure for Tick {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_tick(response) }
}

pub fn handle_tick(response: &mut QubicApiPacket) -> Option<Tick> {
    let data_len =  std::mem::size_of::<RequestResponseHeader>() + response.data.len();
    if data_len != (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<Tick>()) {
        println!("Wrong Size! {}, {:?}", data_len, &response.data[0..8]);
        return None;
    }
    Some(Tick::new(&response.data))
}


#[test]
fn test_tick_size() {
    let sz = std::mem::size_of::<Tick>();
    assert_eq!(sz, 352);
}