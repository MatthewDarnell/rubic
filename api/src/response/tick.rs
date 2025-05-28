use crate::header::RequestResponseHeader;
use crate::QubicApiPacket;
use crate::response::FormatQubicResponseDataToStructure;

#[derive(Debug, Clone)]
pub struct Tick {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub millisecond: u8,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,

    pub prev_resource_testing_digest: u32,
    pub salted_resource_testing_digest: u32,
    
    pub prev_transaction_body_digest: u32,
    pub salted_transaction_body_digest: u32,
    
    pub prev_spectrum_digest: [u8; 32],
    pub prev_universe_digest: [u8; 32],
    pub prev_computer_digest: [u8; 32],
    pub salted_spectrum_digest: [u8; 32],
    pub salted_universe_digest: [u8; 32],
    pub salted_computer_digest: [u8; 32],
    
    
    pub transaction_digest: [u8; 32],
    pub expected_next_tick_transaction_digest: [u8; 32],

    
    signature: [u8; 64]
}

impl Tick {
    pub fn print(&self) {
        println!("-----\nTick:\n\ttick.({})\n\tcomputor.({})\n\tepoch.({})\n\t\n\tyear.({})\n\tsig: {:?}...\n-----",
                 self.tick,
                 self.computor_index,
                 self.epoch,
                self.year,
                 &self.signature[..8]);
    }
    pub fn new(data: &Vec<u8>) -> Tick {
        let(_, tick_bytes) = data.split_at(std::mem::size_of::<RequestResponseHeader>());
        Tick {
            computor_index: u16::from_le_bytes([tick_bytes[0], tick_bytes[1]]),
            epoch: u16::from_le_bytes([tick_bytes[2], tick_bytes[3]]),
            tick: u32::from_le_bytes([tick_bytes[4], tick_bytes[5], tick_bytes[6], tick_bytes[7]]),
            
            millisecond: u8::from_le_bytes([tick_bytes[8]]),
            second: u8::from_le_bytes([tick_bytes[9]]),
            minute: u8::from_le_bytes([tick_bytes[10]]),
            hour: u8::from_le_bytes([tick_bytes[11]]),
            day: u8::from_le_bytes([tick_bytes[12]]),
            month: u8::from_le_bytes([tick_bytes[13]]),
            year: u8::from_le_bytes([tick_bytes[14]]),

            prev_resource_testing_digest: u32::from_le_bytes([tick_bytes[15], tick_bytes[16], tick_bytes[17], tick_bytes[18]]),
            salted_resource_testing_digest: u32::from_le_bytes([tick_bytes[19], tick_bytes[20], tick_bytes[21], tick_bytes[22]]),

            prev_transaction_body_digest: u32::from_le_bytes([tick_bytes[23], tick_bytes[24], tick_bytes[25], tick_bytes[26]]),
            salted_transaction_body_digest: u32::from_le_bytes([tick_bytes[27], tick_bytes[28], tick_bytes[29], tick_bytes[30]]),

            
            prev_spectrum_digest: tick_bytes[31..63].try_into().unwrap(),
            prev_universe_digest: tick_bytes[63..95].try_into().unwrap(),
            prev_computer_digest: tick_bytes[95..127].try_into().unwrap(),
            salted_spectrum_digest: tick_bytes[127..159].try_into().unwrap(),
            salted_universe_digest: tick_bytes[159..191].try_into().unwrap(),
            salted_computer_digest: tick_bytes[191..223].try_into().unwrap(),

            transaction_digest: tick_bytes[223..255].try_into().unwrap(),
            expected_next_tick_transaction_digest: tick_bytes[255..287].try_into().unwrap(),
            
            signature: tick_bytes[287..351].try_into().unwrap()
        }
    }
}

impl FormatQubicResponseDataToStructure for Tick {
    fn format_qubic_response_data_to_structure(response: &mut QubicApiPacket) -> Option<Self> {handle_tick(response) }
}

pub fn handle_tick(response: &mut QubicApiPacket) -> Option<Tick> {
    let data: &Vec<u8> = &response.as_bytes();
    if data.len() < (std::mem::size_of::<RequestResponseHeader>() + std::mem::size_of::<Tick>()) {
        println!("Wrong Size! {}, {:?}", data.len(), &data[0..8]);
        return None;
    }
    Some(Tick::new(data))
}


#[test]
fn test_tick_size() {
    let sz = std::mem::size_of::<Tick>();
    assert_eq!(sz, 352);
}